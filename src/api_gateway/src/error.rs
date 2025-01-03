use actix_web::{
    error::ResponseError,
    http::{header::ContentType, StatusCode},
    HttpResponse,
};
use derive_more::{Display, Error};
use serde::Serialize;
use serde_json::json;
use std::fmt;
use tracing::error;

#[derive(Debug, Display, Error)]
pub enum ApiError {
    #[display(fmt = "Authentication error: {}", _0)]
    AuthenticationError(String),

    #[display(fmt = "Authorization error: {}", _0)]
    AuthorizationError(String),

    #[display(fmt = "Validation error: {}", _0)]
    ValidationError(String),

    #[display(fmt = "Rate limit error: {}", _0)]
    RateLimitError(String),

    #[display(fmt = "Internal server error")]
    InternalServerError,

    #[display(fmt = "Bad request: {}", _0)]
    BadRequest(String),

    #[display(fmt = "Not found: {}", _0)]
    NotFound(String),

    #[display(fmt = "Service error: {}", _0)]
    ServiceError(String),

    #[display(fmt = "Database error: {}", _0)]
    DatabaseError(String),

    #[display(fmt = "External service error: {}", _0)]
    ExternalServiceError(String),
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();
        let error_response = ErrorResponse {
            error: self.error_type(),
            message: self.to_string(),
            details: self.error_details(),
        };

        error!(
            error.type = %error_response.error,
            error.message = %error_response.message,
            error.status_code = %status_code.as_u16(),
            "API error occurred"
        );

        HttpResponse::build(status_code)
            .insert_header(ContentType::json())
            .json(error_response)
    }

    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::AuthenticationError(_) => StatusCode::UNAUTHORIZED,
            ApiError::AuthorizationError(_) => StatusCode::FORBIDDEN,
            ApiError::ValidationError(_) => StatusCode::BAD_REQUEST,
            ApiError::RateLimitError(_) => StatusCode::TOO_MANY_REQUESTS,
            ApiError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::ServiceError(_) => StatusCode::SERVICE_UNAVAILABLE,
            ApiError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::ExternalServiceError(_) => StatusCode::BAD_GATEWAY,
        }
    }
}

impl ApiError {
    fn error_type(&self) -> String {
        match self {
            ApiError::AuthenticationError(_) => "authentication_error",
            ApiError::AuthorizationError(_) => "authorization_error",
            ApiError::ValidationError(_) => "validation_error",
            ApiError::RateLimitError(_) => "rate_limit_error",
            ApiError::InternalServerError => "internal_server_error",
            ApiError::BadRequest(_) => "bad_request",
            ApiError::NotFound(_) => "not_found",
            ApiError::ServiceError(_) => "service_error",
            ApiError::DatabaseError(_) => "database_error",
            ApiError::ExternalServiceError(_) => "external_service_error",
        }
        .to_string()
    }

    fn error_details(&self) -> Option<serde_json::Value> {
        match self {
            ApiError::ValidationError(msg) => Some(json!({
                "validation_errors": [msg]
            })),
            ApiError::RateLimitError(msg) => Some(json!({
                "retry_after": 60, // Example value
                "limit_info": msg
            })),
            ApiError::ServiceError(msg) => Some(json!({
                "service_info": msg
            })),
            ApiError::ExternalServiceError(msg) => Some(json!({
                "service_info": msg
            })),
            _ => None,
        }
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(error: sqlx::Error) -> Self {
        error!(?error, "Database error occurred");
        ApiError::DatabaseError(error.to_string())
    }
}

impl From<reqwest::Error> for ApiError {
    fn from(error: reqwest::Error) -> Self {
        error!(?error, "External service error occurred");
        ApiError::ExternalServiceError(error.to_string())
    }
}

impl From<std::io::Error> for ApiError {
    fn from(error: std::io::Error) -> Self {
        error!(?error, "IO error occurred");
        ApiError::InternalServerError
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(error: serde_json::Error) -> Self {
        error!(?error, "JSON serialization error occurred");
        ApiError::BadRequest(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test;

    #[test]
    fn test_error_response() {
        let error = ApiError::ValidationError("Invalid input".to_string());
        let resp = error.error_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        let error = ApiError::AuthenticationError("Invalid token".to_string());
        let resp = error.error_response();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

        let error = ApiError::RateLimitError("Too many requests".to_string());
        let resp = error.error_response();
        assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
    }

    #[test]
    fn test_error_conversion() {
        let io_error = std::io::Error::new(std::io::ErrorKind::Other, "IO error");
        let api_error: ApiError = io_error.into();
        assert!(matches!(api_error, ApiError::InternalServerError));

        let json_error =
            serde_json::Error::syntax(serde_json::error::ErrorCode::ExpectedColon, 0, 0);
        let api_error: ApiError = json_error.into();
        assert!(matches!(api_error, ApiError::BadRequest(_)));
    }
}
