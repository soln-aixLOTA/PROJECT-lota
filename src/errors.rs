use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Authentication failed")]
    Unauthorized,
    #[error("Resource not found")]
    NotFound,
    #[error("Internal server error")]
    Internal,
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("Service unavailable")]
    ServiceUnavailable,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<String>,
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ApiError::Unauthorized => HttpResponse::Unauthorized().json(ErrorResponse {
                error: "unauthorized".to_string(),
                message: self.to_string(),
                details: None,
            }),
            ApiError::NotFound => HttpResponse::NotFound().json(ErrorResponse {
                error: "not_found".to_string(),
                message: self.to_string(),
                details: None,
            }),
            ApiError::Internal => HttpResponse::InternalServerError().json(ErrorResponse {
                error: "internal_error".to_string(),
                message: self.to_string(),
                details: None,
            }),
            ApiError::BadRequest(msg) => HttpResponse::BadRequest().json(ErrorResponse {
                error: "bad_request".to_string(),
                message: msg.to_string(),
                details: None,
            }),
            ApiError::RateLimitExceeded => HttpResponse::TooManyRequests().json(ErrorResponse {
                error: "rate_limit_exceeded".to_string(),
                message: self.to_string(),
                details: None,
            }),
            ApiError::ServiceUnavailable => {
                HttpResponse::ServiceUnavailable().json(ErrorResponse {
                    error: "service_unavailable".to_string(),
                    message: self.to_string(),
                    details: None,
                })
            }
        }
    }
}
