use actix_web::{
    error::ResponseError,
    http::{header::HeaderMap, StatusCode},
    HttpResponse,
};
use serde::Serialize;
use std::fmt;
use tracing::error;

#[derive(Debug)]
pub enum ApiError {
    Unauthorized(String),
    Forbidden(String),
    NotFound(String),
    BadRequest(String),
    InternalError(String),
    ServiceUnavailable(String),
    Configuration(String),
    RateLimitExceeded(u64),
    ValidationError { field: String, message: String },
}

#[derive(Serialize)]
struct ErrorResponse {
    code: u16,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<serde_json::Value>,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ApiError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            ApiError::Forbidden(msg) => write!(f, "Forbidden: {}", msg),
            ApiError::NotFound(msg) => write!(f, "Not Found: {}", msg),
            ApiError::BadRequest(msg) => write!(f, "Bad Request: {}", msg),
            ApiError::InternalError(msg) => write!(f, "Internal Server Error: {}", msg),
            ApiError::ServiceUnavailable(msg) => write!(f, "Service Unavailable: {}", msg),
            ApiError::Configuration(msg) => write!(f, "Configuration Error: {}", msg),
            ApiError::RateLimitExceeded(retry_after) => {
                write!(f, "Rate Limit Exceeded. Retry after {} seconds", retry_after)
            }
            ApiError::ValidationError { field, message } => {
                write!(f, "Validation Error - {}: {}", field, message)
            }
        }
    }
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        let mut response = match self {
            ApiError::Unauthorized(_) => HttpResponse::Unauthorized(),
            ApiError::Forbidden(_) => HttpResponse::Forbidden(),
            ApiError::NotFound(_) => HttpResponse::NotFound(),
            ApiError::BadRequest(_) => HttpResponse::BadRequest(),
            ApiError::InternalError(_) => {
                error!("Internal server error: {}", self);
                HttpResponse::InternalServerError()
            }
            ApiError::ServiceUnavailable(_) => HttpResponse::ServiceUnavailable(),
            ApiError::Configuration(_) => {
                error!("Configuration error: {}", self);
                HttpResponse::InternalServerError()
            }
            ApiError::RateLimitExceeded(retry_after) => {
                let mut resp = HttpResponse::TooManyRequests();
                resp.insert_header(("Retry-After", retry_after.to_string()));
                return resp.json(ErrorResponse {
                    code: 429,
                    message: self.to_string(),
                    details: Some(serde_json::json!({
                        "retry_after": retry_after
                    })),
                });
            }
            ApiError::ValidationError { field, message } => {
                HttpResponse::BadRequest().json(ErrorResponse {
                    code: 400,
                    message: "Validation Error".to_string(),
                    details: Some(serde_json::json!({
                        "field": field,
                        "message": message
                    })),
                })
            }
        };

        response.json(ErrorResponse {
            code: self.status_code().as_u16(),
            message: self.to_string(),
            details: None,
        })
    }

    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            ApiError::Forbidden(_) => StatusCode::FORBIDDEN,
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            ApiError::Configuration(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::RateLimitExceeded(_) => StatusCode::TOO_MANY_REQUESTS,
            ApiError::ValidationError { .. } => StatusCode::BAD_REQUEST,
        }
    }
}

impl From<std::io::Error> for ApiError {
    fn from(err: std::io::Error) -> Self {
        ApiError::InternalError(err.to_string())
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        ApiError::BadRequest(err.to_string())
    }
}

impl From<reqwest::Error> for ApiError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            ApiError::ServiceUnavailable("Service timeout".into())
        } else if err.is_connect() {
            ApiError::ServiceUnavailable("Service unavailable".into())
        } else {
            ApiError::InternalError(err.to_string())
        }
    }
} 