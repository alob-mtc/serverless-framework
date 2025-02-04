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
use std::io::{Cursor, Read, Write};
use std::path::PathBuf;

pub async fn create_function(
    name: &str,
    runtime: &str,
    function_content: Vec<u8>,
) -> crate::error::Result<(Option<HashMap<String, String>>, PathBuf)> {
    let temp_dir = tempfile::tempdir()
        .map_err(|e| Error::SystemError(format!("Failed to create temp dir: {e}")))?
        .into_path()
        .join(name);

    let handler_name = to_camel_case_handler(name);
    let file = create_fn_files_base(&temp_dir, name, runtime)
        .map_err(|e| Error::SystemError(e.to_string()))?;
    // a drop of the buf-writer will force a flush to disk
    let mut file_writer = std::io::BufWriter::new(file);
    file_writer
        .write_all(
            MAIN_TEMPLATE
                .replace("{{ROUTE}}", name)
                .replace("{{HANDLER}}", &handler_name)
                .as_bytes(),
        )
        .map_err(|e| Error::SystemError(e.to_string()))?;
    file_writer
        .flush()
        .map_err(|e| Error::SystemError(e.to_string()))?;

    let buffer = Cursor::new(function_content);
    // extract the zip file from in-memory buffer
    extract_zip_from_cursor(buffer, &temp_dir).map_err(|e| Error::SystemError(e.to_string()))?;
    let config_file = find_file_in_path("config.json", &temp_dir).ok_or(Error::BadFunction(
        "function does not include config file".to_string(),
    ))?;
    let config = fs::read_to_string(config_file).map_err(|e| Error::SystemError(e.to_string()))?;
    let mut config: Config =
        serde_json::from_str(&config).map_err(|e| Error::SystemError(e.to_string()))?;
    Ok((config.env.take(), temp_dir))
}

pub async fn provision_docker(
    path: PathBuf,
    name: &str,
    envs: HashMap<String, String>,
) -> crate::error::Result<()> {
    let mut dockerfile_content = DOCKERFILE_TEMPLATE.replace("{{FUNCTION}}", name);
    dockerfile_content = dockerfile_content.replace("{{ENV}}", &envs_to_string(envs));
    provisioning(&path, name, &dockerfile_content)
        .await
        .map_err(|e| Error::SystemError(e.to_string()))?;
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
    let (envs, path) = create_function(&name, &runtime, content).await?;
    // build the function docker image
    provision_docker(path, &name, envs.unwrap()).await?;

    let function_name = name.clone();
    match FunctionDBRepo::find_function_by_name(conn, &name).await {
        None => {
            let function = FunctionModel {
                id: 0,
                auth_id: 0,
                name,
                runtime,
            };
            FunctionDBRepo::create_function(conn, function).await;
        }
        Some(_) => {}
    }

    _ = temp; // remove lint error
    Ok(format!("Function '{}' deployed successfully", function_name).to_string())
}
