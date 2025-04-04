// use crate::template::ROUTES_TEMPLATE;
use crate::utils::{create_fn_project_file, init_go_mod, GlobalConfig};
use fn_utils::{compress_dir_with_excludes, template::ROUTES_TEMPLATE, to_camel_case_handler};
use reqwest::blocking::{multipart, Client};
use std::fs::File;
use std::io::{self, Cursor, Read, Write};
use std::path::Path;
use std::time::Duration;
use thiserror::Error;

// Constants
const API_ENDPOINT: &str = "http://127.0.0.1:3000/upload";
const REQUEST_TIMEOUT_SECS: u64 = 120;
const CONFIG_FILE_PATH: &str = "./config.json";

/// Errors that can occur during serverless function operations
#[derive(Debug, Error)]
pub enum FunctionError {
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Network request error: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Function not found: {0}")]
    FunctionNotFound(String),

    #[error("Compression error: {0}")]
    CompressionError(String),
}

/// Creates a new serverless function project with the specified name and runtime.
///
/// # Arguments
///
/// * `name` - The name of the function to create
/// * `runtime` - The runtime to use (e.g., "go")
///
/// # Returns
///
/// A Result indicating success or containing an error
pub fn create_new_project(name: &str, runtime: &str) -> Result<(), FunctionError> {
    println!("Creating service... '{name}' [RUNTIME:'{runtime}']");
    let handler_name = to_camel_case_handler(name);

    // Create project file
    let file = create_fn_project_file(name, runtime)?;
    let mut file = std::io::BufWriter::new(&file);

    // Write template with replacements
    file.write_all(
        ROUTES_TEMPLATE
            .replace("{{ROUTE}}", name)
            .replace("{{HANDLER}}", &handler_name)
            .as_bytes(),
    )?;

    // Initialize go module
    init_go_mod(name)?;
    println!("Function created");

    Ok(())
}

/// Deploys an existing function to the serverless platform.
///
/// # Arguments
///
/// * `name` - The name of the function to deploy
///
/// # Returns
///
/// A Result indicating success or containing an error
pub fn deploy_function(name: &str) -> Result<(), FunctionError> {
    // Read configuration file
    let mut config_file = File::open(CONFIG_FILE_PATH)?;
    let mut contents = String::new();
    config_file.read_to_string(&mut contents)?;

    let config: GlobalConfig = serde_json::from_str(&contents)?;

    // Validate function exists in config
    if !config.function_name.contains(&name.to_string()) {
        return Err(FunctionError::FunctionNotFound(name.to_string()));
    }

    let _runtime = config.runtime;
    println!("Deploying service... '{}'", name);

    // Create ZIP archive
    let mut dest_zip = Cursor::new(Vec::new());
    compress_dir_with_excludes(&Path::new(name), &mut dest_zip, &["go.mod", "go.sum"])
        .map_err(|e| FunctionError::CompressionError(e.to_string()))?;

    // Reset the cursor to the beginning of the buffer
    dest_zip.set_position(0);

    // Create multipart form
    let form = multipart::Form::new().part(
        "file",
        multipart::Part::reader(dest_zip)
            .file_name(format!("{name}.zip"))
            .mime_str("application/zip")?,
    );

    // Build client with timeout
    let client = Client::builder()
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
        .build()?;

    // Send request to API
    let response = client.post(API_ENDPOINT).multipart(form).send()?;

    // Check the response
    if response.status().is_success() {
        let response_text = response.text()?;
        println!("Response: {}", response_text);
        Ok(())
    } else {
        let status = response.status();
        let error_text = response
            .text()
            .unwrap_or_else(|_| "Unknown error".to_string());
        println!(
            "Failed to upload file: Status: {}, Error: {}",
            status, error_text
        );
        // Create a custom error message
        Err(FunctionError::CompressionError(format!(
            "API error: Status code {}",
            status
        )))
    }
}
