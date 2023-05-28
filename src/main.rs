use std::fs;

mod schema;

fn main() {
    let data = fs::read_to_string("./test_data.json").expect("Unable to read file");

    let json: serde_json::Value =
        serde_json::from_str(&data).expect("JSON does not have correct format.");
    let schema: schema::Schema = schema::Schema::from_json(&json);

    dbg!(schema.map);
}
