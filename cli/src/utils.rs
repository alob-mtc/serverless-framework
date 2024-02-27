use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub function_name: String,
    pub runtime: String,
}

pub fn to_camel_case_handler(input: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;

    for (i, c) in input.chars().enumerate() {
        if c == '-' {
            capitalize_next = true;
        } else if capitalize_next || i == 0 {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result.push_str("Handler");
    result
}

pub fn create_fn_project_file(name: &str, runtime: &str) -> std::io::Result<File> {
    create_config_file(name, runtime)?;

    let path = Path::new(name);
    if path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!("Folder '{}' already exists.", name),
        ));
    }

    fs::create_dir(&path)?;

    let routes_file_path = path.join("function.go");
    let routes_file = File::create(&routes_file_path)?;

    Ok(routes_file)
}

fn create_config_file(name: &str, runtime: &str) -> std::io::Result<()> {
    let mut config_file = File::create("./config.json")?;
    let config = Config {
        function_name: name.to_string(),
        runtime: runtime.to_string(),
    };
    let serialized = serde_json::to_string(&config)?;
    println!("config: {}", serialized);
    config_file.write_all(serialized.as_bytes())
}
