use anyhow::{anyhow, Result};
use bcrypt::{hash, verify, BcryptError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::Type, PartialEq, Eq, Clone, Copy)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum UserRole {
    User,
    Admin,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub role: UserRole,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub role: UserRole,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

impl User {
    pub fn verify_password(&self, password: &str) -> Result<bool> {
        verify(password, &self.password_hash)
            .map_err(|e| anyhow!("Failed to verify password: {}", e))
    }
}

impl CreateUserRequest {
    pub fn hash_password(&self, password: &str) -> Result<String> {
        hash(password, bcrypt::DEFAULT_COST).map_err(|e| anyhow!("Failed to hash password: {}", e))
    }
}
