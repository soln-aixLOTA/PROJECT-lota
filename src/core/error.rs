<<<<<<< HEAD
use axum::extract::multipart::MultipartError;
=======
>>>>>>> 921251a (fetch)
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Debug)]
pub enum DocumentError {
    NotFound(String),
    ValidationError(String),
    AuthenticationError(String),
    AuthorizationError(String),
    DatabaseError(String),
    StorageError(String),
    SerializationError(String),
    InternalError(String),
}

impl Display for DocumentError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            DocumentError::NotFound(msg) => write!(f, "Not found: {}", msg),
            DocumentError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            DocumentError::AuthenticationError(msg) => write!(f, "Authentication error: {}", msg),
            DocumentError::AuthorizationError(msg) => write!(f, "Authorization error: {}", msg),
            DocumentError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            DocumentError::StorageError(msg) => write!(f, "Storage error: {}", msg),
            DocumentError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            DocumentError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for DocumentError {}

impl IntoResponse for DocumentError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            DocumentError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            DocumentError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
            DocumentError::AuthenticationError(msg) => (StatusCode::UNAUTHORIZED, msg),
            DocumentError::AuthorizationError(msg) => (StatusCode::FORBIDDEN, msg),
            DocumentError::DatabaseError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            DocumentError::StorageError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            DocumentError::SerializationError(msg) => (StatusCode::BAD_REQUEST, msg),
            DocumentError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(json!({
            "error": {
                "message": error_message,
                "code": status.as_u16()
            }
        }));

        (status, body).into_response()
    }
}

impl From<sqlx::Error> for DocumentError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => DocumentError::NotFound("Resource not found".to_string()),
            _ => DocumentError::DatabaseError(err.to_string()),
        }
    }
}

impl From<serde_json::Error> for DocumentError {
    fn from(err: serde_json::Error) -> Self {
        DocumentError::SerializationError(err.to_string())
    }
}

impl From<std::string::FromUtf8Error> for DocumentError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        DocumentError::SerializationError(err.to_string())
    }
}

impl From<uuid::Error> for DocumentError {
    fn from(err: uuid::Error) -> Self {
        DocumentError::ValidationError(err.to_string())
    }
}

<<<<<<< HEAD
impl From<MultipartError> for DocumentError {
    fn from(err: MultipartError) -> Self {
        DocumentError::ValidationError(err.to_string())
    }
}

impl From<std::io::Error> for DocumentError {
    fn from(err: std::io::Error) -> Self {
        DocumentError::StorageError(err.to_string())
    }
}

=======
>>>>>>> 921251a (fetch)
pub type DocumentResult<T> = Result<T, DocumentError>;
