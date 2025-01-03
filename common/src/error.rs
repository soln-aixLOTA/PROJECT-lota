use thiserror::Error;

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

impl Error {
    pub fn internal<T: ToString>(msg: T) -> Self {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = Error::internal("test error");
        assert!(matches!(err, Error::Internal(_)));

        let err = Error::bad_request("test error");
        assert!(matches!(err, Error::BadRequest(_)));

        let err = Error::not_found("test error");
        assert!(matches!(err, Error::NotFound(_)));

        let err = Error::unauthorized("test error");
        assert!(matches!(err, Error::Unauthorized(_)));

        let err = Error::forbidden("test error");
        assert!(matches!(err, Error::Forbidden(_)));

        let err = Error::conflict("test error");
        assert!(matches!(err, Error::Conflict(_)));
    }

    #[test]
    fn test_error_display() {
        let err = Error::internal("test error");
        assert_eq!(err.to_string(), "Internal server error: test error");

        let err = Error::bad_request("test error");
        assert_eq!(err.to_string(), "Invalid request: test error");

        let err = Error::not_found("test error");
        assert_eq!(err.to_string(), "Not found: test error");

        let err = Error::unauthorized("test error");
        assert_eq!(err.to_string(), "Unauthorized: test error");

        let err = Error::forbidden("test error");
        assert_eq!(err.to_string(), "Forbidden: test error");

        let err = Error::conflict("test error");
        assert_eq!(err.to_string(), "Conflict: test error");
    }
}
