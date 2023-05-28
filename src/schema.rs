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
    fn create_map(objects: Vec<SchemaObject>) -> HashMap<String, Vec<SchemaValueType>> {
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
            let name = key.clone();
            map.entry(key)
                .or_insert_with(Vec::new)
                .push(SchemaValueType::OBJECT(Schema::from_objects(name, value)));
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
        match json {
            JsonValue::Object(_) => {
                Self::from_objects("root".into(), vec![SchemaObject::from_json(json)])
            }
            JsonValue::Array(_) => {
                let objects = json
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|obj| SchemaObject::from_json(obj))
                    .collect::<Vec<SchemaObject>>();

                Self::from_objects("root".into(), objects)
            }
            _ => panic!("Invalid input JSON"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use std::fs;

    #[test]
    #[should_panic]
    fn invalid_input() {
        Schema::from_json(&serde_json::Value::Null);
    }

    #[test]
    fn test_schema_from_object() {
        let json = serde_json::json!({
            "name": "John Doe",
            "age": 43,
            "address": {
                "street": "10 Downing Street",
                "city": "London"
            },
            "phones": [
                "+44 1234567",
                "+44 2345678"
            ]
        });

        let schema = Schema::from_json(&json);

        assert_eq!(schema.name, "root");
        assert_eq!(schema.map.len(), 4);
        assert_eq!(
            schema.map.get("name").unwrap(),
            &vec![SchemaValueType::PRIMITIVE("STRING".into())]
        );
        assert_eq!(
            schema.map.get("age").unwrap(),
            &vec![SchemaValueType::PRIMITIVE("NUMBER".into())]
        );
        assert_eq!(
            schema.map.get("address").unwrap(),
            &vec![SchemaValueType::OBJECT(Schema {
                name: "address".into(),
                map: vec![
                    (
                        "street".into(),
                        vec![SchemaValueType::PRIMITIVE("STRING".into())]
                    ),
                    (
                        "city".into(),
                        vec![SchemaValueType::PRIMITIVE("STRING".into())]
                    )
                ]
                .into_iter()
                .collect()
            })]
        );
        assert_eq!(
            schema.map.get("phones").unwrap(),
            &vec![SchemaValueType::ARRAY(vec![SchemaValueType::PRIMITIVE(
                "STRING".into()
            )])]
        );
    }

    #[test]
    fn test_schema_from_array() {
        let json = serde_json::json!([
            {
                "name": "John Doe",
                "age": 43,
                "address": {
                    "street": "10 Downing Street",
                    "city": "London"
                },
                "phones": [
                    "+44 1234567",
                    "+44 2345678"
                ]
            },
            {
                "name": "Jane Doe",
                "age": "66",
                "address": null,
                "phones": null
            }
        ]);

        let schema = Schema::from_json(&json);

        assert_eq!(schema.name, "root");
        assert_eq!(schema.map.len(), 4);
        assert_eq!(
            schema.map.get("name").unwrap(),
            &vec![SchemaValueType::PRIMITIVE("STRING".into())]
        );
        assert_eq!(
            schema.map.get("age").unwrap(),
            &vec![
                SchemaValueType::PRIMITIVE("NUMBER".into()),
                SchemaValueType::PRIMITIVE("STRING".into())
            ]
        );
        assert_eq!(
            schema.map.get("address").unwrap(),
            &vec![
                SchemaValueType::PRIMITIVE("NULL".into()),
                SchemaValueType::OBJECT(Schema {
                    name: "address".into(),
                    map: vec![
                        (
                            "street".into(),
                            vec![SchemaValueType::PRIMITIVE("STRING".into())]
                        ),
                        (
                            "city".into(),
                            vec![SchemaValueType::PRIMITIVE("STRING".into())]
                        )
                    ]
                    .into_iter()
                    .collect()
                }),
            ]
        );
        assert_eq!(
            schema.map.get("phones").unwrap(),
            &vec![
                SchemaValueType::ARRAY(vec![SchemaValueType::PRIMITIVE("STRING".into())]),
                SchemaValueType::PRIMITIVE("NULL".into()),
            ]
        );
    }
}
