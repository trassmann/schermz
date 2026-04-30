use super::*;

fn default_config() -> Config {
    Config::default()
}

fn merged_config() -> Config {
    Config {
        merge_objects: true,
        ..Config::default()
    }
}

fn config_with_enum_threshold(threshold: usize) -> Config {
    Config {
        enum_threshold: threshold,
        ..Config::default()
    }
}

#[test]
#[should_panic]
fn root_null_panics() {
    Schema::from_json(&serde_json::Value::Null, &default_config());
}

#[test]
#[should_panic]
fn root_string_panics() {
    Schema::from_json(&serde_json::json!("hello"), &default_config());
}

#[test]
#[should_panic]
fn root_number_panics() {
    Schema::from_json(&serde_json::json!(42), &default_config());
}

#[test]
#[should_panic]
fn root_bool_panics() {
    Schema::from_json(&serde_json::json!(true), &default_config());
}

#[test]
fn empty_root_object() {
    let json = serde_json::json!({});
    insta::assert_json_snapshot!(Schema::from_json(&json, &default_config()).to_json());
}

#[test]
fn empty_root_array() {
    let json = serde_json::json!([]);
    insta::assert_json_snapshot!(Schema::from_json(&json, &default_config()).to_json());
}

#[test]
fn root_array_with_only_primitives_is_empty() {
    // The CLI ignores non-object elements at the root array level.
    let json = serde_json::json!([1, "hello", null, true]);
    insta::assert_json_snapshot!(Schema::from_json(&json, &default_config()).to_json());
}

#[test]
fn bool_values() {
    let json = serde_json::json!({ "active": true, "verified": false });
    insta::assert_json_snapshot!(Schema::from_json(&json, &default_config()).to_json());
}

#[test]
fn empty_nested_object_and_array() {
    let json = serde_json::json!({
        "empty_obj": {},
        "empty_arr": []
    });
    insta::assert_json_snapshot!(Schema::from_json(&json, &default_config()).to_json());
}

#[test]
fn deeply_nested_objects() {
    let json = serde_json::json!({
        "a": { "b": { "c": { "d": { "value": "deep" } } } }
    });
    insta::assert_json_snapshot!(Schema::from_json(&json, &default_config()).to_json());
}

#[test]
fn array_of_primitives_only() {
    let json = serde_json::json!({
        "tags": [1, 2, "three", "four", null, true]
    });
    insta::assert_json_snapshot!(Schema::from_json(&json, &default_config()).to_json());
}

#[test]
fn nested_arrays_with_strings() {
    // Used to panic ("Invalid value type") because nested arrays sent strings
    // through `to_schema_value_type`. SchemaValueType::from_value_type now
    // handles ValueType::String directly.
    let json = serde_json::json!({
        "matrix": [["a", "bb"], ["ccc"]]
    });
    insta::assert_json_snapshot!(Schema::from_json(&json, &default_config()).to_json());
}

#[test]
fn merged_strings_collapse_min_max() {
    let json = serde_json::json!([
        { "name": "ab" },
        { "name": "abcd" },
        { "name": "abc" }
    ]);
    insta::assert_json_snapshot!(Schema::from_json(&json, &merged_config()).to_json());
}

#[test]
fn unmerged_keeps_distinct_object_shapes() {
    let json = serde_json::json!([
        { "info": { "a": 1 } },
        { "info": { "a": 1, "b": 2 } }
    ]);
    insta::assert_json_snapshot!(Schema::from_json(&json, &default_config()).to_json());
}

#[test]
fn merged_collapses_distinct_object_shapes() {
    let json = serde_json::json!([
        { "info": { "a": 1 } },
        { "info": { "a": 1, "b": 2 } }
    ]);
    insta::assert_json_snapshot!(Schema::from_json(&json, &merged_config()).to_json());
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

    insta::assert_json_snapshot!(Schema::from_json(&json, &merged_config()).to_json());
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

    insta::assert_json_snapshot!(Schema::from_json(&json, &merged_config()).to_json());
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

    insta::assert_json_snapshot!(Schema::from_json(&json, &default_config()).to_json());
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
    insta::assert_json_snapshot!(Schema::from_json(&json, &default_config()).to_json());
}

