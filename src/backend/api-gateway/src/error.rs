use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Database error: {0}")]
    Database(String),
    #[error("External service error: {0}")]
    ExternalService(String),
    #[error("Authentication error: {0}")]
    Auth(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<Error> for io::Error {
    fn from(err: Error) -> Self {
        match err {
            Error::Io(e) => e,
            _ => io::Error::new(io::ErrorKind::Other, err.to_string()),
        }
    }
}
