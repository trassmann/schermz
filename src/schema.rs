mod value_type;

#[cfg(test)]
mod tests;

use itertools::Itertools;
use serde_json::{Number, Value as JsonValue};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use value_type::{SchemaObject, ValueType};

// Drop string values from the enum candidate set if any one is longer than this.
// Long values (JSON blobs, base64, free-text) won't be useful as enum members
// downstream even if cardinality is small.
const MAX_ENUM_STRING_LEN: usize = 200;

/// Knobs for schema construction. Defaults match the CLI defaults.
#[derive(Debug, Clone)]
pub struct Config {
    pub merge_objects: bool,
    pub enum_threshold: usize,
    pub discriminator_max_arms: usize,
    /// When non-empty, only these field names are considered as candidate
    /// discriminators (in the listed order). When empty, the algorithm
    /// auto-picks among all qualifying candidates.
    pub discriminator_fields: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            merge_objects: false,
            enum_threshold: 30,
            discriminator_max_arms: 20,
            discriminator_fields: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SchemaValueType {
    Primitive(String),
    String(usize, usize),
    Array {
        types: Vec<SchemaValueType>,
        discriminator: Option<String>,
    },
    Object(Schema),
}

impl SchemaValueType {
    fn from_value_type(value_type: &ValueType, config: &Config) -> Self {
        match value_type {
            ValueType::Null => Self::Primitive("NULL".into()),
            ValueType::Bool => Self::Primitive("BOOL".into()),
            ValueType::Number(_) => Self::Primitive("NUMBER".into()),
            ValueType::String { len, .. } => Self::String(*len, *len),
            ValueType::Object(obj) => Self::Object(Schema::from_objects(vec![obj.clone()], config)),
            ValueType::Array(arr) => {
                let mut types = arr
                    .iter()
                    .map(|vt| Self::from_value_type(vt, config))
                    .collect::<Vec<_>>();
                types.dedup();
                Self::Array {
                    types,
                    discriminator: None,
                }
            }
        }
    }

    /// `inherited_discriminator` carries down the discriminator field name
    /// from an enclosing union (so the matching key in nested variant Schemas
    /// can be marked with `"discriminator": true`).
    fn to_json(&self, inherited_discriminator: Option<&str>) -> JsonValue {
        match self {
            SchemaValueType::Primitive(name) => JsonValue::String(name.clone()),
            SchemaValueType::String(min, max) => {
                if min == max {
                    JsonValue::String(format!("STRING({min})"))
                } else {
                    JsonValue::String(format!("STRING({min}, {max})"))
                }
            }
            SchemaValueType::Array {
                types,
                discriminator,
            } => {
                let arr: Vec<JsonValue> = types
                    .iter()
                    .map(|t| t.to_json(discriminator.as_deref()))
                    .collect();
                let mut obj = serde_json::Map::new();
                obj.insert("ARRAY".into(), JsonValue::Array(arr));
                if let Some(d) = discriminator {
                    obj.insert("discriminator".into(), JsonValue::String(d.clone()));
                }
                JsonValue::Object(obj)
            }
            SchemaValueType::Object(schema) => schema.to_json_with_hint(inherited_discriminator),
        }
    }
}

/// Bounded set: stops accepting new entries once it has more than `threshold`
/// distinct values. Used to cap enum-candidate accumulation so we don't waste
/// memory on high-cardinality fields.
#[derive(Debug, Clone, PartialEq)]
struct BoundedSet<T: Eq + Hash> {
    values: HashSet<T>,
    capped: bool,
}

impl<T: Eq + Hash> BoundedSet<T> {
    fn new() -> Self {
        Self {
            values: HashSet::new(),
            capped: false,
        }
    }

    fn insert(&mut self, value: T, threshold: usize) {
        if self.capped {
            return;
        }
        self.values.insert(value);
        if self.values.len() > threshold {
            self.capped = true;
        }
    }

