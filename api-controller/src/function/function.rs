use std::fs;
use super::*;
use crate::function::error::Error;
use docker_wrapper::{provisioning, runner};
use error::Result;
use fn_utils::{extract_zip_from_cursor, template::{DOCKERFILE_TEMPLATE, MAIN_TEMPLATE}, to_camel_case_handler};
use function::store::{FunctionAddr, FunctionStore};
use function::utils::{create_fn_files_base, random_port};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::io::Cursor;
use utils::ScopeCall;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Function {
    pub name: String,
    pub runtime: String,
    pub content: Vec<u8>,
}

pub fn create_function(name: &str, runtime: &str, function_content: Vec<u8>) -> Result<()> {
    let handler_name = to_camel_case_handler(name);
    let (path, file) = create_fn_files_base(name, runtime).map_err(|e| Error::SystemError(e.to_string()))?;
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
    extract_zip_from_cursor(buffer, &path).map_err(|e| Error::SystemError(e.to_string()))
}

pub fn provision_docker(name: &str) -> Result<()> {
    let defer = ScopeCall {
        c: Some(|| {
            // clean up
            let _ = fs::remove_dir_all("temp").map_err(|e| Error::SystemError(e.to_string()));
            let _ = fs::remove_file("Dockerfile").map_err(|e| Error::SystemError(e.to_string()));
        }),
    };
    let dockerfile_content = DOCKERFILE_TEMPLATE.replace("{{FUNCTION}}", name);
    provisioning(name, &dockerfile_content).map_err(|e| Error::SystemError(e.to_string()))?;
    println!("Function docker image built");
    Ok(())
}

pub async fn deploy_function(function_store: &FunctionStore, function: Function) -> Result<String> {
    let name = function.name;
    let runtime = function.runtime;
    let content = function.content;
    create_function(&name, &runtime, content)?;
    // build the function docker image
    provision_docker(&name)?;
    function_store.register_function(&name).await;
    Ok(format!("Function '{}' deployed successfully", name).to_string())
}

pub async fn start_function(function_store: &FunctionStore, name: &str) -> Result<String> {
    return match function_store.get_function(name).await {
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
            let timeout = 5;
            match runner(name, &format!("{port}:8080"), timeout) {
                None => {
                    return Err(Error::FunctionFailedToStart(name.to_string()));
                }
                Some(_) => {
                    let function = FunctionAddr {
                        name: name.to_string(),
                        addr: addr.to_string(),
                    };
                    function_store
                        .add_function(function, tokio::time::Duration::from_secs(timeout))
                        .await;
                    println!("Function started at: {}", addr);
                    // 1 seconds sleep
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    Ok(addr)
                }
            }
        }
    };
}
