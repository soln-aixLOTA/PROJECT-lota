use actix_web::{error::ResponseError, http::StatusCode, HttpResponse};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub code: u16,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();
        let response = ErrorResponse {
            code: status.as_u16(),
            message: self.to_string(),
            details: None,
        };

        HttpResponse::build(status).json(response)
    }

    fn status_code(&self) -> StatusCode {
        match self {
            Error::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Config(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Validation(_) => StatusCode::BAD_REQUEST,
            Error::NotFound(_) => StatusCode::NOT_FOUND,
            Error::Conflict(_) => StatusCode::CONFLICT,
            Error::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
