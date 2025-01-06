use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::core::error::DocumentError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
    pub roles: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: String,
    pub roles: Vec<String>,
}

pub struct JwtAuth {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl JwtAuth {
    pub fn new(secret: &[u8]) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret),
            decoding_key: DecodingKey::from_secret(secret),
        }
    }

    pub fn create_token(&self, user_id: &str, roles: Vec<String>) -> Result<String, DocumentError> {
        let now = Utc::now();
        let exp = now + Duration::hours(24);

        let claims = Claims {
            sub: user_id.to_string(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            roles,
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| DocumentError::Auth(e.to_string()))
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims, DocumentError> {
        let validation = Validation::default();
        decode::<Claims>(token, &self.decoding_key, &validation)
            .map(|data| data.claims)
            .map_err(|e| DocumentError::Auth(e.to_string()))
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|value| value.to_str().ok());

        let token = match auth_header {
            Some(value) if value.starts_with("Bearer ") => value[7..].to_string(),
            _ => {
                return Err((
                    StatusCode::UNAUTHORIZED,
                    Json(json!({"error": "Missing or invalid authorization header"})),
                )
                    .into_response())
            }
        };

        let jwt = JwtAuth::new(b"your-secret-key");
        let claims = match jwt.validate_token(&token) {
            Ok(claims) => claims,
            Err(_) => {
                return Err((
                    StatusCode::UNAUTHORIZED,
                    Json(json!({"error": "Invalid token"})),
                )
                    .into_response())
            }
        };

        Ok(AuthUser {
            id: claims.sub,
            roles: claims.roles,
        })
    }
}
