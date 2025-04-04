use crate::error::Error;
use crate::models::{Config, Function};
use crate::utils::{create_fn_files_base, envs_to_string};
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
use tracing::{error, info};

/// Creates a function file structure and extracts its configuration.
///
/// This function performs the following steps:
/// 1. Creates a temporary directory for the function based on its name.
/// 2. Creates the base function file (using a main template) and writes it to disk.
/// 3. Extracts the provided ZIP content into the temporary directory.
/// 4. Searches for and parses a `config.json` file within the extracted files.
///
/// # Arguments
///
/// * `name` - The name of the function.
/// * `runtime` - The runtime used by the function (e.g. "go").
/// * `function_content` - The zipped function content.
///
/// # Returns
///
/// A tuple containing:
/// - An optional map of environment variables extracted from the configuration.
/// - The path to the function files.
async fn create_function(
    name: &str,
    runtime: &str,
    function_content: Vec<u8>,
) -> crate::error::Result<(Option<HashMap<String, String>>, PathBuf)> {
    // Create a temporary directory for this function.
    let temp_dir = tempfile::tempdir()
        .map_err(|e| Error::SystemError(format!("Failed to create temp dir: {e}")))?
        .into_path()
        .join(name);

    // Convert function name into a CamelCase handler name.
    let handler_name = to_camel_case_handler(name);

    // Create the base function file (e.g., main.go) using the provided template.
    let file = create_fn_files_base(&temp_dir, name, runtime)
        .map_err(|e| Error::SystemError(e.to_string()))?;
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

    // Extract the function ZIP content from an in-memory buffer.
    let buffer = Cursor::new(function_content);
    extract_zip_from_cursor(buffer, &temp_dir).map_err(|e| Error::SystemError(e.to_string()))?;

    // Locate and read the configuration file.
    let config_file = find_file_in_path("config.json", &temp_dir).ok_or(Error::BadFunction(
        "Function does not include config file".to_string(),
    ))?;
    let config_content =
        fs::read_to_string(config_file).map_err(|e| Error::SystemError(e.to_string()))?;
    let mut config: Config =
        serde_json::from_str(&config_content).map_err(|e| Error::SystemError(e.to_string()))?;

    Ok((config.env.take(), temp_dir))
}

/// Provisions a Docker container for the function using the provided configuration.
///
/// This function generates a Dockerfile by replacing placeholders in the template
/// with the function's name and its environment variables, and then calls the provisioning
/// routine to build the Docker image.
///
/// # Arguments
///
/// * `path` - The file path to the function files.
/// * `name` - The function's name.
/// * `envs` - A map of environment variables for the function.
///
/// # Returns
///
/// A result indicating success or failure.
async fn provision_docker(
    path: PathBuf,
    name: &str,
    envs: HashMap<String, String>,
) -> crate::error::Result<()> {
    let mut dockerfile_content = DOCKERFILE_TEMPLATE.replace("{{FUNCTION}}", name);
    dockerfile_content = dockerfile_content.replace("{{ENV}}", &envs_to_string(envs));

    provisioning(&path, name, &dockerfile_content)
        .await
        .map_err(|e| Error::SystemError(e.to_string()))?;
    info!("Function docker image built");
    Ok(())
}

/// Deploys a function by building its files, provisioning a Docker container, and
/// registering it in the database if necessary.
///
/// This function:
/// 1. Creates the function's file structure and extracts its configuration.
/// 2. Provisions the Docker container for the function using the configuration.
/// 3. Registers the function in the database if it does not already exist.
///
/// # Arguments
///
/// * `conn` - A reference to the database connection.
/// * `function` - The function metadata and content.
///
/// # Returns
///
/// A success message indicating that the function was deployed.
pub async fn deploy_function(
    conn: &DatabaseConnection,
    function: Function,
) -> crate::error::Result<String> {
    let name = function.name;
    let runtime = function.runtime;
    let content = function.content;

    // Create the function files and extract configuration.
    let (envs, path) = create_function(&name, &runtime, content).await?;
    // Ensure environment variables are available.
    let envs = envs.ok_or_else(|| {
        Error::BadFunction("Missing environment configuration in function".to_string())
    })?;
    // Build the function Docker image.
    provision_docker(path, &name, envs).await?;

    // Register the function in the database if it's not already registered.
    if FunctionDBRepo::find_function_by_name(conn, &name)
        .await
        .is_none()
    {
        let new_function = FunctionModel {
            id: 0,
            auth_id: 0,
            name: name.clone(),
            runtime,
        };
        FunctionDBRepo::create_function(conn, new_function).await;
    }

    info!("Function '{}' deployed successfully", name);
    Ok(format!("Function '{}' deployed successfully", name))
}
