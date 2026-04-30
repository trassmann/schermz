use super::*;

#[test]
#[should_panic]
fn root_null_panics() {
    Schema::from_json(&serde_json::Value::Null, false);
}

#[test]
#[should_panic]
fn root_string_panics() {
    Schema::from_json(&serde_json::json!("hello"), false);
}

#[test]
#[should_panic]
fn root_number_panics() {
    Schema::from_json(&serde_json::json!(42), false);
}

#[test]
#[should_panic]
fn root_bool_panics() {
    Schema::from_json(&serde_json::json!(true), false);
}

#[test]
fn empty_root_object() {
    let json = serde_json::json!({});
    insta::assert_json_snapshot!(Schema::from_json(&json, false).to_json());
}

#[test]
fn empty_root_array() {
    let json = serde_json::json!([]);
    insta::assert_json_snapshot!(Schema::from_json(&json, false).to_json());
}

#[test]
fn root_array_with_only_primitives_is_empty() {
    // The CLI ignores non-object elements at the root array level.
    let json = serde_json::json!([1, "hello", null, true]);
    insta::assert_json_snapshot!(Schema::from_json(&json, false).to_json());
}

#[test]
fn bool_values() {
    let json = serde_json::json!({ "active": true, "verified": false });
    insta::assert_json_snapshot!(Schema::from_json(&json, false).to_json());
}

#[test]
fn empty_nested_object_and_array() {
    let json = serde_json::json!({
        "empty_obj": {},
        "empty_arr": []
    });
    insta::assert_json_snapshot!(Schema::from_json(&json, false).to_json());
}

#[test]
fn deeply_nested_objects() {
    let json = serde_json::json!({
        "a": { "b": { "c": { "d": { "value": "deep" } } } }
    });
    insta::assert_json_snapshot!(Schema::from_json(&json, false).to_json());
}

#[test]
fn array_of_primitives_only() {
    let json = serde_json::json!({
        "tags": [1, 2, "three", "four", null, true]
    });
    insta::assert_json_snapshot!(Schema::from_json(&json, false).to_json());
}

#[test]
fn nested_arrays_with_strings() {
    // Used to panic ("Invalid value type") because nested arrays sent strings
    // through `to_schema_value_type`. SchemaValueType::from_value_type now
    // handles ValueType::String directly.
    let json = serde_json::json!({
        "matrix": [["a", "bb"], ["ccc"]]
    });
    insta::assert_json_snapshot!(Schema::from_json(&json, false).to_json());
}

#[test]
fn merged_strings_collapse_min_max() {
    let json = serde_json::json!([
        { "name": "ab" },
        { "name": "abcd" },
        { "name": "abc" }
    ]);
    insta::assert_json_snapshot!(Schema::from_json(&json, true).to_json());
}

#[test]
fn unmerged_keeps_distinct_object_shapes() {
    let json = serde_json::json!([
        { "info": { "a": 1 } },
        { "info": { "a": 1, "b": 2 } }
    ]);
    insta::assert_json_snapshot!(Schema::from_json(&json, false).to_json());
}

#[test]
fn merged_collapses_distinct_object_shapes() {
    let json = serde_json::json!([
        { "info": { "a": 1 } },
        { "info": { "a": 1, "b": 2 } }
    ]);
    insta::assert_json_snapshot!(Schema::from_json(&json, true).to_json());
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

// ---------- Feature 1: optionality tracking ----------

#[test]
fn optional_top_level_key_in_array() {
    // `b` is missing from one of three parents -> optional. `a` always present.
    let json = serde_json::json!([
        { "a": 1 },
        { "a": 2, "b": 20 },
        { "a": 3 }
    ]);
    insta::assert_json_snapshot!(Schema::from_json(&json, false).to_json());
}

#[test]
fn single_object_input_never_marks_optional() {
    // Sample size of 1 -> we can't infer optionality.
    let json = serde_json::json!({ "a": 1, "b": 2 });
    insta::assert_json_snapshot!(Schema::from_json(&json, false).to_json());
}

#[test]
fn nested_object_optional_uses_correct_denominator() {
    // outer "addr" is present in 2/3 parents (one entry has no addr at all).
    // Within addr (merged), "x" is in both, "y" only in one.
    let json = serde_json::json!([
        { "addr": { "x": 1, "y": 2 } },
        { "addr": { "x": 1 } },
        { "other": 1 }
    ]);
    insta::assert_json_snapshot!(Schema::from_json(&json, true).to_json());
}

#[test]
fn unmerged_variants_have_no_optional_within() {
    // Without -m each variant shape contains by definition only objects with
    // identical keys, so no key is ever optional within a variant.
    let json = serde_json::json!([
        { "addr": { "x": 1, "y": 2 } },
        { "addr": { "x": 1 } }
    ]);
    insta::assert_json_snapshot!(Schema::from_json(&json, false).to_json());
}

#[test]
fn array_of_objects_optional_field() {
    // The merged shape inside `items[]` should mark `b` as optional because
    // only one of the two array elements (across all parents) had it.
    let json = serde_json::json!({
        "items": [
            { "a": 1 },
            { "a": 2, "b": 20 }
        ]
    });
    insta::assert_json_snapshot!(Schema::from_json(&json, true).to_json());
}

#[test]
fn null_value_counts_as_present() {
    // {x: null} is "x is present" — different from {} (x missing).
    // x: 2/3, y: 1/3 -> both optional, but x is *seen* in 2 parents not 1.
    let json = serde_json::json!([
        { "x": 1 },
        { "x": null },
        { "y": 7 }
    ]);
    insta::assert_json_snapshot!(Schema::from_json(&json, false).to_json());
}

#[test]
fn type_variation_alone_is_not_optionality() {
    // Same key in every parent, just with different scalar types.
    let json = serde_json::json!([
        { "id": 1 },
        { "id": "two" },
        { "id": 3 }
    ]);
    insta::assert_json_snapshot!(Schema::from_json(&json, false).to_json());
}
