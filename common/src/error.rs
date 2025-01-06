use serde::Serialize;
use serde_json::json;
use thiserror::Error;
use tracing::error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Invalid request: {0}")]
    BadRequest(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Debug, Serialize, PartialEq)]
pub struct ErrorResponse {
    pub error_type: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl Error {
    pub fn internal<T: ToString>(msg: T) -> Self {
        error!(message = %msg.to_string(), "Internal error occurred");
        Error::Internal(msg.to_string())
    }

    pub fn bad_request<T: ToString>(msg: T) -> Self {
        Error::BadRequest(msg.to_string())
    }

    pub fn not_found<T: ToString>(msg: T) -> Self {
        Error::NotFound(msg.to_string())
    }

    pub fn unauthorized<T: ToString>(msg: T) -> Self {
        Error::Unauthorized(msg.to_string())
    }

    pub fn forbidden<T: ToString>(msg: T) -> Self {
        Error::Forbidden(msg.to_string())
    }

    pub fn conflict<T: ToString>(msg: T) -> Self {
        Error::Conflict(msg.to_string())
    }

    pub fn error_type(&self) -> &'static str {
        match self {
            Error::Internal(_) => "INTERNAL_ERROR",
            Error::BadRequest(_) => "BAD_REQUEST",
            Error::NotFound(_) => "NOT_FOUND",
            Error::Unauthorized(_) => "UNAUTHORIZED",
            Error::Forbidden(_) => "FORBIDDEN",
            Error::Conflict(_) => "CONFLICT",
            Error::Other(_) => "UNKNOWN_ERROR",
        }
    }

    pub fn to_response(&self) -> ErrorResponse {
        ErrorResponse {
            error_type: self.error_type().to_string(),
            message: self.to_string(),
            details: self.error_details(),
        }
    }

    fn error_details(&self) -> Option<serde_json::Value> {
        match self {
            Error::Internal(msg) => Some(json!({
                "internal_details": msg,
                "error_code": "500"
            })),
            Error::BadRequest(msg) => Some(json!({
                "validation_details": msg,
                "error_code": "400"
            })),
            Error::NotFound(msg) => Some(json!({
                "resource": msg,
                "error_code": "404"
            })),
            Error::Unauthorized(msg) => Some(json!({
                "auth_error": msg,
                "error_code": "401"
            })),
            Error::Forbidden(msg) => Some(json!({
                "permission_error": msg,
                "error_code": "403"
            })),
            Error::Conflict(msg) => Some(json!({
                "conflict_details": msg,
                "error_code": "409"
            })),
            Error::Other(err) => Some(json!({
                "error_details": err.to_string(),
                "error_code": "500"
            })),
        }
    }
}

impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Self {
        error!(?err, "Database error occurred");
        match err {
            sqlx::Error::RowNotFound => Error::not_found("Resource not found"),
            _ => Error::internal(format!("Database error: {}", err)),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        error!(?err, "IO error occurred");
        Error::internal(format!("IO error: {}", err))
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        error!(?err, "JSON serialization error occurred");
        Error::bad_request(format!("Invalid JSON: {}", err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;
    use sqlx::error::Error as SqlxError;
    use std::io::{Error as IoError, ErrorKind};

    #[test]
    fn test_error_creation() {
        let err = Error::internal("test error");
        assert!(matches!(err, Error::Internal(_)));

        let err = Error::bad_request("test error");
        assert!(matches!(err, Error::BadRequest(_)));

        let err = Error::not_found("test error");
        assert!(matches!(err, Error::NotFound(_)));

        let err = Error::unauthorized("test error");
        assert!(matches!(err, Error::Unauthorized(_)));

        let err = Error::forbidden("test error");
        assert!(matches!(err, Error::Forbidden(_)));

        let err = Error::conflict("test error");
        assert!(matches!(err, Error::Conflict(_)));
    }

    #[test]
    fn test_error_display() {
        let err = Error::internal("test error");
        assert_eq!(err.to_string(), "Internal server error: test error");

        let err = Error::bad_request("test error");
        assert_eq!(err.to_string(), "Invalid request: test error");

        let err = Error::not_found("test error");
        assert_eq!(err.to_string(), "Not found: test error");

        let err = Error::unauthorized("test error");
        assert_eq!(err.to_string(), "Unauthorized: test error");

        let err = Error::forbidden("test error");
        assert_eq!(err.to_string(), "Forbidden: test error");

        let err = Error::conflict("test error");
        assert_eq!(err.to_string(), "Conflict: test error");
    }

    #[test]
    fn test_error_conversion() {
        // Test IO error conversion
        let io_error = IoError::new(ErrorKind::Other, "test error");
        let err: Error = io_error.into();
        assert!(matches!(err, Error::Internal(_)));

        // Test JSON error conversion
        let json_error =
            serde_json::Error::syntax(serde_json::error::ErrorCode::ExpectedColon, 0, 0);
        let err: Error = json_error.into();
        assert!(matches!(err, Error::BadRequest(_)));

        // Test SQLx error conversion
        let sqlx_error = SqlxError::RowNotFound;
        let err: Error = sqlx_error.into();
        assert!(matches!(err, Error::NotFound(_)));

        // Test anyhow error conversion
        let anyhow_error = anyhow!("test error");
        let err: Error = anyhow_error.into();
        assert!(matches!(err, Error::Other(_)));
    }

    #[test]
    fn test_error_response_format() {
        // Test internal error response
        let err = Error::internal("system failure");
        let response = err.to_response();
        assert_eq!(response.error_type, "INTERNAL_ERROR");
        assert_eq!(response.message, "Internal server error: system failure");
        assert!(response.details.is_some());
        if let Some(details) = response.details {
            assert_eq!(details["error_code"], "500");
            assert_eq!(details["internal_details"], "system failure");
        }

        // Test not found error response
        let err = Error::not_found("user");
        let response = err.to_response();
        assert_eq!(response.error_type, "NOT_FOUND");
        assert_eq!(response.message, "Not found: user");
        assert!(response.details.is_some());
        if let Some(details) = response.details {
            assert_eq!(details["error_code"], "404");
            assert_eq!(details["resource"], "user");
        }

        // Test bad request error response
        let err = Error::bad_request("invalid input");
        let response = err.to_response();
        assert_eq!(response.error_type, "BAD_REQUEST");
        assert_eq!(response.message, "Invalid request: invalid input");
        assert!(response.details.is_some());
        if let Some(details) = response.details {
            assert_eq!(details["error_code"], "400");
            assert_eq!(details["validation_details"], "invalid input");
        }
    }

    #[test]
    fn test_error_details() {
        // Test unauthorized error details
        let err = Error::unauthorized("invalid token");
        let details = err.error_details().unwrap();
        assert_eq!(details["error_code"], "401");
        assert_eq!(details["auth_error"], "invalid token");

        // Test forbidden error details
        let err = Error::forbidden("insufficient permissions");
        let details = err.error_details().unwrap();
        assert_eq!(details["error_code"], "403");
        assert_eq!(details["permission_error"], "insufficient permissions");

        // Test conflict error details
        let err = Error::conflict("resource already exists");
        let details = err.error_details().unwrap();
        assert_eq!(details["error_code"], "409");
        assert_eq!(details["conflict_details"], "resource already exists");

        // Test other error details
        let err = Error::Other(anyhow!("unknown error"));
        let details = err.error_details().unwrap();
        assert_eq!(details["error_code"], "500");
        assert_eq!(details["error_details"], "unknown error");
    }

    #[test]
    fn test_database_error_handling() {
        // Test row not found
        let err: Error = SqlxError::RowNotFound.into();
        assert!(matches!(err, Error::NotFound(_)));
        assert_eq!(err.to_string(), "Not found: Resource not found");

        // Test other database errors
        let err: Error = SqlxError::Database(Box::new(sqlx::error::DatabaseError::Custom(
            "Custom DB error".into(),
        )))
        .into();
        assert!(matches!(err, Error::Internal(_)));
        assert!(err.to_string().contains("Database error"));
    }
}
