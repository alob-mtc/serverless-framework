use std::fmt;

// Error
pub type AppResult<T> = Result<T, RuntimeError>;

#[derive(Debug)]
pub enum RuntimeError {
    Exec(String),
    System(String),
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeError::Exec(e) => write!(f, "{e}"),
            RuntimeError::System(e) => write!(f, "System Error: {e}"),
        }
    }
}
