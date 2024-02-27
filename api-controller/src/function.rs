use crate::template::{DOCKERFILE_TEMPLATE, MAIN_TEMPLATE};
use docker_wrapper::{provisioning, runner};
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Function {
    pub name: String,
    pub runtime: String,
    pub content: String,
}

pub fn create_function(name: &str, runtime: &str, function_content: &str) {
    let handler_name = to_camel_case_handler(name);
    let files = create_fn_files(name, runtime).unwrap();
    let mut file = std::io::BufWriter::new(&files[0]);
    file.write_all(
        MAIN_TEMPLATE
            .replace("{{ROUTE}}", name)
            .replace("{{HANDLER}}", &handler_name)
            .as_bytes(),
    )
    .unwrap();
    let mut file = std::io::BufWriter::new(&files[1]);
    file.write_all(function_content.as_bytes()).unwrap();
}

fn to_camel_case_handler(input: &str) -> String {
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

fn create_fn_files(name: &str, _runtime: &str) -> std::io::Result<Vec<File>> {
    let path = Path::new("temp");
    if path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!("Folder '{}' already exists.", name),
        ));
    }

    fs::create_dir(&path)?;

    let path = path.join(name);
    if path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!("Folder '{}' already exists.", name),
        ));
    }

    fs::create_dir(&path)?;

    let main_file_path = path.join("main.go");
    let main_file = File::create(&main_file_path)?;

    let routes_path = path.join("functions");
    fs::create_dir(&routes_path)?;

    let routes_file_path = routes_path.join("routes.go");
    let routes_file = File::create(&routes_file_path)?;

    Ok(vec![main_file, routes_file])
}

pub fn provision_docker(name: &str) {
    let dockerfile_content = DOCKERFILE_TEMPLATE.replace("{{FUNCTION}}", name);
    provisioning(name, &dockerfile_content).unwrap();
    // clean up
    fs::remove_dir_all("temp").unwrap();
    fs::remove_file("Dockerfile").unwrap();
}

pub fn start_function(name: &str) {
    runner(name, "", "8080:8080", 3);
    // 2 seconds sleep
    std::thread::sleep(std::time::Duration::from_secs(1));
}
