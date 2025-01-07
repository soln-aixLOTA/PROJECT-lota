use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::env;
use uuid::Uuid;

use crate::core::error::{AppError, AppResult};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: Uuid,
    pub exp: usize,
    pub username: String,
}

impl Claims {
    pub fn new(user_id: Uuid, username: String) -> Self {
        let exp = (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize;
        Self {
            sub: user_id,
            exp,
            username,
        }
    }

    pub fn to_token(&self) -> AppResult<String> {
        let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        encode(
            &Header::default(),
            self,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .map_err(|e| AppError::Auth(format!("Failed to create token: {}", e)))
    }

    pub fn from_token(token: &str) -> AppResult<Self> {
        let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|e| AppError::Auth(format!("Invalid token: {}", e)))?;

        Ok(token_data.claims)
    }
}

pub fn hash_password(password: &str) -> AppResult<String> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST)
        .map_err(|e| AppError::Internal(format!("Failed to hash password: {}", e)))
}

pub fn verify_password(password: &str, hash: &str) -> AppResult<bool> {
    bcrypt::verify(password, hash)
        .map_err(|e| AppError::Internal(format!("Failed to verify password: {}", e)))
}
