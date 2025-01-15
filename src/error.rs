use actix_web::{http::StatusCode, ResponseError};
use serde_json::json;
use sqlx;

#[derive(Debug)]
pub enum AppError {
    Authentication(String),
    Authorization(String),
    BadRequest(String),
    Database(sqlx::Error),
    NotFound(String),
    Internal(String),
    TooManyRequests(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Authentication(msg) => write!(f, "Authentication error: {}", msg),
            AppError::Authorization(msg) => write!(f, "Authorization error: {}", msg),
            AppError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            AppError::Database(err) => write!(f, "Database error: {}", err),
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::Internal(msg) => write!(f, "Internal error: {}", msg),
            AppError::TooManyRequests(msg) => write!(f, "Too many requests: {}", msg),
        }
    }
}

impl ResponseError for AppError {
    fn error_response(&self) -> actix_web::HttpResponse {
        let status = self.status_code();
        let message = self.to_string();

        actix_web::HttpResponse::build(status).json(json!({
            "error": message
        }))
    }

    fn status_code(&self) -> StatusCode {
        match self {
            AppError::Authentication(_) => StatusCode::UNAUTHORIZED,
            AppError::Authorization(_) => StatusCode::FORBIDDEN,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::TooManyRequests(_) => StatusCode::TOO_MANY_REQUESTS,
        }
    }
}
