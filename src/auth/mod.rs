use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use crate::models::user::User;
use crate::error::AppError;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;
use serde::{Deserialize, Serialize};

pub type AppResult<T> = Result<T, AppError>;

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: Uuid,
    pub exp: i64,
    pub iat: i64,
    pub jti: String,
    pub token_type: TokenType,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum TokenType {
    Access,
    Refresh,
}

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
}

pub fn generate_access_token(user: &User) -> AppResult<String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| AppError::Internal(format!("Failed to get system time: {}", e)))?
        .as_secs() as i64;

    let claims = Claims {
        sub: user.id,
        exp: now + *ACCESS_TOKEN_EXPIRY,
        iat: now,
        jti: Uuid::new_v4().to_string(),
        token_type: TokenType::Access,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(&JWT_SECRET),
    )
    .map_err(|e| AppError::Internal(format!("Failed to create token: {}", e)))
}

pub fn generate_refresh_token(user: &User) -> AppResult<String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| AppError::Internal(format!("Failed to get system time: {}", e)))?
        .as_secs() as i64;

    let claims = Claims {
        sub: user.id,
        exp: now + *REFRESH_TOKEN_EXPIRY,
        iat: now,
        jti: Uuid::new_v4().to_string(),
        token_type: TokenType::Refresh,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(&JWT_SECRET),
    )
    .map_err(|e| AppError::Internal(format!("Failed to create token: {}", e)))
}

pub fn validate_access_token(token: &str) -> AppResult<AuthUser> {
    let mut validation = Validation::default();
    validation.validate_exp = true;

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(&JWT_SECRET),
        &validation,
    )
    .map_err(|e| AppError::Authentication(format!("Invalid token: {}", e)))?;

    if token_data.claims.token_type != TokenType::Access {
        return Err(AppError::Authentication("Invalid token type".into()));
    }

    Ok(AuthUser {
        user_id: token_data.claims.sub,
    })
}

pub fn validate_refresh_token(token: &str) -> AppResult<Uuid> {
    let mut validation = Validation::default();
    validation.validate_exp = true;

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(&JWT_SECRET),
        &validation,
    )
    .map_err(|e| AppError::Authentication(format!("Invalid token: {}", e)))?;

    if token_data.claims.token_type != TokenType::Refresh {
        return Err(AppError::Authentication("Invalid token type".into()));
    }

    Ok(token_data.claims.sub)
}
