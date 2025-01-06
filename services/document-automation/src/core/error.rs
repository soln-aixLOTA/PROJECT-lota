use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;
use tracing::error;

pub type DocumentResult<T> = Result<T, DocumentError>;

#[derive(Debug, Error)]
pub enum DocumentError {
    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("UUID error: {0}")]
    Uuid(#[from] uuid::Error),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Multipart error: {0}")]
    Multipart(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<axum::extract::multipart::MultipartError> for DocumentError {
    fn from(err: axum::extract::multipart::MultipartError) -> Self {
        DocumentError::Multipart(err.to_string())
    }
}

impl IntoResponse for DocumentError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            DocumentError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            DocumentError::Validation(msg) => (StatusCode::BAD_REQUEST, msg),
            DocumentError::Database(err) => {
                error!(?err, "Database error occurred");
                match err {
                    sqlx::Error::RowNotFound => (StatusCode::NOT_FOUND, "Resource not found"),
                    _ => (StatusCode::INTERNAL_SERVER_ERROR, "Database error occurred"),
                }
            }
            DocumentError::Json(err) => {
                error!(?err, "JSON serialization error occurred");
                (StatusCode::BAD_REQUEST, "Invalid JSON format")
            }
            DocumentError::Uuid(err) => {
                error!(?err, "UUID parsing error occurred");
                (StatusCode::BAD_REQUEST, "Invalid UUID format")
            }
            DocumentError::Storage(msg) => {
                error!(error = %msg, "Storage error occurred");
                (StatusCode::INTERNAL_SERVER_ERROR, msg)
            }
            DocumentError::Auth(msg) => (StatusCode::UNAUTHORIZED, msg),
            DocumentError::Io(err) => {
                error!(?err, "IO error occurred");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
            DocumentError::Multipart(msg) => (StatusCode::BAD_REQUEST, msg),
            DocumentError::Internal(msg) => {
                error!(error = %msg, "Internal error occurred");
                (StatusCode::INTERNAL_SERVER_ERROR, msg)
            }
        };

        let body = json!({
            "error": {
                "type": self.error_type(),
                "message": message,
                "details": self.error_details()
            }
        });

        (status, axum::Json(body)).into_response()
    }
}

