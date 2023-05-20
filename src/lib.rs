use std::collections::HashMap;

use neon::prelude::*;
use serde_json::{Map, Value};

#[derive(Debug, PartialEq)]
enum ResultValue {
    Type(FieldType),
    Obj(HashMap<String, ResultValue>),
    Array(Vec<ResultValue>),
}

#[derive(Debug, PartialEq)]
enum FieldType {
    NULL,
    BOOL,
    NUMBER,
    STRING,
}

fn vec_to_array<'a, C: Context<'a>>(vec: &Vec<ResultValue>, cx: &mut C) -> JsResult<'a, JsArray> {
    let arr = JsArray::new(cx, vec.len() as u32);

    for (i, result_val) in vec.iter().enumerate() {
        match result_val {
            ResultValue::Type(t) => {
                let t_str = cx.string(format!("{:?}", t));
                arr.set(cx, i as u32, t_str)?;
            }
            ResultValue::Array(a) => {
                let new_arr = vec_to_array(a, cx)?;
                arr.set(cx, i as u32, new_arr)?;
            }
            ResultValue::Obj(o) => {
                let new_obj = hashmap_to_object(o, cx)?;
                arr.set(cx, i as u32, new_obj)?;
            }
        }
    }

    Ok(arr)
}

fn hashmap_to_object<'a, C: Context<'a>>(
    hash_map: &HashMap<String, ResultValue>,
    cx: &mut C,
) -> JsResult<'a, JsObject> {
    let obj = cx.empty_object();

    for (key, value) in hash_map {
        match value {
            ResultValue::Type(t) => {
                let t_str = cx.string(format!("{:?}", t));
                obj.set(cx, key.as_str(), t_str)?;
            }
            ResultValue::Array(arr) => {
                let t_arr = vec_to_array(arr, cx)?;
                obj.set(cx, key.as_str(), t_arr)?;
            }
            ResultValue::Obj(o) => {
                let t_obj = hashmap_to_object(o, cx)?;
                obj.set(cx, key.as_str(), t_obj)?;
            }
        }
    }

    Ok(obj)
}

fn get_typed_array(input_arr: &Vec<Value>) -> Vec<ResultValue> {
    input_arr
        .iter()
        .map(|json_value| match json_value {
            Value::Null => ResultValue::Type(FieldType::NULL),
            Value::Bool(_) => ResultValue::Type(FieldType::BOOL),
            Value::Number(_) => ResultValue::Type(FieldType::NUMBER),
            Value::String(_) => ResultValue::Type(FieldType::STRING),
            Value::Object(obj) => ResultValue::Obj(get_typed_map(obj)),
            Value::Array(arr) => ResultValue::Array(get_typed_array(arr)),
        })
        .collect()
}

fn get_typed_map(input_obj: &Map<String, Value>) -> HashMap<String, ResultValue> {
    let mut result = HashMap::<String, ResultValue>::new();
    for (key, value) in input_obj {
        match value {
            Value::Null => {
                result.insert(key.to_string(), ResultValue::Type(FieldType::NULL));
            }
            Value::Bool(_) => {
                result.insert(key.to_string(), ResultValue::Type(FieldType::BOOL));
            }
            Value::Number(_) => {
                result.insert(key.to_string(), ResultValue::Type(FieldType::NUMBER));
            }
            Value::String(_) => {
                result.insert(key.to_string(), ResultValue::Type(FieldType::STRING));
            }
            Value::Object(obj) => {
                result.insert(key.to_string(), ResultValue::Obj(get_typed_map(obj)));
            }
            Value::Array(arr) => {
                result.insert(key.to_string(), ResultValue::Array(get_typed_array(arr)));
            }
        }
    }

    result
}

fn create_typed_object(mut cx: FunctionContext) -> JsResult<JsObject> {
    let data_handle = cx.argument::<JsString>(0)?;
    let data_raw = data_handle.value(&mut cx);

    let json: Value = serde_json::from_str(&data_raw).unwrap();
    let mut result_map = HashMap::<String, Vec<ResultValue>>::new();

    for obj in json.as_array().unwrap() {
        let typed_map = get_typed_map(obj.as_object().unwrap());
        for (key, value) in typed_map {
            match result_map.get_mut(&key) {
                Some(types) => {
                    if !types.contains(&value) {
                        types.push(value);
                    }
                }
                None => {
                    result_map.insert(key.to_string(), vec![value]);
                }
            }
        }
    }

    let result_json = cx.empty_object();

    for (key, value) in result_map {
        let result_arr = vec_to_array(&value, &mut cx)?;
        result_json.set(&mut cx, key.as_str(), result_arr)?;
    }

    Ok(result_json)
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("create_typed_object", create_typed_object)?;
    Ok(())
}
