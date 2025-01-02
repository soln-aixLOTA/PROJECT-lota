use actix_web::{HttpResponse, ResponseError};
use common::CommonError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UserError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("User not found: {0}")]
    NotFound(String),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Authorization error: {0}")]
    Authorization(String),

    #[error("Conflict error: {0}")]
    Conflict(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<UserError> for CommonError {
    fn from(error: UserError) -> Self {
        match error {
            UserError::Database(e) => CommonError::Database(e.to_string()),
            UserError::Validation(e) => CommonError::Validation(e),
            UserError::NotFound(e) => CommonError::NotFound(e),
            UserError::Authentication(e) => CommonError::Authentication(e),
            UserError::Authorization(e) => CommonError::Authorization(e),
            UserError::Internal(e) => CommonError::Internal(e),
            UserError::Conflict(e) => CommonError::Internal(e),
        }
    }
}

impl ResponseError for UserError {
    fn error_response(&self) -> HttpResponse {
        let error: CommonError = self.clone().into();
        match self {
            UserError::Database(_) => HttpResponse::InternalServerError().json(error),
            UserError::Validation(_) => HttpResponse::BadRequest().json(error),
            UserError::NotFound(_) => HttpResponse::NotFound().json(error),
            UserError::Authentication(_) => HttpResponse::Unauthorized().json(error),
            UserError::Authorization(_) => HttpResponse::Forbidden().json(error),
            UserError::Conflict(_) => HttpResponse::Conflict().json(error),
            UserError::Internal(_) => HttpResponse::InternalServerError().json(error),
        }
    }
}

pub type Result<T> = std::result::Result<T, UserError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_conversion() {
        let error = UserError::Authentication("Invalid credentials".to_string());
        let common_error: CommonError = error.into();
        match common_error {
            CommonError::Authentication(msg) => {
                assert_eq!(msg, "Invalid credentials");
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_error_response() {
        let error = UserError::NotFound("User not found".to_string());
        let response = error.error_response();
        assert_eq!(response.status(), 404);
    }
}
