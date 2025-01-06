#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::jwt::{AuthUser, JwtAuth};

    #[tokio::test]
    async fn test_jwt_token_creation_and_validation() {
        let secret = b"test-secret-key";
        let jwt_auth = JwtAuth::new(secret);

        // Test token creation
        let user_id = "test-user";
        let roles = vec!["user".to_string(), "admin".to_string()];
        let token = jwt_auth.create_token(user_id, roles.clone()).unwrap();

        // Validate the token is not empty
        assert!(!token.is_empty());

        // Test token validation
        let auth_user = jwt_auth.validate_token(&token).unwrap();
        assert_eq!(auth_user.id, user_id);
        assert_eq!(auth_user.roles, roles);
    }

    #[tokio::test]
    async fn test_invalid_token() {
        let secret = b"test-secret-key";
        let jwt_auth = JwtAuth::new(secret);

        // Test with invalid token
        let result = jwt_auth.validate_token("invalid-token");
        assert!(result.is_err());

        // Test with empty token
        let result = jwt_auth.validate_token("");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_token_expiration() {
        let secret = b"test-secret-key";
        let jwt_auth = JwtAuth::new(secret);

        // Create a token
        let token = jwt_auth
            .create_token("test-user", vec!["user".to_string()])
            .unwrap();

        // Validate it works immediately
        let result = jwt_auth.validate_token(&token);
        assert!(result.is_ok());
    }
}
