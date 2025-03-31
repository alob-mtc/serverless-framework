use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a deployable function.
///
/// # Fields
/// - `name`: The unique name of the function.
/// - `runtime`: The runtime environment required by the function (e.g., "go").
/// - `content`: The zipped binary content of the function.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Function {
    pub name: String,
    pub runtime: String,
    pub content: Vec<u8>,
}

/// Represents the configuration for a function.
///
/// This configuration is typically extracted from a JSON file
/// bundled with the function's package.
///
/// # Fields
/// - `function_name`: The name of the function (should correspond to the `Function`'s name).
/// - `runtime`: The runtime environment for the function.
/// - `env`: Optional key-value pairs representing environment variables.
#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    function_name: String,
    runtime: String,
    pub(crate) env: Option<HashMap<String, String>>,
}