impl DocumentError {
    fn error_type(&self) -> &'static str {
        match self {
            DocumentError::NotFound(_) => "NOT_FOUND",
            DocumentError::Validation(_) => "VALIDATION_ERROR",
            DocumentError::Database(_) => "DATABASE_ERROR",
            DocumentError::Json(_) => "JSON_ERROR",
            DocumentError::Uuid(_) => "UUID_ERROR",
            DocumentError::Storage(_) => "STORAGE_ERROR",
            DocumentError::Auth(_) => "AUTHENTICATION_ERROR",
            DocumentError::Io(_) => "IO_ERROR",
            DocumentError::Multipart(_) => "MULTIPART_ERROR",
            DocumentError::Internal(_) => "INTERNAL_ERROR",
        }
    }

    fn error_details(&self) -> Option<serde_json::Value> {
        match self {
            DocumentError::Validation(msg) => Some(json!({
                "validation_message": msg,
                "error_code": "400"
            })),
            DocumentError::Database(err) => Some(json!({
                "database_message": err.to_string(),
                "error_code": match err {
                    sqlx::Error::RowNotFound => "404",
                    _ => "500"
                }
            })),
            DocumentError::Storage(msg) => Some(json!({
                "storage_message": msg,
                "error_code": "500"
            })),
            DocumentError::Auth(msg) => Some(json!({
                "auth_error": msg,
                "error_code": "401"
            })),
            DocumentError::Json(err) => Some(json!({
                "parse_error": err.to_string(),
                "error_code": "400"
            })),
            DocumentError::Uuid(err) => Some(json!({
                "uuid_error": err.to_string(),
                "error_code": "400"
            })),
            DocumentError::Multipart(msg) => Some(json!({
                "multipart_error": msg,
                "error_code": "400"
            })),
            DocumentError::Io(err) => Some(json!({
                "io_error": err.to_string(),
                "error_code": "500"
            })),
            DocumentError::Internal(msg) => Some(json!({
                "internal_error": msg,
                "error_code": "500"
            })),
            DocumentError::NotFound(msg) => Some(json!({
                "resource": msg,
                "error_code": "404"
            })),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;
    use http::StatusCode;
    use serde_json::Value;
    use std::io::{Error as IoError, ErrorKind};
    use uuid::Error as UuidError;

    #[tokio::test]
    async fn test_error_responses() {
        // Test NotFound error
        let error = DocumentError::NotFound("Test resource".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        // Test Validation error
        let error = DocumentError::Validation("Invalid input".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        // Test Auth error
        let error = DocumentError::Auth("Invalid token".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_error_conversion() {
        // Test IO error conversion
        let io_error = IoError::new(ErrorKind::Other, "IO error");
        let doc_error: DocumentError = io_error.into();
        assert!(matches!(doc_error, DocumentError::Io(_)));

        // Test JSON error conversion
        let json_error =
            serde_json::Error::syntax(serde_json::error::ErrorCode::ExpectedColon, 0, 0);
        let doc_error: DocumentError = json_error.into();
        assert!(matches!(doc_error, DocumentError::Json(_)));

        // Test UUID error conversion
        let uuid_error = UuidError::InvalidLength(10);
        let doc_error: DocumentError = uuid_error.into();
        assert!(matches!(doc_error, DocumentError::Uuid(_)));

        // Test SQLx error conversion
        let sqlx_error = sqlx::Error::RowNotFound;
        let doc_error: DocumentError = sqlx_error.into();
        assert!(matches!(doc_error, DocumentError::Database(_)));
    }

    #[tokio::test]
    async fn test_error_response_format() {
        // Test validation error response
        let error = DocumentError::Validation("test error".to_string());
        let response = error.into_response();
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["error"]["type"], "VALIDATION_ERROR");
        assert_eq!(json["error"]["message"], "test error");
        assert_eq!(json["error"]["details"]["validation_message"], "test error");
        assert_eq!(json["error"]["details"]["error_code"], "400");

        // Test database error response
        let error = DocumentError::Database(sqlx::Error::RowNotFound);
        let response = error.into_response();
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["error"]["type"], "DATABASE_ERROR");
        assert_eq!(json["error"]["message"], "Resource not found");
        assert_eq!(json["error"]["details"]["error_code"], "404");
    }

    #[test]
    fn test_error_type_strings() {
        let test_cases = vec![
            (DocumentError::NotFound("test".into()), "NOT_FOUND"),
            (DocumentError::Validation("test".into()), "VALIDATION_ERROR"),
            (
                DocumentError::Database(sqlx::Error::RowNotFound),
                "DATABASE_ERROR",
            ),
            (
                DocumentError::Json(serde_json::Error::syntax(
                    serde_json::error::ErrorCode::ExpectedColon,
                    0,
                    0,
                )),
                "JSON_ERROR",
            ),
            (
                DocumentError::Uuid(UuidError::InvalidLength(10)),
                "UUID_ERROR",
            ),
            (DocumentError::Storage("test".into()), "STORAGE_ERROR"),
            (DocumentError::Auth("test".into()), "AUTHENTICATION_ERROR"),
            (
                DocumentError::Io(IoError::new(ErrorKind::Other, "test")),
                "IO_ERROR",
            ),
            (DocumentError::Multipart("test".into()), "MULTIPART_ERROR"),
            (DocumentError::Internal("test".into()), "INTERNAL_ERROR"),
        ];

        for (error, expected_type) in test_cases {
            assert_eq!(error.error_type(), expected_type);
        }
    }

    #[test]
    fn test_error_details() {
        // Test validation error details
        let error = DocumentError::Validation("Invalid input".to_string());
        let details = error.error_details().unwrap();
        assert_eq!(details["validation_message"], "Invalid input");
        assert_eq!(details["error_code"], "400");

        // Test storage error details
        let error = DocumentError::Storage("File not found".to_string());
        let details = error.error_details().unwrap();
        assert_eq!(details["storage_message"], "File not found");
        assert_eq!(details["error_code"], "500");

        // Test auth error details
        let error = DocumentError::Auth("Invalid token".to_string());
        let details = error.error_details().unwrap();
        assert_eq!(details["auth_error"], "Invalid token");
        assert_eq!(details["error_code"], "401");

        // Test multipart error details
        let error = DocumentError::Multipart("Invalid form data".to_string());
        let details = error.error_details().unwrap();
        assert_eq!(details["multipart_error"], "Invalid form data");
        assert_eq!(details["error_code"], "400");
    }

    #[tokio::test]
    async fn test_database_error_responses() {
        // Test row not found
        let error = DocumentError::Database(sqlx::Error::RowNotFound);
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        // Test other database errors
        let error = DocumentError::Database(sqlx::Error::Database(Box::new(
            sqlx::error::DatabaseError::Custom("Custom DB error".into()),
        )));
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
