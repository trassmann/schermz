use itertools::Itertools;
use serde_json::Value as JsonValue;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
enum ValueType {
    Null,
    Bool,
    Number,
    String(usize),
    Object(SchemaObject),
    Array(Vec<ValueType>),
}

#[derive(Debug, Clone)]
struct SchemaObjectKey {
    pub id: String,
    pub v_type: ValueType,
}

#[derive(Debug, Clone)]
pub struct SchemaObject {
    keys: Vec<SchemaObjectKey>,
}

impl ValueType {
    pub fn from_json(json: &JsonValue) -> Self {
        match json {
            JsonValue::Null => Self::Null,
            JsonValue::Bool(_) => Self::Bool,
            JsonValue::Number(_) => Self::Number,
            JsonValue::String(_) => {
                let str = json.as_str().unwrap();
                Self::String(str.len())
            }
            JsonValue::Object(_) => Self::Object(SchemaObject::from_json(json)),
            JsonValue::Array(arr) => {
                let values = arr.iter().map(Self::from_json).collect();
                Self::Array(values)
            }
        }
    }

    pub fn to_schema_value_type(&self, merge_objects: bool) -> SchemaValueType {
        match self {
            ValueType::Null => SchemaValueType::Primitive("NULL".into()),
            ValueType::Bool => SchemaValueType::Primitive("BOOL".into()),
            ValueType::Number => SchemaValueType::Primitive("NUMBER".into()),
            ValueType::Object(obj) => SchemaValueType::Object(Schema::from_objects(
                "object".into(),
                vec![obj.clone()],
                merge_objects,
            )),
            ValueType::Array(arr) => {
                let mut value_types = arr
                    .iter()
                    .map(|value_type| value_type.to_schema_value_type(merge_objects))
                    .collect::<Vec<SchemaValueType>>();

                value_types.dedup();

                SchemaValueType::Array(value_types)
            }
            _ => panic!("Invalid value type"),
        }
    }
}

