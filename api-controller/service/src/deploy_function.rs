use crate::error::Error;
use crate::models::{Config, Function};
use crate::utils::{create_fn_files_base, defer_fn, envs_to_string};
use docker_wrapper::core::provisioning::provisioning;
use entity::function::Model as FunctionModel;
use fn_utils::template::{DOCKERFILE_TEMPLATE, MAIN_TEMPLATE};
use fn_utils::{extract_zip_from_cursor, find_file_in_path, to_camel_case_handler};
use repository::db_repo::FunctionDBRepo;
use sea_orm::DatabaseConnection;
use std::collections::HashMap;
use std::fs;
use std::io::{Cursor, Write};

pub fn create_function(
    name: &str,
    runtime: &str,
    function_content: Vec<u8>,
) -> crate::error::Result<Option<HashMap<String, String>>> {
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

pub fn provision_docker(name: &str, envs: HashMap<String, String>) -> crate::error::Result<()> {
    let mut dockerfile_content = DOCKERFILE_TEMPLATE.replace("{{FUNCTION}}", name);
    dockerfile_content = dockerfile_content.replace("{{ENV}}", &envs_to_string(envs));
    provisioning(name, &dockerfile_content).map_err(|e| Error::SystemError(e.to_string()))?;
    println!("Function docker image built");
    Ok(())
}

pub async fn deploy_function(
    conn: &DatabaseConnection,
    function: Function,
) -> crate::error::Result<String> {
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

    let function_name = name.clone();
    let function = FunctionModel {
        id: 0,
        auth_id: 0,
        name,
        runtime,
    };
    FunctionDBRepo::create_function(conn, function).await;
    _ = temp; // remove lint error
    Ok(format!("Function '{}' deployed successfully", function_name).to_string())
}
