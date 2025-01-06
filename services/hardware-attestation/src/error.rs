use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AttestationError {
    #[error("NVML initialization failed: {0}")]
    NvmlInitError(String),

    #[error("Failed to get device count: {0}")]
    DeviceCountError(String),

    #[error("Failed to access GPU at index {index}: {message}")]
    GpuAccessError { index: usize, message: String },

    #[error("Failed to get GPU info: {0}")]
    GpuInfoError(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Not found: {0}")]
    NotFoundError(String),

    #[error("Internal server error: {0}")]
    InternalError(String),
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error_code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl IntoResponse for AttestationError {
    fn into_response(self) -> Response {
        let (status, error_response) = match self {
            AttestationError::NvmlInitError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse {
                    error_code: "NVML_INIT_ERROR".to_string(),
                    message: "Failed to initialize NVIDIA driver".to_string(),
                    details: Some(msg),
                },
            ),
            AttestationError::DeviceCountError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse {
                    error_code: "DEVICE_COUNT_ERROR".to_string(),
                    message: "Failed to enumerate GPU devices".to_string(),
                    details: Some(msg),
                },
            ),
            AttestationError::GpuAccessError { index, message } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse {
                    error_code: "GPU_ACCESS_ERROR".to_string(),
                    message: format!("Failed to access GPU at index {}", index),
                    details: Some(message),
                },
            ),
            AttestationError::GpuInfoError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse {
                    error_code: "GPU_INFO_ERROR".to_string(),
                    message: "Failed to retrieve GPU information".to_string(),
                    details: Some(msg),
                },
            ),
            AttestationError::StorageError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse {
                    error_code: "STORAGE_ERROR".to_string(),
                    message: "Database operation failed".to_string(),
                    details: Some(msg),
                },
            ),
            AttestationError::NotFoundError(msg) => (
                StatusCode::NOT_FOUND,
                ErrorResponse {
                    error_code: "NOT_FOUND".to_string(),
                    message: "Resource not found".to_string(),
                    details: Some(msg),
                },
            ),
            AttestationError::InternalError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse {
                    error_code: "INTERNAL_ERROR".to_string(),
                    message: "An internal server error occurred".to_string(),
                    details: Some(msg),
                },
            ),
        };

        (status, Json(error_response)).into_response()
    }
}

// Convenience function to convert anyhow::Error to AttestationError
impl From<anyhow::Error> for AttestationError {
    fn from(err: anyhow::Error) -> Self {
        AttestationError::InternalError(err.to_string())
    }
}
