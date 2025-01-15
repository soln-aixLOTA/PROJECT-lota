use openssl::rand::rand_bytes;
use std::env;
use crate::error::ApiError;

const SECRET_LENGTH: usize = 32;

pub fn get_jwt_secret() -> Result<String, ApiError> {
    env::var("JWT_SECRET").map_err(|_| {
        ApiError::InternalError("JWT_SECRET environment variable must be set".to_string())
    })
}

pub fn generate_secret() -> Result<String, ApiError> {
    let mut buf = vec![0; SECRET_LENGTH];
    rand_bytes(&mut buf).map_err(|e| {
        ApiError::InternalError(format!("Failed to generate secure secret: {}", e))
    })?;
    Ok(hex::encode(buf))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_secret() {
        let secret = generate_secret().unwrap();
        assert_eq!(secret.len(), SECRET_LENGTH * 2); // hex encoding doubles length
        assert!(secret.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
