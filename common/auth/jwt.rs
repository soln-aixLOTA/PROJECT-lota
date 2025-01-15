use crate::errors::ApiError;
use anyhow::{anyhow, Result};
use jsonwebtoken::{decode, encode, errors::Error, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::env;
use tracing::{error, instrument};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

#[instrument]
pub fn create_jwt(user_id: String) -> Result<String, ApiError> {
    let key = env::var("JWT_SECRET").map_err(|_| {
        error!("JWT_SECRET not set");
        ApiError::JwtError(
            jsonwebtoken::errors::Error::from(jsonwebtoken::errors::ErrorKind::InvalidToken).into(),
        )
    })?;
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::minutes(60))
        .ok_or_else(|| {
            error!("Invalid expiration time");
            ApiError::JwtError(
                jsonwebtoken::errors::Error::from(jsonwebtoken::errors::ErrorKind::InvalidToken)
                    .into(),
            )
        })?
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id,
        exp: expiration,
    };

    let header = Header::default();
    let encoding_key = EncodingKey::from_secret(key.as_bytes());
    encode(&header, &claims, &encoding_key).map_err(|e| {
        error!("Failed to create JWT: {}", e);
        ApiError::from(e)
    })
}

#[instrument]
pub fn validate_jwt(token: &str) -> Result<String, ApiError> {
    let key = env::var("JWT_SECRET").map_err(|_| {
        error!("JWT_SECRET not set");
        ApiError::JwtError(
            jsonwebtoken::errors::Error::from(jsonwebtoken::errors::ErrorKind::InvalidToken).into(),
        )
    })?;
    let decoding_key = DecodingKey::from_secret(key.as_bytes());
    let validation = Validation::default();
    decode::<Claims>(token, &decoding_key, &validation)
        .map_err(|e| {
            error!("Failed to validate JWT: {}", e);
            ApiError::from(e)
        })
        .map(|decoded| decoded.claims.sub)
}
