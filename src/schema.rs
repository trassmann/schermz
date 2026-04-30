mod value_type;

#[cfg(test)]
mod tests;

use itertools::Itertools;
use serde_json::Value as JsonValue;
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

use value_type::{SchemaObject, ValueType};

#[derive(Debug, Clone, PartialEq)]
pub enum SchemaValueType {
    Primitive(String),
    String(usize, usize),
    Array(Vec<SchemaValueType>),
    Object(Schema),
}

impl SchemaValueType {
    fn from_value_type(value_type: &ValueType, merge_objects: bool) -> Self {
        match value_type {
            ValueType::Null => Self::Primitive("NULL".into()),
            ValueType::Bool => Self::Primitive("BOOL".into()),
            ValueType::Number => Self::Primitive("NUMBER".into()),
            ValueType::String(len) => Self::String(*len, *len),
            ValueType::Object(obj) => {
                Self::Object(Schema::from_objects(vec![obj.clone()], merge_objects))
            }
            ValueType::Array(arr) => {
                let mut value_types = arr
                    .iter()
                    .map(|vt| Self::from_value_type(vt, merge_objects))
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

#[derive(Debug, Clone, PartialEq)]
pub struct Schema {
    pub map: HashMap<String, Vec<SchemaValueType>>,
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
    ) -> HashMap<String, Vec<SchemaValueType>> {
        let mut map = HashMap::<String, Vec<SchemaValueType>>::new();
        let mut string_lens = HashMap::<String, Vec<usize>>::new();
        let mut object_types = CollectedObjects::new();
        let mut array_object_types = CollectedObjects::new();
        let mut array_primitive_types_map = HashMap::<String, Vec<SchemaValueType>>::new();
        let mut array_string_lens_map = HashMap::<String, Vec<usize>>::new();

        for obj in objects {
            for key in &obj.keys {
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
                                ValueType::String(len) => {
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
                                    );
                                    if !entry.contains(&vtype) {
                                        entry.push(vtype);
                                    }
                                }
                            }
                        }
                    }
                    ValueType::String(len) => {
                        string_lens.entry(key.id.clone()).or_default().push(*len);
                    }
                    primitive_type => {
                        let entry = map.entry(key.id.clone()).or_default();
                        let vtype = SchemaValueType::from_value_type(primitive_type, merge_objects);
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
            map.entry(key)
                .or_default()
                .push(SchemaValueType::String(min, max));
        }

        for (key, value) in object_types {
            if merge_objects {
                map.entry(key)
                    .or_default()
                    .push(SchemaValueType::Object(Schema::from_objects(value, true)));
            } else {
                for objects_group in Self::group_objects_by_keys_fingerprint(value) {
                    map.entry(key.clone())
                        .or_default()
                        .push(SchemaValueType::Object(Schema::from_objects(
                            objects_group,
                            false,
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
                    vec![SchemaValueType::Object(Schema::from_objects(value, true))]
                }
                Some(value) => Self::group_objects_by_keys_fingerprint(value)
                    .into_iter()
                    .map(|group| SchemaValueType::Object(Schema::from_objects(group, false)))
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
            map.entry(key)
                .or_default()
                .push(SchemaValueType::Array(all_array_types));
        }

        map
    }

    fn from_objects(objects: Vec<SchemaObject>, merge_objects: bool) -> Self {
        Self {
            map: Self::create_map(objects, merge_objects),
        }
    }

    pub fn to_json(&self) -> JsonValue {
        let mut map = serde_json::Map::new();

        for (key, value) in &self.map {
            let mut entry = serde_json::Map::new();
            let types: Vec<JsonValue> = value.iter().map(SchemaValueType::to_json).collect();
            entry.insert("types".into(), JsonValue::Array(types));
            map.insert(key.clone(), JsonValue::Object(entry));
        }

        JsonValue::Object(map)
    }

    pub fn from_json(json: &JsonValue, merge_objects: bool) -> Self {
        match json {
            JsonValue::Object(_) => {
                Self::from_objects(vec![SchemaObject::from_json(json)], merge_objects)
            }
            JsonValue::Array(arr) => {
                let objects = arr
                    .iter()
                    .filter_map(|el| match el {
                        JsonValue::Object(_) => Some(SchemaObject::from_json(el)),
                        _ => None,
                    })
                    .collect::<Vec<SchemaObject>>();

                Self::from_objects(objects, merge_objects)
            }
            _ => panic!("schermz expects the root JSON value to be an object or an array"),
        }
    }
}
