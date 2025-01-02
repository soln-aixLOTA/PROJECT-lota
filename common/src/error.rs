use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

#[derive(Debug, Error, Serialize, Deserialize)]
#[serde(tag = "type", content = "details")]
pub enum CommonError {
    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Authorization error: {0}")]
    Authorization(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Configuration error: {0}")]
    Configuration(String),
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

impl ErrorResponse {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: None,
            request_id: None,
        }
    }

    pub fn with_details(mut self, details: impl Serialize) -> Self {
        self.details = serde_json::to_value(details).ok();
        self
    }

    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }
}

impl From<CommonError> for ErrorResponse {
    fn from(error: CommonError) -> Self {
        let (code, message) = match &error {
            CommonError::Authentication(_) => ("AUTH_ERROR", error.to_string()),
            CommonError::Authorization(_) => ("FORBIDDEN", error.to_string()),
            CommonError::Validation(_) => ("VALIDATION_ERROR", error.to_string()),
            CommonError::RateLimit(_) => ("RATE_LIMIT_EXCEEDED", error.to_string()),
            CommonError::NotFound(_) => ("NOT_FOUND", error.to_string()),
            CommonError::Internal(_) => {
                ("INTERNAL_ERROR", "An internal error occurred".to_string())
            }
            CommonError::Database(_) => ("DATABASE_ERROR", "A database error occurred".to_string()),
            CommonError::Configuration(_) => ("CONFIG_ERROR", error.to_string()),
        };

        ErrorResponse::new(code, message)
    }
}

pub type Result<T> = std::result::Result<T, CommonError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_response_creation() {
        let error = CommonError::Authentication("Invalid token".to_string());
        let response: ErrorResponse = error.into();

        assert_eq!(response.code, "AUTH_ERROR");
        assert!(response.message.contains("Invalid token"));
    }

    #[test]
    fn test_error_response_with_details() {
        let response =
            ErrorResponse::new("TEST_ERROR", "Test error").with_details(serde_json::json!({
                "field": "username",
                "reason": "too short"
            }));

        assert!(response.details.is_some());
    }
}