#[test]
fn single_object_input_never_marks_optional() {
    // Sample size of 1 -> we can't infer optionality.
    let json = serde_json::json!({ "a": 1, "b": 2 });
    insta::assert_json_snapshot!(Schema::from_json(&json, &default_config()).to_json());
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
    insta::assert_json_snapshot!(Schema::from_json(&json, &merged_config()).to_json());
}

#[test]
fn unmerged_variants_have_no_optional_within() {
    // Without -m each variant shape contains by definition only objects with
    // identical keys, so no key is ever optional within a variant.
    let json = serde_json::json!([
        { "addr": { "x": 1, "y": 2 } },
        { "addr": { "x": 1 } }
    ]);
    insta::assert_json_snapshot!(Schema::from_json(&json, &default_config()).to_json());
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
    insta::assert_json_snapshot!(Schema::from_json(&json, &merged_config()).to_json());
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
    insta::assert_json_snapshot!(Schema::from_json(&json, &default_config()).to_json());
}

#[test]
fn type_variation_alone_is_not_optionality() {
    // Same key in every parent, just with different scalar types.
    let json = serde_json::json!([
        { "id": 1 },
        { "id": "two" },
        { "id": 3 }
    ]);
    insta::assert_json_snapshot!(Schema::from_json(&json, &default_config()).to_json());
}

// ---------- Feature 2: enum value extraction ----------

#[test]
fn string_enum_two_distinct_values_emitted_sorted() {
    let json = serde_json::json!([
        { "status": "in_force" },
        { "status": "opened" },
        { "status": "in_force" }
    ]);
    insta::assert_json_snapshot!(Schema::from_json(&json, &default_config()).to_json());
}

#[test]
fn string_enum_over_threshold_skipped() {
    // 31 distinct values, default threshold is 30 -> no `values` emitted.
    let mut items = Vec::new();
    for i in 0..31 {
        items.push(serde_json::json!({ "k": format!("v{i}") }));
    }
    let json = serde_json::Value::Array(items);
    insta::assert_json_snapshot!(Schema::from_json(&json, &default_config()).to_json());
}

#[test]
fn custom_threshold_admits_more_values() {
    // Same 31 distinct values, threshold raised to 100 -> `values` is emitted.
    let mut items = Vec::new();
    for i in 0..31 {
        items.push(serde_json::json!({ "k": format!("v{i}") }));
    }
    let json = serde_json::Value::Array(items);
    insta::assert_json_snapshot!(
        Schema::from_json(&json, &config_with_enum_threshold(100)).to_json()
    );
}

#[test]
fn threshold_zero_disables_values_entirely() {
    let json = serde_json::json!([{ "k": "a" }, { "k": "b" }]);
    insta::assert_json_snapshot!(
        Schema::from_json(&json, &config_with_enum_threshold(0)).to_json()
    );
}

#[test]
fn mixed_string_and_number_skips_values() {
    // Single key sometimes a string, sometimes a number -> no `values`.
    let json = serde_json::json!([
        { "id": "a" },
        { "id": 1 },
        { "id": "b" }
    ]);
    insta::assert_json_snapshot!(Schema::from_json(&json, &default_config()).to_json());
}

#[test]
fn long_strings_skip_values_even_below_threshold() {
    // Two distinct values, but one is over MAX_ENUM_STRING_LEN (200 chars).
    let big = "x".repeat(250);
    let json = serde_json::json!([
        { "blob": "small" },
        { "blob": big }
    ]);
    insta::assert_json_snapshot!(Schema::from_json(&json, &default_config()).to_json());
}

#[test]
fn integer_only_numbers_emit_values_sorted_numerically() {
    let json = serde_json::json!([
        { "code": 8 },
        { "code": 1 },
        { "code": 5 },
        { "code": 2 },
        { "code": 3 }
    ]);
    insta::assert_json_snapshot!(Schema::from_json(&json, &default_config()).to_json());
}

