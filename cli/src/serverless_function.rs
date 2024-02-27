use crate::template::ROUTES_TEMPLATE;
use crate::utils::{create_fn_project_file, to_camel_case_handler, Config};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::{Read, Write};

pub fn create_new_project(name: &str, runtime: &str) {
    println!("Creating function... '{name}' [RUNTIME:'{runtime}']");
    let handler_name = to_camel_case_handler(name);
    let file = create_fn_project_file(name, runtime).unwrap();
    let mut file = std::io::BufWriter::new(&file);
    file.write_all(
        ROUTES_TEMPLATE
            .replace("{{ROUTE}}", name)
            .replace("{{HANDLER}}", &handler_name)
            .as_bytes(),
    )
    .unwrap();
    println!("Function created");
}

/*
TODO: archive the function and send to a remote server

build the docker image
*/
pub fn deploy_function() {
    let mut config_file = File::open("./config.json").unwrap();
    let mut contents = String::new();
    config_file.read_to_string(&mut contents).unwrap();
    let config: Config = serde_json::from_str(&contents).unwrap();
    let name = config.function_name;
    let runtime = config.runtime;
    println!("Deploying function... '{}'", name);
    let file = File::open(format!("{name}/function.go")).unwrap();
    let mut file = std::io::BufReader::new(file);
    let mut function_content = String::new();
    file.read_to_string(&mut function_content).unwrap();
    let function = Function {
        name,
        runtime,
        content: function_content,
    };

    // make a request to the server /upload
    let response = Client::new()
        .post("http://localhost:3000/upload")
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&function).unwrap())
        .send()
        .unwrap();

    // Check the response
    if response.status().is_success() {
        let response_text = response.text().expect("Failed to read response");
        println!("Response: {}", response_text);
    } else {
        println!("Failed to upload file: {:?}", response.status());
    }
}

#[derive(Serialize, Deserialize)]
struct Function {
    name: String,
    runtime: String,
    content: String,
}
