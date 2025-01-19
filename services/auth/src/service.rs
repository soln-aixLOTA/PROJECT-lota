use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;
use validator::Validate;

use crate::{
    repository::AuthRepository,
    LoginRequest, RegisterRequest, User,
};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

#[derive(Clone)]
pub struct AuthService {
    repository: AuthRepository,
    jwt_secret: String,
}

impl AuthService {
    pub fn new(repository: AuthRepository, jwt_secret: String) -> Self {
        Self {
            repository,
            jwt_secret,
        }
    }

    pub async fn register(&self, req: RegisterRequest) -> Result<String, String> {
        // Validate request
        if let Err(e) = req.validate() {
            return Err(format!("Validation error: {}", e));
        }

        // Hash password
        let password_hash = hash(req.password.as_bytes(), DEFAULT_COST)
            .map_err(|e| format!("Password hashing error: {}", e))?;

        // Create user
        let user = User {
            id: Uuid::new_v4(),
            username: req.username,
            email: req.email,
            password_hash,
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
        };

        let user_id = user.id;

        // Save user
        self.repository
            .create_user(user)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        // Generate JWT
        self.generate_token(user_id)
    }

    pub async fn login(&self, req: LoginRequest) -> Result<String, String> {
        // Get user
        let user = self
            .repository
            .find_user_by_username(&req.username)
            .await
            .map_err(|e| format!("Database error: {}", e))?
            .ok_or_else(|| "Invalid credentials".to_string())?;

        // Verify password
        if !verify(req.password.as_bytes(), &user.password_hash)
            .map_err(|e| format!("Password verification error: {}", e))? {
            return Err("Invalid credentials".to_string());
        }

        // Generate JWT
        self.generate_token(user.id)
    }

    fn generate_token(&self, user_id: Uuid) -> Result<String, String> {
        let claims = Claims {
            sub: user_id.to_string(),
            exp: (OffsetDateTime::now_utc() + time::Duration::hours(24)).unix_timestamp() as usize,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|e| format!("Token generation error: {}", e))
    }
}
