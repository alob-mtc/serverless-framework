use std::fmt;

// Error
pub type AppResult<T> = Result<T, Error>;

pub enum Error {
    Exec(String),
    System(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Exec(e) => write!(f, "{e}"),
            Error::System(e) => write!(f, "System Error: {e}"),
        }
    }
}
