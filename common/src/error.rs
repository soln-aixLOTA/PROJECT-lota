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
        Error::bad_request(format!("Invalid JSON: {}", err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::error::{DatabaseError, ErrorKind};
    use std::borrow::Cow;

    #[test]
    fn test_json_error() {
        let err: Error = serde_json::Error::from(serde_json::error::Error::io(std::io::Error::new(
            std::io::ErrorKind::Other,
            "test error",
        ))).into();
        assert!(matches!(err, Error::BadRequest { .. }));
    }

    #[test]
    fn test_database_error() {
        let err: Error = sqlx::Error::Database(Box::new(TestDatabaseError)).into();
        assert!(matches!(err, Error::Internal { .. }));
    }

    struct TestDatabaseError;

    impl DatabaseError for TestDatabaseError {
        fn message(&self) -> &str {
            "test error"
        }

        fn code(&self) -> Option<Cow<'_, str>> {
            None
        }

        fn as_error(&self) -> &(dyn std::error::Error + Send + Sync + 'static) {
            self
        }

        fn as_error_mut(&mut self) -> &mut (dyn std::error::Error + Send + Sync + 'static) {
            self
        }

        fn into_error(self: Box<Self>) -> Box<dyn std::error::Error + Send + Sync + 'static> {
            self
        }

        fn kind(&self) -> ErrorKind {
            ErrorKind::Other
        }
    }

    impl std::fmt::Display for TestDatabaseError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "test error")
        }
    }

    impl std::fmt::Debug for TestDatabaseError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "TestDatabaseError")
        }
    }

    impl std::error::Error for TestDatabaseError {}
}
