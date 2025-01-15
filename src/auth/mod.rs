use crate::error::AppError;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use lazy_static::lazy_static;
use log::error;
use serde::{Deserialize, Serialize};
use std::env;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
    pub role: String,
}

impl Claims {
    pub fn new(sub: String, role: String) -> Self {
        Self {
            sub,
            exp: (Utc::now() + Duration::hours(1)).timestamp(),
            role,
        }
    }
}

lazy_static! {
    static ref JWT_KEY: Vec<u8> = std::env::var("JWT_KEY")
        .expect("JWT_KEY must be set")
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

pub fn create_access_token(claims: Claims) -> AppResult<String> {
    let jwt_secret = env::var("JWT_SECRET").map_err(|_| {
        error!("JWT_SECRET not set");
        AppError::Internal("JWT configuration error".to_string())
    })?;

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("Failed to create access token: {}", e)))
}

pub fn create_refresh_token(claims: Claims) -> AppResult<String> {
    let jwt_secret = env::var("JWT_SECRET").map_err(|_| {
        error!("JWT_SECRET not set");
        AppError::Internal("JWT configuration error".to_string())
    })?;

    let mut refresh_claims = claims;
    refresh_claims.exp = (Utc::now() + Duration::days(7)).timestamp();

    encode(
        &Header::default(),
        &refresh_claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("Failed to create refresh token: {}", e)))
}

pub fn validate_token(token: &str) -> AppResult<Claims> {
    let jwt_key = env::var("JWT_KEY").map_err(|_| {
        error!("JWT_KEY not set");
        AppError::Internal("JWT configuration error".to_string())
    })?;

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_key.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| AppError::Authentication(format!("Invalid token: {}", e)))?;

    Ok(token_data.claims)
}

pub fn get_user_id_from_token(token: &str) -> AppResult<String> {
    let mut validation = Validation::default();
    validation.validate_exp = true;

    let token_data = decode::<Claims>(token, &DecodingKey::from_secret(&JWT_KEY), &validation)
        .map_err(|e| AppError::Authentication(format!("Invalid token: {}", e)))?;

    Ok(token_data.claims.sub)
}
