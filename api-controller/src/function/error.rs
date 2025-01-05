use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    FunctionNotRegistered(String),
    FunctionFailedToStart(String),
    BadFunction(String),
    SystemError(String),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        println!("--> {:<12} {self:?}", "INTO_RES");
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
            Error::SystemError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "This is on us and we are working on it".to_string(),
            )
                .into_response(),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {}
