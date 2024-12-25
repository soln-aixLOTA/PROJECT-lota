use async_trait::async_trait;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
    pub tenant_id: String,
    pub roles: Vec<String>,
}

#[async_trait]
pub trait AuthenticationService {
    async fn validate_token(&self, token: &str) -> Result<Claims, LotaBotsError>;
    async fn check_permissions(&self, claims: &Claims, required_permissions: &[String]) -> Result<bool, LotaBotsError>;
}

pub struct AuthMiddleware<T: AuthenticationService> {
    auth_service: T,
}

impl<T: AuthenticationService> AuthMiddleware<T> {
    pub fn new(auth_service: T) -> Self {
        Self { auth_service }
    }
    
    pub async fn authenticate(&self, token: &str) -> Result<Claims, LotaBotsError> {
        let claims = self.auth_service.validate_token(token).await?;
        
        info!(
            user_id = claims.sub,
            tenant_id = claims.tenant_id,
            "User authenticated successfully"
        );
        
        Ok(claims)
    }
}
