use actix_web::{error::ResponseError, http::StatusCode, HttpResponse};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ApiError {
    pub fn new<T: Into<String>, U: Into<String>>(code: T, message: U) -> Self {
        ApiError {
            code: code.into(),
            message: message.into(),
            details: None,
        }
    }

    pub fn with_details<T: Into<String>, U: Into<String>>(
        code: T,
        message: U,
        details: serde_json::Value,
    ) -> Self {
        ApiError {
            code: code.into(),
            message: message.into(),
            details: Some(details),
        }
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        let mut response = HttpResponse::build(self.status_code());

        if let Some(details) = &self.details {
            response.json(serde_json::json!({
                "error": {
                    "code": self.code,
                    "message": self.message,
                    "details": details
                }
            }))
        } else {
            response.json(serde_json::json!({
                "error": {
                    "code": self.code,
                    "message": self.message
                }
            }))
        }
    }

    fn status_code(&self) -> StatusCode {
        match self.code.as_str() {
            "VALIDATION_ERROR" => StatusCode::BAD_REQUEST,
            "RATE_LIMIT_EXCEEDED" => StatusCode::TOO_MANY_REQUESTS,
            "UNAUTHORIZED" => StatusCode::UNAUTHORIZED,
            "FORBIDDEN" => StatusCode::FORBIDDEN,
            "NOT_FOUND" => StatusCode::NOT_FOUND,
            "INTERNAL_ERROR" => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl std::error::Error for ApiError {}
