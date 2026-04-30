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
    /// Maximum total cardinality across all variants for a field to qualify
    /// as a discriminator. Above this, the discriminator annotation is skipped.
    #[arg(long, default_value_t = 20)]
    discriminator_max_arms: usize,
    /// Comma-separated priority list of field names to consider when picking
    /// a discriminator. When set, the auto-detection is restricted to these
    /// fields (in the listed order). Useful for forcing a known-good
    /// discriminator when the heuristic doesn't pick one.
    #[arg(long, value_delimiter = ',')]
    discriminator_fields: Vec<String>,
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
    let config = schema::Config {
        merge_objects: args.merge_objects,
        enum_threshold: args.enum_threshold,
        discriminator_max_arms: args.discriminator_max_arms,
        discriminator_fields: args.discriminator_fields.clone(),
    };
    let schema = schema::Schema::from_json(&json, &config);
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