#[test]
fn float_numbers_skip_values() {
    // Any float disqualifies the whole set -- not useful as enum members.
    let json = serde_json::json!([
        { "price": 1 },
        { "price": 2.5 },
        { "price": 3 }
    ]);
    insta::assert_json_snapshot!(Schema::from_json(&json, &default_config()).to_json());
}

#[test]
fn bool_only_field_skips_values() {
    // Booleans are trivially enumerable from the type alone; spec recommends
    // skipping them to avoid noise.
    let json = serde_json::json!([
        { "active": true },
        { "active": false },
        { "active": true }
    ]);
    insta::assert_json_snapshot!(Schema::from_json(&json, &default_config()).to_json());
}

#[test]
fn string_with_null_still_emits_values() {
    // STRING + NULL is not "mixed" for the purpose of values -- null is the
    // absence of a value, not a competing scalar type.
    let json = serde_json::json!([
        { "tag": "a" },
        { "tag": null },
        { "tag": "b" }
    ]);
    insta::assert_json_snapshot!(Schema::from_json(&json, &default_config()).to_json());
}

#[test]
fn string_with_object_skips_values() {
    // Mixing a scalar with a non-scalar variant disqualifies the key.
    let json = serde_json::json!([
        { "thing": "hello" },
        { "thing": { "nested": 1 } }
    ]);
    insta::assert_json_snapshot!(Schema::from_json(&json, &default_config()).to_json());
}

// ---------- Feature 3: discriminator detection ----------

#[test]
fn discriminator_detected_for_two_disjoint_variants() {
    // Classic tagged union: status partitions the two shapes cleanly. Premium
    // is shared (100 in both variants) so it can't be a discriminator and
    // status is the only qualifying field.
    let json = serde_json::json!({
        "items": [
            { "status": "opened", "premium": 100 },
            { "status": "in_force", "premium": 100, "paidDate": "2026-01-01" }
        ]
    });
    insta::assert_json_snapshot!(Schema::from_json(&json, &default_config()).to_json());
}

#[test]
fn discriminator_skipped_when_values_overlap() {
    // status="open" appears in both variants -> disjointness fails.
    let json = serde_json::json!({
        "items": [
            { "status": "open", "a": 1 },
            { "status": "open", "b": 2 },
            { "status": "closed", "a": 3 }
        ]
    });
    insta::assert_json_snapshot!(Schema::from_json(&json, &default_config()).to_json());
}

#[test]
fn discriminator_picks_smaller_cardinality_then_alphabetical() {
    // Both `kind` and `tag` discriminate cleanly. `kind` has total cardinality 3
    // (1+1+1), `tag` has 6 (2+2+2) -> pick `kind`. Then if both had cardinality
    // 3, alphabetical tie-break would pick `kind` over `tag` anyway.
    let json = serde_json::json!({
        "items": [
            { "kind": "a", "tag": "x1", "extra": 1 },
            { "kind": "a", "tag": "x2", "extra": 1 },
            { "kind": "b", "tag": "y1", "other": 1 },
            { "kind": "b", "tag": "y2", "other": 1 },
            { "kind": "c", "tag": "z1", "more": 1 },
            { "kind": "c", "tag": "z2", "more": 1 }
        ]
    });
    insta::assert_json_snapshot!(Schema::from_json(&json, &default_config()).to_json());
}

#[test]
fn discriminator_skipped_when_above_max_arms_cap() {
    // 21 distinct id values across 21 variants -> over the default cap of 20.
    let mut items = Vec::new();
    for i in 0..21 {
        items.push(serde_json::json!({
            "id": format!("v{i}"),
            // make each variant structurally distinct so they don't merge
            format!("only_in_v{i}"): true
        }));
    }
    let json = serde_json::json!({ "items": items });
    insta::assert_json_snapshot!(Schema::from_json(&json, &default_config()).to_json());
}

