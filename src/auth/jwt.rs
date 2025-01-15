    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::error::DocumentError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: u64,
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

    pub fn create_token(
        &self,
        user_id: &str,
        roles: Vec<String>,
        expiry: u64,
    ) -> Result<String, DocumentError> {
        let claims = Claims {
            sub: user_id.to_string(),
            exp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                + expiry,
            roles,
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| DocumentError::AuthenticationError(e.to_string()))
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims, DocumentError> {
        let validation = Validation::default();
        decode::<Claims>(token, &self.decoding_key, &validation)
            .map(|data| data.claims)
            .map_err(|e| DocumentError::AuthenticationError(e.to_string()))
    }
}

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: String,
    pub roles: Vec<String>,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .ok_or((
                StatusCode::UNAUTHORIZED,
                "Missing Authorization header".to_string(),
            ))?;

        if !auth_header.starts_with("Bearer ") {
            return Err((
                StatusCode::UNAUTHORIZED,
                "Invalid Authorization header format".to_string(),
            ));
        }

        let token = &auth_header[7..];
        let jwt_secret = std::env::var("JWT_SECRET")
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Authentication configuration missing".to_string()))?;
        let jwt_auth = JwtAuth::new(jwt_secret.as_bytes());
        let claims = jwt_auth
            .validate_token(token)
            .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;

        Ok(AuthUser {
            user_id: claims.sub,
            roles: claims.roles,
        })
    }
}
