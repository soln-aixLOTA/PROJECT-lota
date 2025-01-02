use actix_web::{http::StatusCode, ResponseError};
use jsonwebtoken::errors::Error as JwtError;
use serde::Serialize;
use sqlx::Error as SqlxError;
use thiserror::Error;

#[derive(Error, Debug, Serialize)]
pub enum ApiError {
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Bad Request: {0}")]
    BadRequest(String),
    #[error("Internal Server Error: {0}")]
    InternalServerError(String),
    #[error("Not Found: {0}")]
    NotFound(String),
    #[error("Database Error: {0}")]
    DatabaseError(#[from] SqlxError),
    #[error("JWT Error: {0}")]
    JwtError(#[from] JwtError),
    #[error("User Not Found")]
    UserNotFound,
    #[error("Invalid Credentials")]
    InvalidCredentials,
    #[error("Password Mismatch")]
    PasswordMismatch,
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::JwtError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::UserNotFound => StatusCode::NOT_FOUND,
            ApiError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            ApiError::PasswordMismatch => StatusCode::UNAUTHORIZED,
        }
    }
}
