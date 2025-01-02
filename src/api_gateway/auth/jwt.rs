use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tracing::{error, info};

lazy_static::lazy_static! {
    static ref JWT_SECRET: Vec<u8> = std::env::var("JWT_SECRET")
        .expect("JWT_SECRET must be set")
        .into_bytes();
    static ref ACCESS_TOKEN_EXPIRY: i64 = std::env::var("JWT_ACCESS_TOKEN_EXPIRY")
        .unwrap_or_else(|_| "3600".to_string())
        .parse()
        .unwrap_or(3600);
    static ref REFRESH_TOKEN_EXPIRY: i64 = std::env::var("JWT_REFRESH_TOKEN_EXPIRY")
        .unwrap_or_else(|_| "2592000".to_string())
        .parse()
        .unwrap_or(2592000);
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
    pub jti: String, // JWT ID for token tracking
    pub token_type: TokenType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum TokenType {
    Access,
    Refresh,
}

#[derive(Error, Debug)]
pub enum JwtError {
    #[error("Failed to create token")]
    TokenCreation,
    #[error("Failed to validate token")]
    TokenValidation,
    #[error("Token expired")]
    TokenExpired,
    #[error("Invalid token type")]
    InvalidTokenType,
    #[error("Token blacklisted")]
    TokenBlacklisted,
}

pub fn create_jwt(user_id: String, roles: Option<Vec<String>>) -> Result<String, JwtError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let claims = Claims {
        sub: user_id,
        exp: now + *ACCESS_TOKEN_EXPIRY,
        iat: now,
        jti: uuid::Uuid::new_v4().to_string(),
        token_type: TokenType::Access,
        roles,
    };

    let header = Header {
        kid: Some("current-key".to_string()),
        ..Header::default()
    };

    encode(&header, &claims, &EncodingKey::from_secret(&JWT_SECRET)).map_err(|e| {
        error!("Token creation failed: {}", e);
        JwtError::TokenCreation
    })
}

pub fn create_jwt_with_refresh(
    user_id: String,
    roles: Option<Vec<String>>,
) -> Result<(String, String, i64), JwtError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    // Create access token
    let access_claims = Claims {
        sub: user_id.clone(),
        exp: now + *ACCESS_TOKEN_EXPIRY,
        iat: now,
        jti: uuid::Uuid::new_v4().to_string(),
        token_type: TokenType::Access,
        roles: roles.clone(),
    };

    let header = Header {
        kid: Some("current-key".to_string()),
        ..Header::default()
    };

    let access_token = encode(
        &header,
        &access_claims,
        &EncodingKey::from_secret(&JWT_SECRET),
    )
    .map_err(|e| {
        error!("Access token creation failed: {}", e);
        JwtError::TokenCreation
    })?;

    // Create refresh token
    let refresh_claims = Claims {
        sub: user_id,
        exp: now + *REFRESH_TOKEN_EXPIRY,
        iat: now,
        jti: uuid::Uuid::new_v4().to_string(),
        token_type: TokenType::Refresh,
        roles,
    };

    let refresh_token = encode(
        &header,
        &refresh_claims,
        &EncodingKey::from_secret(&JWT_SECRET),
    )
    .map_err(|e| {
        error!("Refresh token creation failed: {}", e);
        JwtError::TokenCreation
    })?;

    Ok((access_token, refresh_token, *ACCESS_TOKEN_EXPIRY))
}

pub fn validate_jwt(token: &str) -> Result<(String, Option<Vec<String>>), JwtError> {
    let mut validation = Validation::default();
    validation.validate_exp = true;
    validation.validate_nbf = true;
    validation.set_required_spec_claims(&["exp", "iat", "sub", "jti"]);

    let token_data = decode::<Claims>(token, &DecodingKey::from_secret(&JWT_SECRET), &validation)
        .map_err(|e| {
        error!("Token validation failed: {}", e);
        JwtError::TokenValidation
    })?;

    // Check if token is blacklisted
    if is_token_blacklisted(&token_data.claims.jti) {
        return Err(JwtError::TokenBlacklisted);
    }

    match token_data.claims.token_type {
        TokenType::Access => Ok((token_data.claims.sub, token_data.claims.roles)),
        TokenType::Refresh => Err(JwtError::InvalidTokenType),
    }
}

pub fn refresh_jwt(refresh_token: &str) -> Result<(String, String, i64), JwtError> {
    let mut validation = Validation::default();
    validation.validate_exp = true;
    validation.validate_nbf = true;
    validation.set_required_spec_claims(&["exp", "iat", "sub", "jti"]);

    let token_data = decode::<Claims>(
        refresh_token,
        &DecodingKey::from_secret(&JWT_SECRET),
        &validation,
    )
    .map_err(|e| {
        error!("Refresh token validation failed: {}", e);
        JwtError::TokenValidation
    })?;

    // Check if token is blacklisted
    if is_token_blacklisted(&token_data.claims.jti) {
        return Err(JwtError::TokenBlacklisted);
    }

    match token_data.claims.token_type {
        TokenType::Refresh => {
            create_jwt_with_refresh(token_data.claims.sub, token_data.claims.roles)
        }
        TokenType::Access => Err(JwtError::InvalidTokenType),
    }
}

// Token blacklisting (should be implemented with Redis in production)
fn is_token_blacklisted(jti: &str) -> bool {
    // TODO: Implement Redis-based token blacklist
    false
}