    fn within_threshold(&self) -> Option<&HashSet<T>> {
        if self.capped {
            None
        } else {
            Some(&self.values)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct KeyEntry {
    types: Vec<SchemaValueType>,
    seen_count: usize,
    values: Option<Vec<JsonValue>>,
    /// Set when `types` contains 2+ Object variants and one of their fields
    /// uniquely discriminates them.
    discriminator: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Schema {
    parent_count: usize,
    map: HashMap<String, KeyEntry>,
}

type CollectedObjects = HashMap<String, Vec<SchemaObject>>;

impl Schema {
    fn group_objects_by_keys_fingerprint(objects: Vec<SchemaObject>) -> Vec<Vec<SchemaObject>> {
        // Fold objects with the same key set into the same group regardless of
        // their position. Ordering is by first occurrence, so deterministic and
        // source-order-friendly for snapshot tests.
        let mut groups: Vec<Vec<SchemaObject>> = Vec::new();
        let mut fp_to_idx: HashMap<String, usize> = HashMap::new();

        for obj in objects {
            let fp = key_fingerprint(&obj);
            if let Some(&idx) = fp_to_idx.get(&fp) {
                groups[idx].push(obj);
            } else {
                fp_to_idx.insert(fp, groups.len());
                groups.push(vec![obj]);
            }
        }
        groups
    }

    fn create_map(objects: Vec<SchemaObject>, config: &Config) -> HashMap<String, KeyEntry> {
        let mut types_map = HashMap::<String, Vec<SchemaValueType>>::new();
        let mut seen_counts = HashMap::<String, usize>::new();
        let mut string_lens = HashMap::<String, Vec<usize>>::new();
        let mut string_values = HashMap::<String, BoundedSet<String>>::new();
        let mut number_values = HashMap::<String, BoundedSet<Number>>::new();
        let mut object_types = CollectedObjects::new();
        let mut array_object_types = CollectedObjects::new();
        let mut array_primitive_types_map = HashMap::<String, Vec<SchemaValueType>>::new();
        let mut array_string_lens_map = HashMap::<String, Vec<usize>>::new();

        for obj in objects {
            for key in &obj.keys {
                *seen_counts.entry(key.id.clone()).or_insert(0) += 1;
                match &key.v_type {
                    ValueType::Object(obj) => {
                        object_types
                            .entry(key.id.clone())
                            .or_default()
                            .push(obj.clone());
                    }
                    ValueType::Array(arr) => {
                        for value_type in arr {
                            match value_type {
                                ValueType::Object(obj) => {
                                    array_object_types
                                        .entry(key.id.clone())
                                        .or_default()
                                        .push(obj.clone());
                                }
                                ValueType::String { len, .. } => {
                                    array_string_lens_map
                                        .entry(key.id.clone())
                                        .or_default()
                                        .push(*len);
                                }
                                primitive_type => {
                                    let entry = array_primitive_types_map
                                        .entry(key.id.clone())
                                        .or_default();
                                    let vtype =
                                        SchemaValueType::from_value_type(primitive_type, config);
                                    if !entry.contains(&vtype) {
                                        entry.push(vtype);
                                    }
                                }
                            }
                        }
                    }
                    ValueType::String { len, value } => {
                        string_lens.entry(key.id.clone()).or_default().push(*len);
                        string_values
                            .entry(key.id.clone())
                            .or_insert_with(BoundedSet::new)
                            .insert(value.clone(), config.enum_threshold);
                    }
                    ValueType::Number(num) => {
                        let entry = types_map.entry(key.id.clone()).or_default();
                        let vtype = SchemaValueType::Primitive("NUMBER".into());
                        if !entry.contains(&vtype) {
                            entry.push(vtype);
                        }
                        number_values
                            .entry(key.id.clone())
                            .or_insert_with(BoundedSet::new)
                            .insert(num.clone(), config.enum_threshold);
                    }
                    primitive_type => {
                        let entry = types_map.entry(key.id.clone()).or_default();
                        let vtype = SchemaValueType::from_value_type(primitive_type, config);
                        if !entry.contains(&vtype) {
                            entry.push(vtype);
                        }
                    }
                }
            }
        }

        for (key, lens) in string_lens {
            let min = *lens.iter().min().unwrap();
            let max = *lens.iter().max().unwrap();
            types_map
                .entry(key)
                .or_default()
                .push(SchemaValueType::String(min, max));
        }

        for (key, value) in object_types {
            if config.merge_objects {
                types_map
                    .entry(key)
                    .or_default()
                    .push(SchemaValueType::Object(Schema::from_objects(value, config)));
            } else {
                for objects_group in Self::group_objects_by_keys_fingerprint(value) {
                    types_map
                        .entry(key.clone())
                        .or_default()
                        .push(SchemaValueType::Object(Schema::from_objects(
                            objects_group,
                            config,
                        )));
                }
            }
        }

        // Iterate the union of keys across all three array maps so arrays whose
        // contents are only primitives or only strings still appear in the output.
        let array_keys: HashSet<String> = array_object_types
            .keys()
            .chain(array_primitive_types_map.keys())
            .chain(array_string_lens_map.keys())
            .cloned()
            .collect();

        for key in array_keys {
            let mut all_array_types: Vec<SchemaValueType> = match array_object_types.remove(&key) {
                Some(value) if config.merge_objects => {
                    vec![SchemaValueType::Object(Schema::from_objects(value, config))]
                }
                Some(value) => Self::group_objects_by_keys_fingerprint(value)
                    .into_iter()
                    .map(|group| SchemaValueType::Object(Schema::from_objects(group, config)))
                    .collect(),
                None => Vec::new(),
            };

            if let Some(primitive_types) = array_primitive_types_map.get_mut(&key) {
                all_array_types.append(primitive_types);
            }
            if let Some(string_lens) = array_string_lens_map.get_mut(&key) {
                let min = *string_lens.iter().min().unwrap();
                let max = *string_lens.iter().max().unwrap();
                all_array_types.push(SchemaValueType::String(min, max));
            }
            let array_discriminator = compute_discriminator(&all_array_types, config);
            types_map
                .entry(key)
                .or_default()
                .push(SchemaValueType::Array {
                    types: all_array_types,
                    discriminator: array_discriminator,
                });
        }

        types_map
            .into_iter()
            .map(|(key, types)| {
                let seen_count = seen_counts.remove(&key).unwrap_or(0);
                let values =
                    enum_values_for_key(&types, string_values.get(&key), number_values.get(&key));
                let discriminator = compute_discriminator(&types, config);
                (
                    key,
                    KeyEntry {
                        types,
                        seen_count,
                        values,
                        discriminator,
                    },
                )
            })
            .collect()
    }

    fn from_objects(objects: Vec<SchemaObject>, config: &Config) -> Self {
        let parent_count = objects.len();
        Self {
            parent_count,
            map: Self::create_map(objects, config),
        }
    }

    pub fn to_json(&self) -> JsonValue {
        self.to_json_with_hint(None)
    }

    fn to_json_with_hint(&self, inherited_discriminator: Option<&str>) -> JsonValue {
        let mut map = serde_json::Map::new();

        for (key, entry) in &self.map {
            let mut out = serde_json::Map::new();
            let types: Vec<JsonValue> = entry
                .types
                .iter()
                .map(|t| t.to_json(entry.discriminator.as_deref()))
                .collect();
            out.insert("types".into(), JsonValue::Array(types));
            if entry.seen_count < self.parent_count {
                out.insert("optional".into(), JsonValue::Bool(true));
            }
            if let Some(values) = &entry.values {
                out.insert("values".into(), JsonValue::Array(values.clone()));
            }
            if let Some(d) = &entry.discriminator {
                out.insert("discriminator".into(), JsonValue::String(d.clone()));
            }
            // If the enclosing union picked THIS key as its discriminator, mark it.
            // This may overwrite the per-key field-level "discriminator" string set
            // above — but only on a leaf scalar, where no inner union exists.
            if Some(key.as_str()) == inherited_discriminator {
                out.insert("discriminator".into(), JsonValue::Bool(true));
            }
            map.insert(key.clone(), JsonValue::Object(out));
        }

        JsonValue::Object(map)
    }

    pub fn from_json(json: &JsonValue, config: &Config) -> Self {
        match json {
            JsonValue::Object(_) => Self::from_objects(vec![SchemaObject::from_json(json)], config),
            JsonValue::Array(arr) => {
                let objects = arr
                    .iter()
                    .filter_map(|el| match el {
                        JsonValue::Object(_) => Some(SchemaObject::from_json(el)),
                        _ => None,
                    })
                    .collect::<Vec<SchemaObject>>();

                Self::from_objects(objects, config)
            }
            _ => panic!("schermz expects the root JSON value to be an object or an array"),
        }
    }
}

/// Stable identity for an object's key set. Used to fold structurally
/// identical objects into the same variant during unmerged grouping.
fn key_fingerprint(obj: &SchemaObject) -> String {
    obj.keys
        .iter()
        .map(|k| k.id.as_str())
        .sorted()
        .collect::<Vec<_>>()
        // U+001E (record separator) keeps `["ab", "c"]` and `["a", "bc"]`
        // distinct without colliding with anything that turns up in JSON keys.
        .join("\u{001e}")
}

/// Decide whether a key's distinct scalar values should be emitted as a
/// `"values"` enum annotation. A single non-null scalar variant (string XOR
/// number) is required; mixed-type unions are skipped to keep downstream codegen
/// rules simple.
fn enum_values_for_key(
    types: &[SchemaValueType],
    string_values: Option<&BoundedSet<String>>,
    number_values: Option<&BoundedSet<Number>>,
) -> Option<Vec<JsonValue>> {
    let mut has_string = false;
    let mut has_number = false;
    let mut has_other = false;
    for t in types {
        match t {
            SchemaValueType::String(_, _) => has_string = true,
            SchemaValueType::Primitive(name) if name == "NUMBER" => has_number = true,
            SchemaValueType::Primitive(name) if name == "NULL" => {}
            _ => has_other = true,
        }
    }
    if has_other {
        return None;
    }
    match (has_string, has_number) {
        (true, false) => emit_string_values(string_values?),
        (false, true) => emit_number_values(number_values?),
        _ => None,
    }
}

fn emit_string_values(set: &BoundedSet<String>) -> Option<Vec<JsonValue>> {
    let values = set.within_threshold()?;
    if values.iter().any(|s| s.len() > MAX_ENUM_STRING_LEN) {
        return None;
    }
    let mut sorted: Vec<&String> = values.iter().collect();
    sorted.sort();
    Some(
        sorted
            .into_iter()
            .map(|s| JsonValue::String(s.clone()))
            .collect(),
    )
}

fn emit_number_values(set: &BoundedSet<Number>) -> Option<Vec<JsonValue>> {
    let values = set.within_threshold()?;
    if !values.iter().all(|n| n.is_i64() || n.is_u64()) {
        return None;
    }
    let mut sorted: Vec<&Number> = values.iter().collect();
    sorted.sort_by_key(|n| {
        n.as_i64()
            .or_else(|| n.as_u64().map(|u| u as i64))
            .unwrap_or(0)
    });
    Some(
        sorted
            .into_iter()
            .map(|n| JsonValue::Number(n.clone()))
            .collect(),
    )
}

/// Find a single field whose values uniquely identify each Object variant in
/// `types`. Returns `None` if there are fewer than 2 Object variants, no field
/// qualifies, or the union exceeds `discriminator_max_arms` arms.
fn compute_discriminator(types: &[SchemaValueType], config: &Config) -> Option<String> {
    let variants: Vec<&Schema> = types
        .iter()
        .filter_map(|t| match t {
            SchemaValueType::Object(s) => Some(s),
            _ => None,
        })
        .collect();
    if variants.len() < 2 {
        return None;
    }

    // Step 1: fields present in every variant.
    let mut common: Vec<&str> = variants[0].map.keys().map(String::as_str).collect();
    common.retain(|f| variants[1..].iter().all(|v| v.map.contains_key(*f)));

    // Step 2: present-and-required-and-has-values in every variant.
    let with_values: Vec<&str> = common
        .into_iter()
        .filter(|f| {
            variants.iter().all(|v| {
                let entry = match v.map.get(*f) {
                    Some(e) => e,
                    None => return false,
                };
                entry.values.is_some() && entry.seen_count == v.parent_count
            })
        })
        .collect();

    // Step 3: pairwise-disjoint values across variants.
    let disjoint: Vec<&str> = with_values
        .into_iter()
        .filter(|f| variant_values_pairwise_disjoint(&variants, f))
        .collect();

    // Step 4: total cardinality cap.
    let mut survivors: Vec<(&str, usize)> = disjoint
        .into_iter()
        .filter_map(|f| {
            let total: usize = variants
                .iter()
                .map(|v| v.map[f].values.as_ref().map(|vs| vs.len()).unwrap_or(0))
                .sum();
            if total <= config.discriminator_max_arms {
                Some((f, total))
            } else {
                None
            }
        })
        .collect();

    if !config.discriminator_fields.is_empty() {
        // User-named priority list. Try each in order; first hit wins.
        for user_field in &config.discriminator_fields {
            if survivors.iter().any(|(f, _)| f == user_field) {
                return Some(user_field.clone());
            }
        }
        return None;
    }

    // Auto: smallest total cardinality, tie-break by name lexicographically.
    survivors.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(b.0)));
    survivors.first().map(|(f, _)| (*f).to_string())
}

fn variant_values_pairwise_disjoint(variants: &[&Schema], field: &str) -> bool {
    let value_sets: Vec<&[JsonValue]> = variants
        .iter()
        .map(|v| {
            v.map[field]
                .values
                .as_deref()
                .expect("checked Some in caller")
        })
        .collect();
    for i in 0..value_sets.len() {
        for j in (i + 1)..value_sets.len() {
            if value_sets[i].iter().any(|v| value_sets[j].contains(v)) {
                return false;
            }
        }
    }
    true
}
