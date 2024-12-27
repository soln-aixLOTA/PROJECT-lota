use actix_web::{
    error::ResponseError,
    http::{header::ContentType, StatusCode},
    HttpResponse,
};
use serde::Serialize;
use std::fmt;

#[derive(Debug)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub status: StatusCode,
}

impl ApiError {
    pub fn new(code: &str, message: &str) -> Self {
        ApiError {
            code: code.to_string(),
            message: message.to_string(),
            status: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn with_details<T: Into<String>, U: Into<String>>(
        code: T,
        message: U,
        status: StatusCode,
    ) -> Self {
        ApiError {
            code: code.into(),
            message: message.into(),
            status,
        }
    }

    pub fn internal_error() -> Self {
        ApiError::new("INTERNAL_ERROR", "An internal error occurred")
    }

    pub fn service_unavailable() -> Self {
        ApiError::with_details(
            "SERVICE_UNAVAILABLE",
            "Service is currently unavailable",
            StatusCode::SERVICE_UNAVAILABLE,
        )
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    code: String,
    message: String,
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        let error = ErrorResponse {
            code: self.code.clone(),
            message: self.message.clone(),
        };

        HttpResponse::build(self.status)
            .insert_header(ContentType::json())
            .json(error)
    }

    fn status_code(&self) -> StatusCode {
        self.status
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}
