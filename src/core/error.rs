use actix_web::{error::ResponseError, HttpResponse};
use base64::DecodeError;
use serde_json::json;
use sqlx::error::Error as SqlxError;
use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),
}

impl From<SqlxError> for AppError {
    fn from(err: SqlxError) -> Self {
        match err {
            SqlxError::RowNotFound => AppError::NotFound("Resource not found".to_string()),
            _ => AppError::Database(err.to_string()),
        }
    }
}

impl From<DecodeError> for AppError {
    fn from(err: DecodeError) -> Self {
        AppError::BadRequest(format!("Invalid base64: {}", err))
    }
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::Database(_) => {
                HttpResponse::InternalServerError().json(json!({ "error": self.to_string() }))
            }
            AppError::Storage(_) => {
                HttpResponse::InternalServerError().json(json!({ "error": self.to_string() }))
            }
            AppError::Auth(_) => HttpResponse::Unauthorized().json(json!({ "error": self.to_string() })),
            AppError::Internal(_) => {
                HttpResponse::InternalServerError().json(json!({ "error": self.to_string() }))
            }
            AppError::NotFound(_) => {
                HttpResponse::NotFound().json(json!({ "error": self.to_string() }))
            }
            AppError::BadRequest(_) => {
                HttpResponse::BadRequest().json(json!({ "error": self.to_string() }))
            }
        }
    }
}