impl SchemaObject {
    pub fn from_json(json: &JsonValue) -> Self {
        let mut keys = Vec::new();

        for (key, value) in json.as_object().unwrap() {
            keys.push(SchemaObjectKey {
                id: key.clone(),
                v_type: ValueType::from_json(value),
            });
        }
        Self { keys }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SchemaValueType {
    Primitive(String),
    String(usize, usize),
    Array(Vec<SchemaValueType>),
    Object(Schema),
}

impl SchemaValueType {
    pub fn to_json(&self) -> JsonValue {
        match self {
            SchemaValueType::Primitive(name) => JsonValue::String(name.clone()),
            SchemaValueType::String(min, max) => {
                if min == max {
                    return JsonValue::String(format!("STRING({})", min));
                }

                JsonValue::String(format!("STRING({}, {})", min, max))
            }
            SchemaValueType::Array(v_types) => {
                let types = v_types
                    .iter()
                    .map(|v| v.to_json())
                    .collect::<Vec<JsonValue>>();

                serde_json::json!({ "ARRAY": types })
            }
            SchemaValueType::Object(schema) => schema.to_json(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Schema {
    pub name: String,
    pub map: HashMap<String, Vec<SchemaValueType>>,
}

type CollectedObjects = HashMap<String, Vec<SchemaObject>>;

impl Schema {
    // Groups objects by a hash created from their keys
    fn group_objects_by_keys_fingerprint(objects: Vec<SchemaObject>) -> Vec<Vec<SchemaObject>> {
        objects
            .into_iter()
            .group_by(|obj| {
                let mut hasher = DefaultHasher::new();
                let sorted_keys = obj
                    .keys
                    .clone()
                    .into_iter()
                    .map(|obj_key| obj_key.id)
                    .sorted()
                    .collect::<Vec<String>>();
                let stringified_keys = sorted_keys.join("");
                stringified_keys.hash(&mut hasher);
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
                        // Collect all objects with the same key id into a vector
                        // so we can merge them together into a single schema
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
                                    let vtype = primitive_type.to_schema_value_type(merge_objects);
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
                        let vtype = primitive_type.to_schema_value_type(merge_objects);
                        if !entry.contains(&vtype) {
                            entry.push(vtype);
                        }
                    }
                }
            }
        }

        for (key, value) in string_lens {
            let min = value.iter().min().unwrap();
            let max = value.iter().max().unwrap();
            map.entry(key)
                .or_default()
                .push(SchemaValueType::String(*min, *max));
        }

        for (key, value) in object_types {
            match merge_objects {
                true => {
                    let name = key.clone();
                    map.entry(key)
                        .or_insert_with(Vec::new)
                        .push(SchemaValueType::Object(Schema::from_objects(
                            name, value, true,
                        )));
                }
                false => {
                    for objects_group in Self::group_objects_by_keys_fingerprint(value) {
                        map.entry(key.clone())
                            .or_default()
                            .push(SchemaValueType::Object(Schema::from_objects(
                                key.clone(),
                                objects_group,
                                false,
                            )));
                    }
                }
            }
        }

        for (key, value) in array_object_types {
            let mut all_array_types = Vec::new();

            match merge_objects {
                true => {
                    let schema = Schema::from_objects(key.clone(), value, true);
                    all_array_types = vec![SchemaValueType::Object(schema)];
                }
                false => {
                    for objects_group in Self::group_objects_by_keys_fingerprint(value) {
                        all_array_types.push(SchemaValueType::Object(Schema::from_objects(
                            key.clone(),
                            objects_group,
                            false,
                        )));
                    }
                }
            }

            if let Some(primitive_types) = array_primitive_types_map.get_mut(&key) {
                all_array_types.append(primitive_types);
            }
            if let Some(string_lens) = array_string_lens_map.get_mut(&key) {
                let min = string_lens.iter().min().unwrap();
                let max = string_lens.iter().max().unwrap();
                all_array_types.push(SchemaValueType::String(*min, *max));
            }
            map.entry(key)
                .or_default()
                .push(SchemaValueType::Array(all_array_types));
        }

        map
    }

    fn from_objects(name: String, objects: Vec<SchemaObject>, merge_objects: bool) -> Self {
        Self {
            name,
            map: Self::create_map(objects, merge_objects),
        }
    }

    pub fn to_json(&self) -> JsonValue {
        let mut map = serde_json::Map::new();

        for (key, value) in &self.map {
            let mut entry = serde_json::Map::new();
            let mut types = Vec::new();

            for vtype in value {
                types.push(vtype.to_json());
            }

            entry.insert("types".into(), serde_json::Value::Array(types));
            map.insert(key.clone(), serde_json::Value::Object(entry));
        }

        serde_json::Value::Object(map)
    }

    pub fn from_json(json: &JsonValue, merge_objects: bool) -> Self {
        match json {
            JsonValue::Object(_) => Self::from_objects(
                "root".into(),
                vec![SchemaObject::from_json(json)],
                merge_objects,
            ),
            JsonValue::Array(_) => {
                let objects = json
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter_map(|el| match el {
                        JsonValue::Object(_) => Some(SchemaObject::from_json(el)),
                        _ => None,
                    })
                    .collect::<Vec<SchemaObject>>();

                Self::from_objects("root".into(), objects, merge_objects)
            }
            _ => panic!("Invalid JSON"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn invalid_input() {
        Schema::from_json(&serde_json::Value::Null, false);
    }

    #[test]
    fn test_schema_from_object() {
        let json = serde_json::json!({
            "name": "John Doe",
            "title": "",
            "age": 43,
            "address": {
                "street": "10 Downing Street",
                "city": "London"
            },
            "phones": [
                "+44 1234567",
                "+44 2345678",
                123456,
                { "mobile": "+44 3456789" }
            ]
        });

        insta::assert_json_snapshot!(Schema::from_json(&json, true).to_json());
    }

    #[test]
    fn test_schema_from_array_merged() {
        let json = serde_json::json!([
            {
                "name": "Sherlock Holmes",
                "title": "",
                "age": 34,
                "personal_data": {
                    "gender": "male",
                    "marital_status": "single",
                },
                "address": {
                    "street": "10 Downing Street",
                    "city": "London",
                    "zip": "12345",
                    "country_code": "UK",
                },
                "phones": [
                    "+44 1234567",
                    "+44 2345678",
                    12311,
                    { "mobile": "+44 3456789" }
                ]
            },
            {
                "name": "Tony Soprano",
                "title": "",
                "age": 39,
                "personal_data": {
                    "gender": "male",
                    "marital_status": "married",
                },
                "address": {
                    "street": "14 Aspen Drive",
                    "city": "Caldwell",
                    "zip": "NJ 07006",
                    "country": "USA",
                    "state": "New Jersey",
                    "country_code": "US",
                },
                "phones": [
                    "+1 1234567",
                    "+1 2345678",
                    "+1 11111111111",
                    "+1 301234566",
                    11224234,
                    { "mobile": "+1 3456789" }
                ]
            },
            {
                "name": "Angela Merkel",
                "title": "",
                "age": 65,
                "personal_data": {
                    "gender": "female",
                    "marital_status": "married",
                },
                "address": {
                    "street": "Gr. Weg 3",
                    "city": "Potsdam",
                    "zip": "14467",
                    "country": "Germany",
                    "state": "Brandenburg",

                },
                "phones": [
                    "+49 1234222567",
                    "+49 2343231678",
                    "+49 1111131111111",
                    "+49 301212334566",
                    9999222,
                    { "mobile": "+49 343156789", "fax": "+49 343156780" }
                ]
            },
            {
                "name": "Jane Doe",
                "title": "Dr.",
                "age": "73",
                "personal_data": {
                    "gender": "female",
                },
                "address": null,
                "phones": null
            }
        ]);

        insta::assert_json_snapshot!(Schema::from_json(&json, true).to_json());
    }

    #[test]
    fn test_schema_from_array_unmerged() {
        let json = serde_json::json!([
            {
                "name": "Sherlock Holmes",
                "title": "",
                "age": 34,
                "personal_data": {
                    "gender": "male",
                    "marital_status": "single",
                },
                "address": {
                    "street": "10 Downing Street",
                    "city": "London",
                    "zip": "12345",
                    "country_code": "UK",
                },
                "phones": [
                    "+44 1234567",
                    "+44 2345678",
                    12311,
                    { "mobile": "+44 3456789" }
                ]
            },
            {
                "name": "Tony Soprano",
                "title": "",
                "age": 39,
                "personal_data": {
                    "gender": "male",
                    "marital_status": "married",
                },
                "address": {
                    "street": "14 Aspen Drive",
                    "city": "Caldwell",
                    "zip": "NJ 07006",
                    "country": "USA",
                    "state": "New Jersey",
                    "country_code": "US",
                },
                "phones": [
                    "+1 1234567",
                    "+1 2345678",
                    "+1 11111111111",
                    "+1 301234566",
                    11224234,
                    { "mobile": "+1 3456789" }
                ]
            },
            {
                "name": "Angela Merkel",
                "title": "",
                "age": 65,
                "personal_data": {
                    "gender": "female",
                    "marital_status": "married",
                },
                "address": {
                    "street": "Gr. Weg 3",
                    "city": "Potsdam",
                    "zip": "14467",
                    "country": "Germany",
                    "state": "Brandenburg",

                },
                "phones": [
                    "+49 1234222567",
                    "+49 2343231678",
                    "+49 1111131111111",
                    "+49 301212334566",
                    9999222,
                    { "mobile": "+49 343156789", "fax": "+49 343156780" }
                ]
            },
            {
                "name": "Jane Doe",
                "title": "Dr.",
                "age": "73",
                "personal_data": {
                    "gender": "female",
                },
                "address": null,
                "phones": null
            }
        ]);

        insta::assert_json_snapshot!(Schema::from_json(&json, false).to_json());
    }
}
