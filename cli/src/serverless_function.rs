// use crate::template::ROUTES_TEMPLATE;
use crate::utils::{create_fn_project_file, init_go_mod, GlobalConfig};
use fn_utils::{compress_dir_with_excludes, template::ROUTES_TEMPLATE, to_camel_case_handler};
use reqwest::blocking::{multipart, Client};
use std::fs::File;
use std::io::{Cursor, Read, Write};

pub fn create_new_project(name: &str, runtime: &str) {
    println!("Creating service... '{name}' [RUNTIME:'{runtime}']");
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
    // init go mod
    init_go_mod(name).unwrap();
    println!("Function created");
}

/*
TODO: archive the service and send to a remote server

build the docker image
*/
pub fn deploy_function(name: &str) {
    let mut config_file = File::open("./config.json").unwrap();
    let mut contents = String::new();
    config_file.read_to_string(&mut contents).unwrap();
    let config: GlobalConfig = serde_json::from_str(&contents).unwrap();
    if !config.function_name.contains(&name.to_string()) {
        println!("Function '{}' not found", name);
        return;
    }
    let _runtime = config.runtime;
    println!("Deploying service... '{}'", name);
    let mut dest_zip = Cursor::new(Vec::new());
    match compress_dir_with_excludes(
        std::path::Path::new(name),
        // don't write to disk
        &mut dest_zip,
        &["go.mod", "go.sum"],
    ) {
        Ok(_) => {
            // Reset the cursor to the beginning of the buffer
            dest_zip.set_position(0);

            let form = multipart::Form::new().part(
                "file",
                multipart::Part::reader(dest_zip)
                    .file_name(format!("{name}.zip"))
                    .mime_str("application/zip")
                    .unwrap(),
            );

            let response = Client::new()
                .post("http://localhost:3000/upload")
                .multipart(form)
                .send()
                .unwrap();

            // Check the response
            if response.status().is_success() {
                let response_text = response.text().expect("Failed to read response");
                println!("Response: {}", response_text);
            } else {
                println!(
                    "Failed to upload file: {:?}, {:?}",
                    response.status(),
                    response.text()
                );
            }
        }
        Err(e) => {
            println!("Failed to deploy service: {}", e);
        }
    }
}
