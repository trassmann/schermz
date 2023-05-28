use serde_json::Value as JsonValue;
use std::collections::HashMap;

#[derive(Debug, Clone)]
enum ValueType {
    NULL,
    BOOL,
    NUMBER,
    STRING,
    OBJECT(SchemaObject),
    ARRAY(Vec<ValueType>),
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
            JsonValue::Null => Self::NULL,
            JsonValue::Bool(_) => Self::BOOL,
            JsonValue::Number(_) => Self::NUMBER,
            JsonValue::String(_) => Self::STRING,
            JsonValue::Object(_) => Self::OBJECT(SchemaObject::from_json(json)),
            JsonValue::Array(arr) => {
                let values = arr.iter().map(|value| Self::from_json(value)).collect();
                Self::ARRAY(values)
            }
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

#[derive(Debug, Clone)]
pub enum SchemaValueType {
    PRIMITIVE(String),
    ARRAY(Vec<SchemaValueType>),
    OBJECT(Schema),
}

#[derive(Debug, Clone)]
pub struct Schema {
    pub name: String,
    pub map: HashMap<String, Vec<SchemaValueType>>,
}

#[derive(Debug, Clone)]
struct KeyObjects(String, Vec<SchemaObject>);

impl Schema {
    fn create_map(objects: Vec<SchemaObject>) -> HashMap<String, Vec<SchemaValueType>> {
        let mut map = HashMap::<String, Vec<SchemaValueType>>::new();
        let mut key_objects = Vec::<KeyObjects>::new();

        for obj in objects {
            for key in &obj.keys {
                match &key.v_type {
                    ValueType::NULL => {
                        map.entry(key.id.clone())
                            .or_insert_with(Vec::new)
                            .push(SchemaValueType::PRIMITIVE("NULL".into()));
                    }
                    ValueType::BOOL => {
                        map.entry(key.id.clone())
                            .or_insert_with(Vec::new)
                            .push(SchemaValueType::PRIMITIVE("BOOL".into()));
                    }
                    ValueType::NUMBER => {
                        map.entry(key.id.clone())
                            .or_insert_with(Vec::new)
                            .push(SchemaValueType::PRIMITIVE("NUMBER".into()));
                    }
                    ValueType::STRING => {
                        map.entry(key.id.clone())
                            .or_insert_with(Vec::new)
                            .push(SchemaValueType::PRIMITIVE("STRING".into()));
                    }
                    ValueType::ARRAY(arr) => {
                        // @TODO: Impl ValueType::to_schema_value_type
                        map.entry(key.id.clone()).or_insert_with(Vec::new).push(
                            SchemaValueType::ARRAY(
                                arr.iter()
                                    .map(|value_type| match value_type {
                                        ValueType::NULL => {
                                            SchemaValueType::PRIMITIVE("NULL".into())
                                        }
                                        ValueType::BOOL => {
                                            SchemaValueType::PRIMITIVE("BOOL".into())
                                        }
                                        ValueType::NUMBER => {
                                            SchemaValueType::PRIMITIVE("NUMBER".into())
                                        }
                                        ValueType::STRING => {
                                            SchemaValueType::PRIMITIVE("STRING".into())
                                        }
                                        ValueType::OBJECT(obj) => {
                                            SchemaValueType::OBJECT(Schema::from_objects(
                                                "array-object".into(),
                                                vec![obj.clone()],
                                            ))
                                        }
                                        ValueType::ARRAY(_) => {
                                            SchemaValueType::PRIMITIVE("ARRAY".into())
                                        }
                                    })
                                    .collect(),
                            ),
                        );
                    }
                    ValueType::OBJECT(obj) => {
                        let existing_key_object = key_objects
                            .iter_mut()
                            .find(|key_object| key_object.0 == key.id);

                        match existing_key_object {
                            Some(key_object) => {
                                key_object.1.push(obj.clone());
                            }
                            None => {
                                key_objects.push(KeyObjects(key.id.clone(), vec![obj.clone()]));
                            }
                        }
                    }
                }
            }
        }

        for key_object in key_objects {
            let name = format!("{}-{}", &key_object.0, "object");

            map.entry(key_object.0)
                .or_insert_with(Vec::new)
                .push(SchemaValueType::OBJECT(Schema::from_objects(
                    name,
                    key_object.1,
                )));
        }

        map
    }

    fn from_objects(name: String, objects: Vec<SchemaObject>) -> Self {
        Self {
            name,
            map: Self::create_map(objects),
        }
    }

    pub fn from_json(json: &JsonValue) -> Self {
        let objects = json
            .as_array()
            .unwrap()
            .iter()
            .map(|obj| SchemaObject::from_json(obj))
            .collect::<Vec<SchemaObject>>();

        Self::from_objects("root".into(), objects)
    }
}
