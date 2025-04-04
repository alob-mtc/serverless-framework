use std::env;
use thiserror::Error;

/// Error that can occur during configuration loading
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Missing environment variable: {0}")]
    MissingVar(String),
    
    #[error("Invalid environment variable value: {0}")]
    InvalidValue(String),
    
    #[error("Environment error: {0}")]
    EnvError(#[from] env::VarError),
}

/// Server configuration 
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Redis connection URL
    pub redis_url: String,
    
    /// Database connection URL
    pub database_url: String,
    
    /// Server listen address
    pub host: String,
    
    /// Server listen port
    pub port: u16,
}

impl ServerConfig {
    /// Default port to use if not configured
    pub const DEFAULT_PORT: u16 = 3000;
    
    /// Default host to bind to if not configured
    pub const DEFAULT_HOST: &'static str = "0.0.0.0";
    
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, ConfigError> {
        // Required variables
        let redis_url = env::var("REDIS_URL")
            .map_err(|_| ConfigError::MissingVar("REDIS_URL".to_string()))?;
            
        let database_url = env::var("DATABASE_URL")
            .map_err(|_| ConfigError::MissingVar("DATABASE_URL".to_string()))?;
        
        // Optional variables with defaults
        let host = env::var("SERVER_HOST").unwrap_or_else(|_| Self::DEFAULT_HOST.to_string());
        
        let port = match env::var("SERVER_PORT") {
            Ok(port_str) => port_str
                .parse::<u16>()
                .map_err(|_| ConfigError::InvalidValue("SERVER_PORT".to_string()))?,
            Err(_) => Self::DEFAULT_PORT,
        };
        
        Ok(Self {
            redis_url,
            database_url,
            host,
            port,
        })
    }
}

/// Function service configuration
#[derive(Debug, Clone)]
pub struct FunctionConfig {
    /// Default runtime to use for functions
    pub default_runtime: String,
    
    /// Maximum function size in bytes
    pub max_function_size: usize,
}

impl FunctionConfig {
    /// Default runtime if not specified
    pub const DEFAULT_RUNTIME: &'static str = "go";
    
    /// Default maximum function size (10MB)
    pub const DEFAULT_MAX_FUNCTION_SIZE: usize = 10 * 1024 * 1024;
    
    /// Load function configuration from environment
    pub fn from_env() -> Self {
        let default_runtime = env::var("DEFAULT_RUNTIME")
            .unwrap_or_else(|_| Self::DEFAULT_RUNTIME.to_string());
            
        let max_function_size = env::var("MAX_FUNCTION_SIZE")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(Self::DEFAULT_MAX_FUNCTION_SIZE);
            
        Self {
            default_runtime,
            max_function_size,
        }
    }
}

/// Complete application configuration
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Server configuration
    pub server: ServerConfig,
    
    /// Function configuration
    pub function: FunctionConfig,
}

impl AppConfig {
    /// Load complete configuration from environment
    pub fn load() -> Result<Self, ConfigError> {
        let server = ServerConfig::from_env()?;
        let function = FunctionConfig::from_env();
        
        Ok(Self {
            server,
            function,
        })
    }
} 