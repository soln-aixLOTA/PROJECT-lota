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
        let status = self.status_code();
        let error_response = ErrorResponse {
            error: self.error_type(),
            message: self.to_string(),
            details: self.error_details(),
        };

        HttpResponse::build(status)
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
            ApiError::AuthenticationError(_) => "AUTHENTICATION_ERROR",
            ApiError::AuthorizationError(_) => "AUTHORIZATION_ERROR",
            ApiError::ValidationError(_) => "VALIDATION_ERROR",
            ApiError::RateLimitError(_) => "RATE_LIMIT_ERROR",
            ApiError::InternalServerError => "INTERNAL_SERVER_ERROR",
            ApiError::BadRequest(_) => "BAD_REQUEST",
            ApiError::NotFound(_) => "NOT_FOUND",
            ApiError::ServiceError(_) => "SERVICE_ERROR",
            ApiError::DatabaseError(_) => "DATABASE_ERROR",
            ApiError::ExternalServiceError(_) => "EXTERNAL_SERVICE_ERROR",
        }
        .to_string()
    }

    fn error_details(&self) -> Option<serde_json::Value> {
        match self {
            ApiError::ValidationError(msg) => Some(json!({
                "validation_message": msg,
                "error_code": "400"
            })),
            ApiError::DatabaseError(msg) => Some(json!({
                "database_message": msg,
                "error_code": "500"
            })),
            ApiError::ExternalServiceError(msg) => Some(json!({
                "service_message": msg,
                "error_code": "502"
            })),
            ApiError::RateLimitError(msg) => Some(json!({
                "rate_limit_info": msg,
                "error_code": "429"
            })),
            ApiError::AuthenticationError(msg) => Some(json!({
                "auth_error": msg,
                "error_code": "401"
            })),
            ApiError::AuthorizationError(msg) => Some(json!({
                "permission_error": msg,
                "error_code": "403"
            })),
            ApiError::ServiceError(msg) => Some(json!({
                "service_status": msg,
                "error_code": "503"
            })),
            ApiError::NotFound(msg) => Some(json!({
                "resource": msg,
                "error_code": "404"
            })),
            ApiError::BadRequest(msg) => Some(json!({
                "request_error": msg,
                "error_code": "400"
            })),
            ApiError::InternalServerError => Some(json!({
                "error_code": "500"
            })),
        }
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(error: sqlx::Error) -> Self {
        error!(?error, "Database error occurred");
        match error {
            sqlx::Error::RowNotFound => ApiError::NotFound("Resource not found".to_string()),
            _ => ApiError::DatabaseError(error.to_string()),
        }
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
    use serde_json::Value;

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

        let db_error = sqlx::Error::RowNotFound;
        let api_error: ApiError = db_error.into();
        assert!(matches!(api_error, ApiError::NotFound(_)));
    }

    #[test]
    fn test_error_details() {
        // Test validation error details
        let error = ApiError::ValidationError("Invalid email".to_string());
        let details = error.error_details().unwrap();
        assert_eq!(details["error_code"], "400");
        assert_eq!(details["validation_message"], "Invalid email");

        // Test rate limit error details
        let error = ApiError::RateLimitError("Rate limit exceeded".to_string());
        let details = error.error_details().unwrap();
        assert_eq!(details["error_code"], "429");
        assert_eq!(details["rate_limit_info"], "Rate limit exceeded");

        // Test database error details
        let error = ApiError::DatabaseError("Connection failed".to_string());
        let details = error.error_details().unwrap();
        assert_eq!(details["error_code"], "500");
        assert_eq!(details["database_message"], "Connection failed");
    }

    #[test]
    fn test_error_response_format() {
        // Test complete error response format
        let error = ApiError::ValidationError("test error".to_string());
        let response = error.error_response();

        // Convert the response body to a Value for easy testing
        let body = test::read_body(response);
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        let json: Value = serde_json::from_str(&body_str).unwrap();

        assert_eq!(json["error"], "VALIDATION_ERROR");
        assert_eq!(json["message"], "Validation error: test error");
        assert_eq!(json["details"]["validation_message"], "test error");
        assert_eq!(json["details"]["error_code"], "400");
    }

    #[test]
    fn test_status_codes() {
        let test_cases = vec![
            (
                ApiError::AuthenticationError("test".into()),
                StatusCode::UNAUTHORIZED,
            ),
            (
                ApiError::AuthorizationError("test".into()),
                StatusCode::FORBIDDEN,
            ),
            (
                ApiError::ValidationError("test".into()),
                StatusCode::BAD_REQUEST,
            ),
            (
                ApiError::RateLimitError("test".into()),
                StatusCode::TOO_MANY_REQUESTS,
            ),
            (
                ApiError::InternalServerError,
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            (ApiError::BadRequest("test".into()), StatusCode::BAD_REQUEST),
            (ApiError::NotFound("test".into()), StatusCode::NOT_FOUND),
            (
                ApiError::ServiceError("test".into()),
                StatusCode::SERVICE_UNAVAILABLE,
            ),
            (
                ApiError::DatabaseError("test".into()),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            (
                ApiError::ExternalServiceError("test".into()),
                StatusCode::BAD_GATEWAY,
            ),
        ];

        for (error, expected_status) in test_cases {
            assert_eq!(error.status_code(), expected_status);
        }
    }

    #[test]
    fn test_error_type_strings() {
        let test_cases = vec![
            (
                ApiError::AuthenticationError("test".into()),
                "AUTHENTICATION_ERROR",
            ),
            (
                ApiError::AuthorizationError("test".into()),
                "AUTHORIZATION_ERROR",
            ),
            (ApiError::ValidationError("test".into()), "VALIDATION_ERROR"),
            (ApiError::RateLimitError("test".into()), "RATE_LIMIT_ERROR"),
            (ApiError::InternalServerError, "INTERNAL_SERVER_ERROR"),
            (ApiError::BadRequest("test".into()), "BAD_REQUEST"),
            (ApiError::NotFound("test".into()), "NOT_FOUND"),
            (ApiError::ServiceError("test".into()), "SERVICE_ERROR"),
            (ApiError::DatabaseError("test".into()), "DATABASE_ERROR"),
            (
                ApiError::ExternalServiceError("test".into()),
                "EXTERNAL_SERVICE_ERROR",
            ),
        ];

        for (error, expected_type) in test_cases {
            assert_eq!(error.error_type(), expected_type);
        }
    }
}
