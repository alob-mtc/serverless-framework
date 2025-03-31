use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;
use tracing::{debug, error};

/// A custom result type using our defined `Error`.
pub type Result<T> = core::result::Result<T, Error>;

/// Custom error type for function-related failures.
///
/// Variants cover cases such as a function not being registered,
/// failure to start a function, malformed function input, or
/// system-level errors.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Function not found: {0}")]
    FunctionNotRegistered(String),
    #[error("Failed to start function: {0}")]
    FunctionFailedToStart(String),
    #[error("Bad function: {0}")]
    BadFunction(String),
    #[error("System error: {0}")]
    SystemError(String),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        debug!("Converting error into response: {:?}", self);
        match self {
            Error::FunctionNotRegistered(f) => {
                (StatusCode::NOT_FOUND, format!("Function not found: {f}")).into_response()
            }
            Error::FunctionFailedToStart(s) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to start function: {s}"),
            )
                .into_response(),
            Error::BadFunction(b) => {
                (StatusCode::BAD_REQUEST, format!("Bad function: {b}")).into_response()
            }
            Error::SystemError(s) => {
                error!("System error occurred: {}", s);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "This is on us and we are working on it".to_string(),
                )
                    .into_response()
            }
        }
    }
}
