use clap::Parser;
use std::fs;

mod schema;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    file: String,
}

fn main() {
    let args = Args::parse();
    let data = fs::read_to_string(args.file).expect("Unable to read file");
    let json: serde_json::Value = serde_json::from_str(&data).expect("Invalid JSON");
    let schema: schema::Schema = schema::Schema::from_json(&json);
    let pretty = serde_json::to_string_pretty(&schema.to_json()).unwrap();
    println!("{}", pretty);
}
