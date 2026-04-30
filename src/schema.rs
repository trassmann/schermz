mod value_type;

#[cfg(test)]
mod tests;

use itertools::Itertools;
use serde_json::{Number, Value as JsonValue};
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

use value_type::{SchemaObject, ValueType};

// Drop string values from the enum candidate set if any one is longer than this.
// Long values (JSON blobs, base64, free-text) won't be useful as enum members
// downstream even if cardinality is small.
const MAX_ENUM_STRING_LEN: usize = 200;

#[derive(Debug, Clone, PartialEq)]
pub enum SchemaValueType {
    Primitive(String),
    String(usize, usize),
    Array(Vec<SchemaValueType>),
    Object(Schema),
}

impl SchemaValueType {
    fn from_value_type(value_type: &ValueType, merge_objects: bool, enum_threshold: usize) -> Self {
        match value_type {
            ValueType::Null => Self::Primitive("NULL".into()),
            ValueType::Bool => Self::Primitive("BOOL".into()),
            ValueType::Number(_) => Self::Primitive("NUMBER".into()),
            ValueType::String { len, .. } => Self::String(*len, *len),
            ValueType::Object(obj) => Self::Object(Schema::from_objects(
                vec![obj.clone()],
                merge_objects,
                enum_threshold,
            )),
            ValueType::Array(arr) => {
                let mut value_types = arr
                    .iter()
                    .map(|vt| Self::from_value_type(vt, merge_objects, enum_threshold))
                    .collect::<Vec<_>>();
                value_types.dedup();
                Self::Array(value_types)
            }
        }
    }

    pub fn to_json(&self) -> JsonValue {
        match self {
            SchemaValueType::Primitive(name) => JsonValue::String(name.clone()),
            SchemaValueType::String(min, max) => {
                if min == max {
                    JsonValue::String(format!("STRING({min})"))
                } else {
                    JsonValue::String(format!("STRING({min}, {max})"))
                }
            }
            SchemaValueType::Array(v_types) => {
                let types = v_types
                    .iter()
                    .map(SchemaValueType::to_json)
                    .collect::<Vec<JsonValue>>();
                serde_json::json!({ "ARRAY": types })
            }
            SchemaValueType::Object(schema) => schema.to_json(),
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

    /// Returns the underlying set if the threshold was never exceeded.
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
}

#[derive(Debug, Clone, PartialEq)]
pub struct Schema {
    parent_count: usize,
    map: HashMap<String, KeyEntry>,
}

type CollectedObjects = HashMap<String, Vec<SchemaObject>>;

impl Schema {
    fn group_objects_by_keys_fingerprint(objects: Vec<SchemaObject>) -> Vec<Vec<SchemaObject>> {
        objects
            .into_iter()
            .chunk_by(|obj| {
                let mut hasher = DefaultHasher::new();
                let sorted_keys = obj
                    .keys
                    .iter()
                    .map(|obj_key| obj_key.id.as_str())
                    .sorted()
                    .collect::<Vec<_>>();
                sorted_keys.join("").hash(&mut hasher);
                hasher.finish()
            })
            .into_iter()
            .map(|(_, gr)| gr.collect_vec())
            .collect()
    }

    fn create_map(
        objects: Vec<SchemaObject>,
        merge_objects: bool,
        enum_threshold: usize,
    ) -> HashMap<String, KeyEntry> {
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
                                    let vtype = SchemaValueType::from_value_type(
                                        primitive_type,
                                        merge_objects,
                                        enum_threshold,
                                    );
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
                            .insert(value.clone(), enum_threshold);
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
                            .insert(num.clone(), enum_threshold);
                    }
                    primitive_type => {
                        let entry = types_map.entry(key.id.clone()).or_default();
                        let vtype = SchemaValueType::from_value_type(
                            primitive_type,
                            merge_objects,
                            enum_threshold,
                        );
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
            if merge_objects {
                types_map
                    .entry(key)
                    .or_default()
                    .push(SchemaValueType::Object(Schema::from_objects(
                        value,
                        true,
                        enum_threshold,
                    )));
            } else {
                for objects_group in Self::group_objects_by_keys_fingerprint(value) {
                    types_map
                        .entry(key.clone())
                        .or_default()
                        .push(SchemaValueType::Object(Schema::from_objects(
                            objects_group,
                            false,
                            enum_threshold,
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
                Some(value) if merge_objects => {
                    vec![SchemaValueType::Object(Schema::from_objects(
                        value,
                        true,
                        enum_threshold,
                    ))]
                }
                Some(value) => Self::group_objects_by_keys_fingerprint(value)
                    .into_iter()
                    .map(|group| {
                        SchemaValueType::Object(Schema::from_objects(group, false, enum_threshold))
                    })
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
            types_map
                .entry(key)
                .or_default()
                .push(SchemaValueType::Array(all_array_types));
        }

        types_map
            .into_iter()
            .map(|(key, types)| {
                let seen_count = seen_counts.remove(&key).unwrap_or(0);
                let values =
                    enum_values_for_key(&types, string_values.get(&key), number_values.get(&key));
                (
                    key,
                    KeyEntry {
                        types,
                        seen_count,
                        values,
                    },
                )
            })
            .collect()
    }

    fn from_objects(
        objects: Vec<SchemaObject>,
        merge_objects: bool,
        enum_threshold: usize,
    ) -> Self {
        let parent_count = objects.len();
        Self {
            parent_count,
            map: Self::create_map(objects, merge_objects, enum_threshold),
        }
    }

    pub fn to_json(&self) -> JsonValue {
        let mut map = serde_json::Map::new();

        for (key, entry) in &self.map {
            let mut out = serde_json::Map::new();
            let types: Vec<JsonValue> = entry.types.iter().map(SchemaValueType::to_json).collect();
            out.insert("types".into(), JsonValue::Array(types));
            if entry.seen_count < self.parent_count {
                out.insert("optional".into(), JsonValue::Bool(true));
            }
            if let Some(values) = &entry.values {
                out.insert("values".into(), JsonValue::Array(values.clone()));
            }
            map.insert(key.clone(), JsonValue::Object(out));
        }

        JsonValue::Object(map)
    }

    pub fn from_json(json: &JsonValue, merge_objects: bool, enum_threshold: usize) -> Self {
        match json {
            JsonValue::Object(_) => Self::from_objects(
                vec![SchemaObject::from_json(json)],
                merge_objects,
                enum_threshold,
            ),
            JsonValue::Array(arr) => {
                let objects = arr
                    .iter()
                    .filter_map(|el| match el {
                        JsonValue::Object(_) => Some(SchemaObject::from_json(el)),
                        _ => None,
                    })
                    .collect::<Vec<SchemaObject>>();

                Self::from_objects(objects, merge_objects, enum_threshold)
            }
            _ => panic!("schermz expects the root JSON value to be an object or an array"),
        }
    }
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
        // Floats aren't useful as enum members.
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
