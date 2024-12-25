use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde::Serialize;
use sqlx::Error as SqlxError;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] SqlxError),

    #[error("Tenant not found: {0}")]
    TenantNotFound(Uuid),

    #[error("Tenant is deleted: {0}")]
    TenantDeleted(Uuid),

    #[error("Tenant is inactive: {0}")]
    TenantInactive(Uuid),

    #[error("Domain already exists: {0}")]
    DomainAlreadyExists(String),

    #[error("User not found: {0}")]
    UserNotFound(Uuid),

    #[error("User already exists with email: {0}")]
    UserAlreadyExists(String),

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Password too weak")]
    WeakPassword,

    #[error("Invalid token")]
    InvalidToken,

    #[error("Token expired")]
    TokenExpired,

    #[error("MFA required")]
    MfaRequired,

    #[error("Invalid MFA code")]
    InvalidMfaCode,

    #[error("Role not found: {0}")]
    RoleNotFound(Uuid),

    #[error("Permission denied")]
    PermissionDenied,

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Internal server error")]
    InternalServerError,
}

#[derive(Serialize)]
struct ErrorResponse {
    code: u16,
    message: String,
}

impl ResponseError for ServiceError {
    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();
        let error_response = ErrorResponse {
            code: status_code.as_u16(),
            message: self.to_string(),
        };

        HttpResponse::build(status_code).json(error_response)
    }

    fn status_code(&self) -> StatusCode {
        match self {
            ServiceError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ServiceError::TenantNotFound(_) => StatusCode::NOT_FOUND,
            ServiceError::TenantDeleted(_) => StatusCode::GONE,
            ServiceError::TenantInactive(_) => StatusCode::FORBIDDEN,
            ServiceError::DomainAlreadyExists(_) => StatusCode::CONFLICT,
            ServiceError::UserNotFound(_) => StatusCode::NOT_FOUND,
            ServiceError::UserAlreadyExists(_) => StatusCode::CONFLICT,
            ServiceError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            ServiceError::WeakPassword => StatusCode::BAD_REQUEST,
            ServiceError::InvalidToken => StatusCode::UNAUTHORIZED,
            ServiceError::TokenExpired => StatusCode::UNAUTHORIZED,
            ServiceError::MfaRequired => StatusCode::UNAUTHORIZED,
            ServiceError::InvalidMfaCode => StatusCode::UNAUTHORIZED,
            ServiceError::RoleNotFound(_) => StatusCode::NOT_FOUND,
            ServiceError::PermissionDenied => StatusCode::FORBIDDEN,
            ServiceError::RateLimitExceeded => StatusCode::TOO_MANY_REQUESTS,
            ServiceError::ResourceLimitExceeded(_) => StatusCode::FORBIDDEN,
            ServiceError::InvalidRequest(_) => StatusCode::BAD_REQUEST,
            ServiceError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
} 