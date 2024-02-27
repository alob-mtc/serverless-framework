use std::fs;
use docker_wrapper::provisioning;
use crate::template::{DOCKERFILE_TEMPLATE, MAIN_TEMPLATE, ROUTES_TEMPLATE};
use crate::utils::{create_fn_files, create_fn_project_file, to_camel_case_handler, Config};
use std::fs::{File};
use std::io::{Read, Write};

pub fn create_new_project(name: &str, runtime: &str) {
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
    let file = File::open(format!("{name}/function.go")).unwrap();
    let mut file = std::io::BufReader::new(file);
    let mut function_content = String::new();
    file.read_to_string(&mut function_content).unwrap();
    println!("{}", contents);
    create_function(&name, &runtime, &function_content);
    // TODO: build the docker image
    let dockerfile_content = DOCKERFILE_TEMPLATE.replace("{{FUNCTION}}", &name);
    provisioning(&name, &dockerfile_content).unwrap();
    // clean up
    fs::remove_dir_all("temp").unwrap();
    fs::remove_file("Dockerfile").unwrap();
}
