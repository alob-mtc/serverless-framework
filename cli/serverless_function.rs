use std::fs::File;
use crate::template::{MAIN_TEMPLATE, ROUTES_TEMPLATE};
use crate::utils::{Config, create_fn_files, create_fn_project_file, to_camel_case_handler};
use std::io::{Read, Write};

pub fn create_new_project(name: &str, runtime: &str) {
    let handler_name = to_camel_case_handler(name);
    let file = create_fn_project_file(name, runtime).unwrap();
    let mut file = std::io::BufWriter::new(&file);
    file.write_all(ROUTES_TEMPLATE
        .replace("{{ROUTE}}", name)
        .replace("{{HANDLER}}", &handler_name).as_bytes(),
    ).unwrap();
}

pub fn create_function(name: &str, runtime: &str) {
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
    file.write_all(
        ROUTES_TEMPLATE
            .replace("{{ROUTE}}", name)
            .replace("{{HANDLER}}", &handler_name)
            .as_bytes(),
    )
    .unwrap();
}

/*
TODO: archive the function and send to a remote server

build the docker image
*/
pub fn deploy_function() {
    let mut config_file = File::open("config.json").unwrap();
    let mut contents = String::new();
    config_file.read_to_string(&mut contents).unwrap();
    let config: Config = serde_json::from_str(&contents).unwrap();
    let name = config.function_name;
    let file = File::open(format!("{}/function.go", name)).unwrap();
    let mut file = std::io::BufReader::new(file);
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    println!("{}", contents);
}
