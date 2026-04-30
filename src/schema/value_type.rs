use serde_json::{Number, Value as JsonValue};

#[derive(Debug, Clone)]
pub(super) enum ValueType {
    Null,
    Bool,
    Number(Number),
    String { len: usize, value: String },
    Object(SchemaObject),
    Array(Vec<ValueType>),
}

#[derive(Debug, Clone)]
pub(super) struct SchemaObjectKey {
    pub id: String,
    pub v_type: ValueType,
}

#[derive(Debug, Clone)]
pub(super) struct SchemaObject {
    pub keys: Vec<SchemaObjectKey>,
}

impl ValueType {
    pub fn from_json(json: &JsonValue) -> Self {
        match json {
            JsonValue::Null => Self::Null,
            JsonValue::Bool(_) => Self::Bool,
            JsonValue::Number(n) => Self::Number(n.clone()),
            JsonValue::String(s) => Self::String {
                len: s.len(),
                value: s.clone(),
            },
            JsonValue::Object(_) => Self::Object(SchemaObject::from_json(json)),
            JsonValue::Array(arr) => {
                let values = arr.iter().map(Self::from_json).collect();
                Self::Array(values)
            }
        }
    }
}

impl SchemaObject {
    pub fn from_json(json: &JsonValue) -> Self {
        let keys = json
            .as_object()
            .expect("SchemaObject::from_json called on non-object JSON")
            .iter()
            .map(|(key, value)| SchemaObjectKey {
                id: key.clone(),
                v_type: ValueType::from_json(value),
            })
            .collect();
        Self { keys }
    }
}
