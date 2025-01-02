use std::fmt;
use std::error::Error;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use tracing::{error, warn};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub code: ErrorCode,
    pub message: String,
    pub request_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub details: Option<ErrorDetails>,
    pub retry_after: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    // Client Errors (4xx)
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    MethodNotAllowed,
    RequestTimeout,
    TooManyRequests,
    PayloadTooLarge,
    
    // Server Errors (5xx)
    InternalError,
    ServiceUnavailable,
    GatewayTimeout,
    WorkerError,
    DatabaseError,
    ValidationError,
    ConfigurationError,
    
    // Integration Errors
    UpstreamServiceError,
    CircuitBreakerOpen,
    RateLimitExceeded,
    
    // Resource Errors
    ResourceExhausted,
    ResourceNotAvailable,
    ResourceConflict,
    
    // Security Errors
    AuthenticationFailed,
    TokenExpired,
    InvalidCredentials,
    
    // Data Errors
    DataNotFound,
    DataValidationFailed,
    DataCorrupted,
    
    // Worker Pool Errors
    WorkerPoolFull,
    WorkerNotResponding,
    WorkerCrashed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetails {
    pub error_type: String,
    pub source: Option<String>,
    pub stack_trace: Option<String>,
    pub correlation_id: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

impl ApiError {
    pub fn new(code: ErrorCode, message: String) -> Self {
        Self {
            code,
            message,
            request_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            details: None,
            retry_after: None,
        }
    }
    
    pub fn with_details(mut self, details: ErrorDetails) -> Self {
        self.details = Some(details);
        self
    }
    
    pub fn with_retry_after(mut self, seconds: u64) -> Self {
        self.retry_after = Some(seconds);
        self
    }
    
    pub fn log(&self) {
        match self.code {
            ErrorCode::InternalError | 
            ErrorCode::ServiceUnavailable |
            ErrorCode::DatabaseError |
            ErrorCode::WorkerCrashed => {
                error!(
                    request_id = %self.request_id,
                    error_code = ?self.code,
                    error_message = %self.message,
                    error_details = ?self.details,
                    "Critical error occurred"
                );
            }
            ErrorCode::TooManyRequests |
            ErrorCode::CircuitBreakerOpen |
            ErrorCode::ResourceExhausted => {
                warn!(
                    request_id = %self.request_id,
                    error_code = ?self.code,
                    error_message = %self.message,
                    retry_after = ?self.retry_after,
                    "Resource limit reached"
                );
            }
            _ => {
                warn!(
                    request_id = %self.request_id,
                    error_code = ?self.code,
                    error_message = %self.message,
                    "Error occurred"
                );
            }
        }
    }
    
    pub fn status_code(&self) -> u16 {
        match self.code {
            ErrorCode::BadRequest => 400,
            ErrorCode::Unauthorized => 401,
            ErrorCode::Forbidden => 403,
            ErrorCode::NotFound => 404,
            ErrorCode::MethodNotAllowed => 405,
            ErrorCode::RequestTimeout => 408,
            ErrorCode::TooManyRequests => 429,
            ErrorCode::PayloadTooLarge => 413,
            ErrorCode::InternalError => 500,
            ErrorCode::ServiceUnavailable => 503,
            ErrorCode::GatewayTimeout => 504,
            _ => 500,
        }
    }
}

impl std::error::Error for ApiError {}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Error({}): {} [Request-ID: {}]",
            self.code as u16,
            self.message,
            self.request_id
        )
    }
}

impl From<std::io::Error> for ApiError {
    fn from(err: std::io::Error) -> Self {
        Self::new(
            ErrorCode::InternalError,
            format!("IO error: {}", err),
        ).with_details(ErrorDetails {
            error_type: "IoError".to_string(),
            source: Some(err.to_string()),
            stack_trace: None,
            correlation_id: None,
            metadata: None,
        })
    }
}

impl From<tokio::task::JoinError> for ApiError {
    fn from(err: tokio::task::JoinError) -> Self {
        Self::new(
            ErrorCode::WorkerCrashed,
            format!("Worker task failed: {}", err),
        ).with_details(ErrorDetails {
            error_type: "WorkerError".to_string(),
            source: Some(err.to_string()),
            stack_trace: None,
            correlation_id: None,
            metadata: None,
        })
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        Self::new(
            ErrorCode::DataValidationFailed,
            format!("JSON error: {}", err),
        ).with_details(ErrorDetails {
            error_type: "JsonError".to_string(),
            source: Some(err.to_string()),
            stack_trace: None,
            correlation_id: None,
            metadata: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_api_error_creation() {
        let error = ApiError::new(
            ErrorCode::BadRequest,
            "Invalid input".to_string(),
        );
        
        assert_eq!(error.code, ErrorCode::BadRequest);
        assert_eq!(error.message, "Invalid input");
        assert_eq!(error.status_code(), 400);
        assert!(error.details.is_none());
        assert!(error.retry_after.is_none());
    }
    
    #[test]
    fn test_api_error_with_details() {
        let details = ErrorDetails {
            error_type: "ValidationError".to_string(),
            source: Some("test".to_string()),
            stack_trace: None,
            correlation_id: Some("test-123".to_string()),
            metadata: None,
        };
        
        let error = ApiError::new(
            ErrorCode::ValidationError,
            "Validation failed".to_string(),
        ).with_details(details.clone());
        
        assert_eq!(error.code, ErrorCode::ValidationError);
        assert!(error.details.is_some());
        let error_details = error.details.unwrap();
        assert_eq!(error_details.error_type, "ValidationError");
        assert_eq!(error_details.source, Some("test".to_string()));
        assert_eq!(error_details.correlation_id, Some("test-123".to_string()));
    }
    
    #[test]
    fn test_api_error_with_retry_after() {
        let error = ApiError::new(
            ErrorCode::TooManyRequests,
            "Rate limit exceeded".to_string(),
        ).with_retry_after(60);
        
        assert_eq!(error.code, ErrorCode::TooManyRequests);
        assert_eq!(error.retry_after, Some(60));
        assert_eq!(error.status_code(), 429);
    }
}