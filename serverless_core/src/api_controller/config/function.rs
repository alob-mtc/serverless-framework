use std::env;

const MAX_FUNCTION_SIZE_ENV_VARIABLE: &str = "MAX_FUNCTION_SIZE";
const DEFAULT_RUNTIME_ENV_VARIABLE: &str = "DEFAULT_RUNTIME";

/// Default runtime if not specified
pub const DEFAULT_RUNTIME_VALUE: &str = "go";

/// Default maximum function size (10MB)
pub const DEFAULT_MAX_FUNCTION_SIZE_VALUE: usize = 10 * 1024 * 1024;

/// Function service configuration
#[derive(Debug, Clone)]
pub struct InvokFunctionConfig {
    /// Default runtime to use for functions
    pub default_runtime: String,

    /// Maximum function size in bytes
    pub max_function_size: usize,
}

impl InvokFunctionConfig {
    /// Load function configuration from environment
    pub fn from_env() -> Self {
        let default_runtime = env::var(DEFAULT_RUNTIME_ENV_VARIABLE)
            .unwrap_or_else(|_| DEFAULT_RUNTIME_VALUE.to_string());

        let max_function_size = env::var(MAX_FUNCTION_SIZE_ENV_VARIABLE)
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(DEFAULT_MAX_FUNCTION_SIZE_VALUE);

        Self {
            default_runtime,
            max_function_size,
        }
    }
}
