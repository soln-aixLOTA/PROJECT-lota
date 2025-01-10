use actix_web::{error::ResponseError, HttpResponse};
use base64::DecodeError;
use serde_json::json;
use sqlx::error::Error as SqlxError;
use std::string::FromUtf8Error;
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

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Authentication error: {0}")]
    AuthenticationError(String),

    #[error("Authorization error: {0}")]
    AuthorizationError(String),

    #[error("Rate limit exceeded")]
    RateLimitError,

    #[error("External service error: {0}")]
    ExternalServiceError(String),

    #[error("Operation timed out: {0}")]
    TimeoutError(String),

    #[error("Data conflict: {0}")]
    ConflictError(String),
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
            AppError::Auth(_) => {
                HttpResponse::Unauthorized().json(json!({ "error": self.to_string() }))
            }
            AppError::Internal(_) => {
                HttpResponse::InternalServerError().json(json!({ "error": self.to_string() }))
            }
            AppError::NotFound(_) => {
                HttpResponse::NotFound().json(json!({ "error": self.to_string() }))
            }
            AppError::BadRequest(_) => {
                HttpResponse::BadRequest().json(json!({ "error": self.to_string() }))
            }
            AppError::ConfigurationError(_) => {
                HttpResponse::InternalServerError().json(json!({ "error": self.to_string() }))
            }
            AppError::AuthenticationError(_) => {
                HttpResponse::Unauthorized().json(json!({ "error": self.to_string() }))
            }
            AppError::AuthorizationError(_) => {
                HttpResponse::Forbidden().json(json!({ "error": self.to_string() }))
            }
            AppError::RateLimitError => {
                HttpResponse::TooManyRequests().json(json!({ "error": self.to_string() }))
            }
            AppError::ExternalServiceError(_) => {
                HttpResponse::BadGateway().json(json!({ "error": self.to_string() }))
            }
            AppError::TimeoutError(_) => {
                HttpResponse::GatewayTimeout().json(json!({ "error": self.to_string() }))
            }
            AppError::ConflictError(_) => {
                HttpResponse::Conflict().json(json!({ "error": self.to_string() }))
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum DocumentError {
    #[error("Document not found")]
    NotFound,

    #[error("File too large (max 10MB)")]
    FileTooLarge,

    #[error("Invalid file type: {0}")]
    InvalidFileType(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Unauthorized access")]
    Unauthorized,

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Invalid UTF-8 in file content: {0}")]
    InvalidUtf8(#[from] FromUtf8Error),

    #[error("JSON serialization error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Storage error: {0}")]
    StorageError(String),
}

impl From<DocumentError> for actix_web::Error {
    fn from(err: DocumentError) -> Self {
        use actix_web::http::StatusCode;
        let error_response = match &err {
            DocumentError::NotFound => (StatusCode::NOT_FOUND, err.to_string()),
            DocumentError::Unauthorized => (StatusCode::UNAUTHORIZED, err.to_string()),
            DocumentError::FileTooLarge => (StatusCode::PAYLOAD_TOO_LARGE, err.to_string()),
            DocumentError::InvalidFileType(_) | DocumentError::InvalidInput(_) => {
                (StatusCode::BAD_REQUEST, err.to_string())
            }
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
        };
        actix_web::error::InternalError::from_response(
            err,
            actix_web::HttpResponse::build(error_response.0)
                .json(json!({ "error": error_response.1 })),
        )
        .into()
    }
}
