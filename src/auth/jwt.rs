use actix_web::{
    dev::Payload,
    FromRequest,
    HttpRequest,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use std::future::Future;
use std::pin::Pin;
use uuid::Uuid;

use crate::error::AppError;

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
    ) -> Result<String, AppError> {
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
            .map_err(|e| AppError::Internal(format!("Failed to create token: {}", e)))
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims, AppError> {
        let mut validation = Validation::default();
        validation.validate_exp = true;
        validation.leeway = 60;

        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature =>
                    AppError::Authentication("Token has expired".into()),
                _ => AppError::Authentication(format!("Invalid token: {}", e))
            })?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if token_data.claims.exp < now {
            return Err(AppError::Authentication("Token has expired".into()));
        }

        Ok(token_data.claims)
    }

    pub fn create_access_token(
        &self,
        user_id: &str,
        roles: Vec<String>,
    ) -> Result<String, AppError> {
        self.create_token(user_id, roles, 900)
    }

    pub fn create_refresh_token(
        &self,
        user_id: &str,
        roles: Vec<String>,
    ) -> Result<String, AppError> {
        self.create_token(user_id, roles, 604800)
    }
}

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: String,
    pub roles: Vec<String>,
}

impl AuthUser {
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r.eq_ignore_ascii_case(role))
    }

    pub fn is_admin(&self) -> bool {
        self.has_role("admin")
    }

    pub fn can_access_document(&self, document_user_id: &str) -> bool {
        self.is_admin() || self.user_id == document_user_id
    }
}

impl FromRequest for AuthUser {
    type Error = AppError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let auth_header = req.headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string());

        let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");

        Box::pin(async move {
            let auth_header = auth_header.ok_or_else(|| {
                AppError::Authentication("Missing Authorization header".into())
            })?;

            if !auth_header.starts_with("Bearer ") {
                return Err(AppError::Authentication("Invalid Authorization header format".into()));
            }

            let token = &auth_header[7..];
            let jwt_auth = JwtAuth::new(jwt_secret.as_bytes());
            let claims = jwt_auth.validate_token(token)?;

            Ok(AuthUser {
                user_id: claims.sub,
                roles: claims.roles,
            })
        })
    }
}
