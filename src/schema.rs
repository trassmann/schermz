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

    pub fn to_schema_value_type(&self) -> SchemaValueType {
        match self {
            ValueType::NULL => SchemaValueType::PRIMITIVE("NULL".into()),
            ValueType::BOOL => SchemaValueType::PRIMITIVE("BOOL".into()),
            ValueType::NUMBER => SchemaValueType::PRIMITIVE("NUMBER".into()),
            ValueType::STRING => SchemaValueType::PRIMITIVE("STRING".into()),
            ValueType::OBJECT(obj) => {
                SchemaValueType::OBJECT(Schema::from_objects("object".into(), vec![obj.clone()]))
            }
            ValueType::ARRAY(arr) => {
                let mut value_types = arr
                    .iter()
                    .map(|value_type| value_type.to_schema_value_type())
                    .collect::<Vec<SchemaValueType>>();

                value_types.dedup();

                SchemaValueType::ARRAY(value_types)
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

#[derive(Debug, Clone, PartialEq)]
pub enum SchemaValueType {
    PRIMITIVE(String),
    ARRAY(Vec<SchemaValueType>),
    OBJECT(Schema),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Schema {
    pub name: String,
    pub map: HashMap<String, Vec<SchemaValueType>>,
}

type CollectedObjects = HashMap<String, Vec<SchemaObject>>;

impl Schema {
    fn create_map(
        objects: Vec<SchemaObject>,
        parent_key: Option<&String>,
    ) -> HashMap<String, Vec<SchemaValueType>> {
        let mut map = HashMap::<String, Vec<SchemaValueType>>::new();
        let mut object_types = CollectedObjects::new();

        for obj in objects {
            for key in &obj.keys {
                match &key.v_type {
                    ValueType::OBJECT(obj) => {
                        // Collect all objects with the same key id into a vector
                        // so we can merge them together into a single schema
                        object_types
                            .entry(key.id.clone())
                            .or_insert_with(Vec::new)
                            .push(obj.clone());
                    }
                    primitive_type => {
                        let entry = map.entry(key.id.clone()).or_insert_with(Vec::new);
                        let vtype = primitive_type.to_schema_value_type();
                        if !entry.contains(&vtype) {
                            entry.push(vtype);
                        }
                    }
                }
            }
        }

        for (key, value) in object_types {
            let name = match parent_key {
                Some(parent_key) => {
                    format!("{}.{}", parent_key, key)
                }
                None => key.clone(),
            };
            map.entry(key)
                .or_insert_with(Vec::new)
                .push(SchemaValueType::OBJECT(Schema::from_objects(name, value)));
        }

        map
    }

    fn from_objects(name: String, objects: Vec<SchemaObject>) -> Self {
        let name_cpy = name.clone();
        let parent_key = Some(&name_cpy);
        Self {
            name,
            map: Self::create_map(objects, parent_key),
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
