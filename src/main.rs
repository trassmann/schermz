use clap::{ArgAction, Parser};
use std::process::ExitCode;
use std::{fs, io};

mod schema;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the JSON file
    #[arg(short, long)]
    file: String,
    /// Whether to merge object types into one
    #[arg(short, long, action = ArgAction::SetTrue)]
    merge_objects: bool,
    /// When a scalar field has at most N distinct observed values, emit them
    /// as an enum-style "values" annotation. Pass 0 to disable.
    #[arg(long, default_value_t = 30)]
    enum_threshold: usize,
}

fn run(args: &Args) -> Result<String, String> {
    let data = fs::read_to_string(&args.file).map_err(|err| match err.kind() {
        io::ErrorKind::NotFound => format!("file not found: {}", args.file),
        _ => format!("could not read {}: {err}", args.file),
    })?;
    let json: serde_json::Value = serde_json::from_str(&data)
        .map_err(|err| format!("{} is not valid JSON: {err}", args.file))?;
    if !json.is_object() && !json.is_array() {
        return Err(format!(
            "{} must contain a JSON object or array at the root",
            args.file
        ));
    }
    let schema = schema::Schema::from_json(&json, args.merge_objects, args.enum_threshold);
    serde_json::to_string_pretty(&schema.to_json())
        .map_err(|err| format!("failed to serialize schema: {err}"))
}

fn main() -> ExitCode {
    let args = Args::parse();
    match run(&args) {
        Ok(pretty) => {
            println!("{pretty}");
            ExitCode::SUCCESS
        }
        Err(msg) => {
            eprintln!("schermz: {msg}");
            ExitCode::FAILURE
        }
    }
}
