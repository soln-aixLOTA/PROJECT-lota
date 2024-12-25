use actix_web::{error::ResponseError, HttpResponse};
use derive_more::Display;
use serde_json::json;

#[derive(Debug, Display)]
pub enum CustomError {
    #[display(fmt = "Authentication error: {}", _0)]
    AuthenticationError(String),
    
    #[display(fmt = "Authorization error: {}", _0)]
    AuthorizationError(String),
    
    #[display(fmt = "Database error: {}", _0)]
    DatabaseError(String),
    
    #[display(fmt = "Validation error: {}", _0)]
    ValidationError(String),
    
    #[display(fmt = "Rate limit exceeded")]
    RateLimitExceeded,
    
    #[display(fmt = "Unauthorized")]
    Unauthorized,
    
    #[display(fmt = "Not Found")]
    NotFound,
    
    #[display(fmt = "Internal Server Error")]
    InternalServerError,
}

impl ResponseError for CustomError {
    fn error_response(&self) -> HttpResponse {
        match self {
            CustomError::AuthenticationError(msg) => {
                HttpResponse::Unauthorized().json(json!({
                    "error": "Authentication error",
                    "details": msg
                }))
            }
            CustomError::AuthorizationError(msg) => {
                HttpResponse::Forbidden().json(json!({
                    "error": "Authorization error",
                    "details": format!("Authorization failed: {}", msg)  // Include details for better context
                }))
            }
            CustomError::DatabaseError(msg) => {
                HttpResponse::InternalServerError().json(json!({
                    "error": "Database error",
                    "details": msg
                }))
            }
            CustomError::ValidationError(msg) => {
                HttpResponse::BadRequest().json(json!({
                    "error": "Validation error",
                    "details": msg
                }))
            }
            CustomError::RateLimitExceeded => {
                HttpResponse::TooManyRequests().json(json!({
                    "error": "Rate limit exceeded",
                    "details": "You have exceeded the allowed number of requests."
                }))
            }
            CustomError::Unauthorized => {
                HttpResponse::Unauthorized().json(json!({
                    "error": "Unauthorized"
                }))
            }
            CustomError::NotFound => {
                HttpResponse::NotFound().json(json!({
                    "error": "Not Found"
                }))
            }
            CustomError::InternalServerError => {
                HttpResponse::InternalServerError().json(json!({
                    "error": "Internal Server Error"
                }))
            }
        }
    }
}

impl From<sqlx::Error> for CustomError {
    fn from(err: sqlx::Error) -> Self {
        CustomError::DatabaseError(err.to_string())
    }
}