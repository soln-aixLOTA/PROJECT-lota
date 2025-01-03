use actix_web::{error::ResponseError, HttpResponse};
use serde::Serialize;
use sqlx::Error as SqlxError;
use std::fmt;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UserError {
    #[error("Database error: {0}")]
    Database(#[from] SqlxError),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
}

impl ResponseError for UserError {
    fn error_response(&self) -> HttpResponse {
        let (status_code, code) = match self {
            UserError::Database(_) => (
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                "DATABASE_ERROR",
            ),
            UserError::Auth(_) => (
                actix_web::http::StatusCode::UNAUTHORIZED,
                "AUTHENTICATION_ERROR",
            ),
            UserError::Validation(_) => {
                (actix_web::http::StatusCode::BAD_REQUEST, "VALIDATION_ERROR")
            }
            UserError::NotFound(_) => (actix_web::http::StatusCode::NOT_FOUND, "NOT_FOUND"),
            UserError::Conflict(_) => (actix_web::http::StatusCode::CONFLICT, "CONFLICT"),
            UserError::Internal(_) => (
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
            ),
            UserError::Config(_) => (
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                "CONFIG_ERROR",
            ),
            UserError::Io(_) => (
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                "IO_ERROR",
            ),
        };

        HttpResponse::build(status_code).json(ErrorResponse {
            code: code.to_string(),
            message: self.to_string(),
        })
    }
}
