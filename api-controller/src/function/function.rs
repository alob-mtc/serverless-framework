use self::utils::defer_fn;

use super::*;
use crate::function::error::Error;
use docker_wrapper::core::{provisioning::provisioning, runner::runner};
use error::Result;
use fn_utils::{
    extract_zip_from_cursor, find_file_in_path,
    template::{DOCKERFILE_TEMPLATE, MAIN_TEMPLATE},
    to_camel_case_handler,
};
use function::utils::{create_fn_files_base, random_port};
use function::{
    store::{FunctionAddr, FunctionStore},
    utils::envs_to_string,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::time::Duration;
use std::{collections::HashMap, io::Cursor};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Function {
    pub name: String,
    pub runtime: String,
    pub content: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    function_name: String,
    runtime: String,
    env: Option<HashMap<String, String>>,
}

pub fn create_function(
    name: &str,
    runtime: &str,
    function_content: Vec<u8>,
) -> Result<Option<HashMap<String, String>>> {
    let handler_name = to_camel_case_handler(name);
    let (path, file) =
        create_fn_files_base(name, runtime).map_err(|e| Error::SystemError(e.to_string()))?;
    let mut file = std::io::BufWriter::new(file);
    file.write_all(
        MAIN_TEMPLATE
            .replace("{{ROUTE}}", name)
            .replace("{{HANDLER}}", &handler_name)
            .as_bytes(),
    )
    .map_err(|e| Error::SystemError(e.to_string()))?;
    let buffer = Cursor::new(function_content);
    // extract the zip file from in-memory buffer
    extract_zip_from_cursor(buffer, &path).map_err(|e| Error::SystemError(e.to_string()))?;
    let config_file = find_file_in_path("config.json", path).ok_or(Error::BadFunction(
        "function does not include config file".to_string(),
    ))?;
    let config = fs::read_to_string(config_file).map_err(|e| Error::SystemError(e.to_string()))?;
    let mut config: Config =
        serde_json::from_str(&config).map_err(|e| Error::SystemError(e.to_string()))?;
    Ok(config.env.take())
}

pub fn provision_docker(name: &str, envs: HashMap<String, String>) -> Result<()> {
    let mut dockerfile_content = DOCKERFILE_TEMPLATE.replace("{{FUNCTION}}", name);
    dockerfile_content = dockerfile_content.replace("{{ENV}}", &envs_to_string(envs));
    provisioning(name, &dockerfile_content).map_err(|e| Error::SystemError(e.to_string()))?;
    println!("Function docker image built");
    Ok(())
}

pub async fn deploy_function(function_store: &FunctionStore, function: Function) -> Result<String> {
    let temp = defer_fn(|| {
        // clean up
        let _ = fs::remove_dir_all("temp").map_err(|e| Error::SystemError(e.to_string()));
        let _ = fs::remove_file("Dockerfile").map_err(|e| Error::SystemError(e.to_string()));
    });
    let name = function.name;
    let runtime = function.runtime;
    let content = function.content;
    let envs = create_function(&name, &runtime, content)?.unwrap();
    // build the function docker image
    provision_docker(&name, envs)?;
    function_store.register_function(&name).await;
    _ = temp; // remove lint error
    Ok(format!("Function '{}' deployed successfully", name).to_string())
}

pub async fn start_function(function_store: &FunctionStore, name: &str) -> Result<String> {
    match function_store.get_function(name).await {
        Some(addr) => {
            println!("Function already running at: {}", addr);
            Ok(addr)
        }
        None => {
            if !function_store.function_exists(name).await {
                return Err(Error::FunctionNotRegistered(name.to_string()));
            }
            let port = random_port();
            let addr = format!("localhost:{}", port);
            let timeout = 10;
            match runner(
                name,
                &format!("{port}:8080"),
                Some(Duration::from_secs(timeout)),
            )
            .await
            {
                Err(e) => Err(Error::FunctionFailedToStart(name.to_string())),
                Ok(_) => {
                    let function = FunctionAddr {
                        name: name.to_string(),
                        addr: addr.to_string(),
                    };
                    function_store
                        .add_function(function, Duration::from_secs(timeout))
                        .await;
                    println!("Function started at: {addr}");
                    Ok(addr)
                }
            }
        }
    }
}