#[test]
fn discriminator_max_arms_raised_admits_more_variants() {
    // Same 21 variants, max-arms raised to 50 -> discriminator emitted.
    let mut items = Vec::new();
    for i in 0..21 {
        items.push(serde_json::json!({
            "id": format!("v{i}"),
            format!("only_in_v{i}"): true
        }));
    }
    let json = serde_json::json!({ "items": items });
    let config = Config {
        discriminator_max_arms: 50,
        ..Config::default()
    };
    insta::assert_json_snapshot!(Schema::from_json(&json, &config).to_json());
}

#[test]
fn discriminator_skipped_for_single_variant() {
    // One Object variant, no union to discriminate.
    let json = serde_json::json!({
        "items": [
            { "status": "ok", "v": 1 },
            { "status": "ok", "v": 2 }
        ]
    });
    insta::assert_json_snapshot!(Schema::from_json(&json, &default_config()).to_json());
}

#[test]
fn discriminator_skipped_with_merge_objects() {
    // -m collapses to a single shape, so there's nothing to discriminate.
    let json = serde_json::json!({
        "items": [
            { "status": "opened", "a": 1 },
            { "status": "in_force", "b": 2 }
        ]
    });
    insta::assert_json_snapshot!(Schema::from_json(&json, &merged_config()).to_json());
}

#[test]
fn discriminator_skipped_for_field_not_in_every_variant() {
    // `tag` is in variant 1 only -> not a candidate. `status` is in both and
    // disjoint, so it wins.
    let json = serde_json::json!({
        "items": [
            { "status": "opened", "tag": "alpha", "v": 1 },
            { "status": "in_force", "v": 2 }
        ]
    });
    insta::assert_json_snapshot!(Schema::from_json(&json, &default_config()).to_json());
}

#[test]
fn discriminator_fields_override_picks_named_field() {
    // Auto-pick would be `kind` alphabetically; user override forces `tag`.
    // Each variant has a distinct extra key so the variants don't collapse
    // into one shape.
    let json = serde_json::json!({
        "items": [
            { "kind": "a", "tag": "x", "extra1": 1 },
            { "kind": "b", "tag": "y", "extra2": 2 }
        ]
    });
    let config = Config {
        discriminator_fields: vec!["tag".into()],
        ..Config::default()
    };
    insta::assert_json_snapshot!(Schema::from_json(&json, &config).to_json());
}

#[test]
fn discriminator_fields_priority_list_picks_first_match() {
    // User lists [foo, kind]. foo doesn't exist; kind qualifies -> kind wins.
    let json = serde_json::json!({
        "items": [
            { "kind": "a", "extra1": 1 },
            { "kind": "b", "extra2": 2 }
        ]
    });
    let config = Config {
        discriminator_fields: vec!["foo".into(), "kind".into()],
        ..Config::default()
    };
    insta::assert_json_snapshot!(Schema::from_json(&json, &config).to_json());
}

#[test]
fn discriminator_fields_no_match_emits_nothing() {
    // User restricts to fields that don't qualify -> no discriminator at all,
    // even if other fields would have qualified for auto-detection.
    let json = serde_json::json!({
        "items": [
            { "status": "opened", "extra1": 1 },
            { "status": "in_force", "extra2": 2 }
        ]
    });
    let config = Config {
        discriminator_fields: vec!["nonexistent".into()],
        ..Config::default()
    };
    insta::assert_json_snapshot!(Schema::from_json(&json, &config).to_json());
}

#[test]
fn discriminator_lifeware_style_5_variants() {
    // Approximates Lifeware's additionalPayments shape: 5 variants discriminated
    // by `status` alone (the spec mentions a composite status+invested case;
    // this single-field version is what the v1 algorithm handles).
    let json = serde_json::json!({
        "additionalPayments": [
            { "status": "applied", "amount": 100 },
            { "status": "opened", "amount": 200, "openedAt": "2026-01-01" },
            { "status": "in_force", "amount": 200, "paidDate": "2026-01-10", "openedAt": "2026-01-01" },
            { "status": "revoked", "amount": 0, "revokedAt": "2026-01-15" },
            { "status": "rejected", "amount": 0, "reason": "kyc" }
        ]
    });
    insta::assert_json_snapshot!(Schema::from_json(&json, &default_config()).to_json());
}
