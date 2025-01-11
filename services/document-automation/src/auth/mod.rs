use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,      // Subject (user_id)
    pub exp: usize,       // Expiration time
    pub iat: usize,       // Issued at
}

pub fn create_token(user_id: &str, secret: &[u8]) -> Result<String, jsonwebtoken::errors::Error> {
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
        &EncodingKey::from_secret(secret),
    )
}

pub fn validate_token(token: &str, secret: &[u8]) -> Result<Claims, jsonwebtoken::errors::Error> {
    let mut validation = Validation::default();
    validation.validate_exp = true;  // Enable expiration validation
    validation.leeway = 0;  // No leeway for expiration time
    
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret),
        &validation,
    )?;

    Ok(token_data.claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_creation_and_validation() {
        let secret = b"your-256-bit-secret";
        let user_id = "test_user";

        // Create token
        let token = create_token(user_id, secret).expect("Failed to create token");
        assert!(!token.is_empty(), "Token should not be empty");

        // Validate token
        let claims = validate_token(&token, secret).expect("Failed to validate token");
        assert_eq!(claims.sub, user_id, "User ID in claims should match original");
        assert!(claims.exp > claims.iat, "Expiration should be after issued time");
    }

    #[test]
    fn test_token_expiration() {
        use std::thread::sleep;
        use std::time::Duration;

        let secret = b"your-256-bit-secret";
        let user_id = "test_user";

        // Create a token that expires in 1 second
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;

        let claims = Claims {
            sub: user_id.to_string(),
            exp: now + 1,  // Expires in 1 second
            iat: now,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret),
        ).expect("Failed to create token");

        // Token should be valid immediately
        let result = validate_token(&token, secret);
        assert!(result.is_ok(), "Token should be valid initially");

        // Wait for token to expire
        sleep(Duration::from_secs(2));

        // Token should be invalid after expiration
        let result = validate_token(&token, secret);
        assert!(result.is_err(), "Token should be invalid after expiration");
    }

    #[test]
    fn test_invalid_token() {
        let secret = b"your-256-bit-secret";
        let wrong_secret = b"wrong-secret";
        let user_id = "test_user";

        // Create token with correct secret
        let token = create_token(user_id, secret).expect("Failed to create token");

        // Validate with wrong secret should fail
        let result = validate_token(&token, wrong_secret);
        assert!(result.is_err(), "Validation with wrong secret should fail");

        // Validate malformed token should fail
        let result = validate_token("invalid.token.format", secret);
        assert!(result.is_err(), "Validation of malformed token should fail");
    }
} 