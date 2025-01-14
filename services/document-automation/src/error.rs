use common::error::{Error as CommonError, ErrorResponse};
use thiserror::Error;
use std::io;

#[derive(Error, Debug)]
pub enum DocumentError {
    #[error("Document not found: {0}")]
    NotFound(String),

    #[error("Invalid document format: {0}")]
    InvalidFormat(String),

    #[error("Document storage error: {0}")]
    StorageError(String),

    #[error("Document processing error: {0}")]
    ProcessingError(String),

    #[error("Document size exceeds limit: {size}MB > {limit}MB")]
    SizeExceeded { size: u64, limit: u64 },

    #[error("Invalid content type: {found}, expected one of: {expected:?}")]
    InvalidContentType { found: String, expected: Vec<String> },

    #[error("Invalid filename: {0}")]
    InvalidFilename(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("File operation failed: {operation} on {path}: {error}")]
    FileOperation {
        operation: String,
        path: String,
        error: String,
    },

    #[error("Concurrent modification detected: {0}")]
    ConcurrentModification(String),

    #[error(transparent)]
    Common(#[from] CommonError),

    #[error(transparent)]
    Io(#[from] io::Error),
}

impl DocumentError {
    pub fn to_response(&self) -> ErrorResponse {
        match self {
            DocumentError::NotFound(msg) => ErrorResponse {
                error_type: "NOT_FOUND".to_string(),
                message: msg.clone(),
                details: None,
            },
            DocumentError::InvalidFormat(msg) => ErrorResponse {
                error_type: "INVALID_FORMAT".to_string(),
                message: msg.clone(),
                details: None,
            },
            DocumentError::StorageError(msg) => ErrorResponse {
                error_type: "STORAGE_ERROR".to_string(),
                message: msg.clone(),
                details: None,
            },
            DocumentError::ProcessingError(msg) => ErrorResponse {
                error_type: "PROCESSING_ERROR".to_string(),
                message: msg.clone(),
                details: None,
            },
            DocumentError::SizeExceeded { size, limit } => ErrorResponse {
                error_type: "SIZE_EXCEEDED".to_string(),
                message: format!("Document size ({size}MB) exceeds limit ({limit}MB)"),
                details: Some(serde_json::json!({
                    "size": size,
                    "limit": limit
                })),
            },
            DocumentError::InvalidContentType { found, expected } => ErrorResponse {
                error_type: "INVALID_CONTENT_TYPE".to_string(),
                message: format!("Invalid content type: {found}, expected one of: {expected:?}"),
                details: Some(serde_json::json!({
                    "found": found,
                    "expected": expected
                })),
            },
            DocumentError::InvalidFilename(msg) => ErrorResponse {
                error_type: "INVALID_FILENAME".to_string(),
                message: msg.clone(),
                details: None,
            },
            DocumentError::InvalidPath(msg) => ErrorResponse {
                error_type: "INVALID_PATH".to_string(),
                message: msg.clone(),
                details: None,
            },
            DocumentError::FileOperation { operation, path, error } => ErrorResponse {
                error_type: "FILE_OPERATION_ERROR".to_string(),
                message: format!("File operation '{}' failed on path '{}': {}", operation, path, error),
                details: Some(serde_json::json!({
                    "operation": operation,
                    "path": path,
                    "error": error
                })),
            },
            DocumentError::ConcurrentModification(msg) => ErrorResponse {
                error_type: "CONCURRENT_MODIFICATION".to_string(),
                message: msg.clone(),
                details: None,
            },
            DocumentError::Common(err) => err.to_response(),
            DocumentError::Io(err) => ErrorResponse {
                error_type: "IO_ERROR".to_string(),
                message: err.to_string(),
                details: None,
            },
        }
    }
}

impl axum::response::IntoResponse for DocumentError {
    fn into_response(self) -> axum::response::Response {
        let status = match &self {
            DocumentError::NotFound(_) => axum::http::StatusCode::NOT_FOUND,
            DocumentError::InvalidFormat(_) | 
            DocumentError::InvalidContentType { .. } |
            DocumentError::SizeExceeded { .. } |
            DocumentError::InvalidFilename(_) |
            DocumentError::InvalidPath(_) => axum::http::StatusCode::BAD_REQUEST,
            DocumentError::StorageError(_) |
            DocumentError::ProcessingError(_) |
            DocumentError::FileOperation { .. } |
            DocumentError::Io(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            DocumentError::ConcurrentModification(_) => axum::http::StatusCode::CONFLICT,
            DocumentError::Common(err) => match err {
                CommonError::Internal(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                CommonError::BadRequest(_) => axum::http::StatusCode::BAD_REQUEST,
                CommonError::NotFound(_) => axum::http::StatusCode::NOT_FOUND,
                CommonError::Unauthorized(_) => axum::http::StatusCode::UNAUTHORIZED,
                CommonError::Forbidden(_) => axum::http::StatusCode::FORBIDDEN,
                CommonError::Conflict(_) => axum::http::StatusCode::CONFLICT,
                CommonError::Other(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            },
        };

        let body = axum::Json(self.to_response());
        (status, body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_not_found() {
        let error = DocumentError::NotFound("test.pdf".to_string());
        let response = error.to_response();
        assert_eq!(response.error_type, "NOT_FOUND");
        assert_eq!(response.message, "test.pdf");
    }

    #[test]
    fn test_size_exceeded() {
        let error = DocumentError::SizeExceeded { size: 15, limit: 10 };
        let response = error.to_response();
        assert_eq!(response.error_type, "SIZE_EXCEEDED");
        assert!(response.message.contains("15MB"));
        assert!(response.message.contains("10MB"));
    }

    #[test]
    fn test_invalid_content_type() {
        let error = DocumentError::InvalidContentType {
            found: "image/png".to_string(),
            expected: vec!["application/pdf".to_string()],
        };
        let response = error.to_response();
        assert_eq!(response.error_type, "INVALID_CONTENT_TYPE");
        assert!(response.message.contains("image/png"));
        assert!(response.message.contains("application/pdf"));
    }

    #[test]
    fn test_file_operation_error() {
        let error = DocumentError::FileOperation {
            operation: "write".to_string(),
            path: "/test/path".to_string(),
            error: "permission denied".to_string(),
        };
        let response = error.to_response();
        assert_eq!(response.error_type, "FILE_OPERATION_ERROR");
        assert!(response.message.contains("write"));
        assert!(response.message.contains("/test/path"));
        assert!(response.message.contains("permission denied"));
    }

    #[test]
    fn test_concurrent_modification() {
        let error = DocumentError::ConcurrentModification("Document was modified by another request".to_string());
        let response = error.to_response();
        assert_eq!(response.error_type, "CONCURRENT_MODIFICATION");
        assert!(response.message.contains("modified by another request"));
    }
} 