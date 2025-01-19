use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Authentication error: {0}")]
    Authentication(String),
    #[error("Authorization error: {0}")]
    Authorization(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub message: String,
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let (status_code, error_code) = match self {
            AppError::Authentication(_) => (actix_web::http::StatusCode::UNAUTHORIZED, "AUTH_ERROR"),
            AppError::Authorization(_) => (actix_web::http::StatusCode::FORBIDDEN, "FORBIDDEN"),
            AppError::NotFound(_) => (actix_web::http::StatusCode::NOT_FOUND, "NOT_FOUND"),
            AppError::ValidationError(_) => (actix_web::http::StatusCode::BAD_REQUEST, "VALIDATION_ERROR"),
            AppError::DatabaseError(_) => (actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, "DB_ERROR"),
            AppError::Internal(_) => (actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR"),
        };

        HttpResponse::build(status_code).json(ErrorResponse {
            message: self.to_string(),
            code: error_code.to_string(),
            details: None,
        })
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
