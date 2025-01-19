use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,      // Subject (user_id)
    pub exp: usize,       // Expiration time
    pub iat: usize,       // Issued at
}

pub fn create_token(user_id: &str, key: &[u8]) -> Result<String, jsonwebtoken::errors::Error> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        exp: now + 24 * 3600,  // Token expires in 24 hours
        iat: now,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(key),
    )
}

pub fn validate_token(token: &str, key: &[u8]) -> Result<Claims, jsonwebtoken::errors::Error> {
    let mut validation = Validation::default();
    validation.validate_exp = true;  // Enable expiration validation
    validation.leeway = 0;  // No leeway for expiration time

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(key),
        &validation,
    )?;

    Ok(token_data.claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_key() -> Vec<u8> {
        // In a real test environment, this would be set via environment variables
        // For unit tests, we use a constant test key
        b"TEST_KEY_DO_NOT_USE_IN_PRODUCTION".to_vec()
    }

    #[test]
    fn test_token_creation_and_validation() {
        let user_id = "test_user";
        let test_key = get_test_key();

        // Create token
        let token = create_token(user_id, &test_key).expect("Failed to create token");
        assert!(!token.is_empty(), "Token should not be empty");

        // Validate token
        let claims = validate_token(&token, &test_key).expect("Failed to validate token");
        assert_eq!(claims.sub, user_id, "User ID in claims should match original");
        assert!(claims.exp > claims.iat, "Expiration should be after issued time");
    }

    #[test]
    fn test_token_expiration() {
        use std::thread::sleep;
        use std::time::Duration;

        let user_id = "test_user";
        let test_key = get_test_key();

        // Create token
        let token = create_token(user_id, &test_key).expect("Failed to create token");

        // Validate token
        let claims = validate_token(&token, &test_key).expect("Failed to validate token");
        assert_eq!(claims.sub, user_id);

        // Sleep for a moment to test expiration
        sleep(Duration::from_secs(1));

        // Token should still be valid
        let result = validate_token(&token, &test_key);
        assert!(result.is_ok(), "Token should still be valid");
    }

    #[test]
    fn test_invalid_token() {
        let user_id = "test_user";
        let test_key = get_test_key();
        let wrong_key = b"wrong_key".to_vec();

        // Create token with correct key
        let token = create_token(user_id, &test_key).expect("Failed to create token");

        // Validate with wrong key should fail
        let result = validate_token(&token, &wrong_key);
        assert!(result.is_err(), "Validation with wrong key should fail");

        // Validate malformed token should fail
        let result = validate_token("invalid.token.format", &test_key);
        assert!(result.is_err(), "Validation of malformed token should fail");
    }
}
