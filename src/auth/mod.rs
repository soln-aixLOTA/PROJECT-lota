pub mod jwt;
pub mod mfa;

use std::env;
use crate::error::AppError;
use crate::models::user::User;
use jwt::JwtAuth;

pub fn generate_access_token(user: &User) -> Result<String, AppError> {
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let jwt_auth = JwtAuth::new(jwt_secret.as_bytes());
    let roles = vec![user.role.to_string()];
    jwt_auth.create_token(&user.id.to_string(), roles, 3600) // 1 hour expiry
        .map_err(|e| AppError::Internal(format!("Failed to generate access token: {}", e)))
}

pub fn generate_refresh_token(user: &User) -> Result<String, AppError> {
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let jwt_auth = JwtAuth::new(jwt_secret.as_bytes());
    let roles = vec![user.role.to_string()];
    jwt_auth.create_token(&user.id.to_string(), roles, 604800) // 1 week expiry
        .map_err(|e| AppError::Internal(format!("Failed to generate refresh token: {}", e)))
}

pub fn validate_refresh_token(token: &str) -> Result<String, AppError> {
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let jwt_auth = JwtAuth::new(jwt_secret.as_bytes());
    let claims = jwt_auth.validate_token(token)
        .map_err(|e| AppError::Authentication(format!("Invalid refresh token: {}", e)))?;
    Ok(claims.sub)
}
