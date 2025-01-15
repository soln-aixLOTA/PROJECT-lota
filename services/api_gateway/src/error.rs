use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;
use thiserror::Error;
use validator::ValidationErrors;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Authentication failed: {0}")]
    AuthenticationError(String),

    #[error("Authorization failed: {0}")]
    AuthorizationError(String),

    #[error("Validation failed: {0}")]
    ValidationError(String),

    #[error("Not found: {0}")]
    NotFoundError(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Internal server error: {0}")]
    InternalError(String),
}

#[derive(Serialize)]
struct ErrorResponse {
    success: bool,
    error: String,
    message: String,
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        let error_response = ErrorResponse {
            success: false,
            error: self.to_string(),
            message: self.to_string(),
        };

        match self {
            ApiError::AuthenticationError(_) => HttpResponse::Unauthorized().json(error_response),
            ApiError::AuthorizationError(_) => HttpResponse::Forbidden().json(error_response),
            ApiError::ValidationError(_) => HttpResponse::BadRequest().json(error_response),
            ApiError::NotFoundError(_) => HttpResponse::NotFound().json(error_response),
            ApiError::DatabaseError(_) => HttpResponse::InternalServerError().json(error_response),
            ApiError::InternalError(_) => HttpResponse::InternalServerError().json(error_response),
        }
    }
}

impl From<ValidationErrors> for ApiError {
    fn from(errors: ValidationErrors) -> Self {
        ApiError::ValidationError(errors.to_string())
    }
}

impl From<bcrypt::BcryptError> for ApiError {
    fn from(err: bcrypt::BcryptError) -> Self {
        ApiError::InternalError(err.to_string())
    }
}

impl From<jsonwebtoken::errors::Error> for ApiError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        ApiError::AuthenticationError(err.to_string())
    }
}
