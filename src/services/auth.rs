use crate::{
    models::user::{CreateUserRequest, LoginRequest, User},
    repositories::user::UserRepository,
    utils::security::{hash_password, verify_password},
};
use anyhow::{anyhow, Result};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde_json::json;
use tracing::info;
use uuid::Uuid;

pub struct AuthService {
    user_repo: UserRepository,
}

impl AuthService {
    pub fn new(user_repo: UserRepository) -> Self {
        Self { user_repo }
    }

    pub async fn register(&self, request: CreateUserRequest) -> Result<User> {
        // Check if user already exists
        if let Some(_) = self.user_repo.find_by_email(&request.email).await? {
            return Err(anyhow!("User with this email already exists"));
        }

        // Hash password
        let password_hash = hash_password(&request.password)?;

        // Create user
        let user = self.user_repo.create(request, password_hash).await?;
        info!("Created new user with ID: {}", user.id);

        Ok(user)
    }

    pub async fn login(&self, request: LoginRequest) -> Result<serde_json::Value> {
        // Find user by email
        let user = match self.user_repo.find_by_email(&request.email).await? {
            Some(user) => user,
            None => return Err(anyhow!("Invalid credentials")),
        };

        // Verify password
        if !verify_password(&request.password, &user.password_hash)? {
            return Err(anyhow!("Invalid credentials"));
        }

        // Generate JWT token
        let expiration = Utc::now()
            .checked_add_signed(Duration::days(1))
            .expect("Invalid timestamp")
            .timestamp();

        let claims = json!({
            "sub": user.id.to_string(),
            "exp": expiration,
            "iat": Utc::now().timestamp(),
            "role": user.role
        });

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret("your-secret-key".as_bytes()),
        )?;

        // Update last login
        self.user_repo
            .update_last_login(user.id, Utc::now())
            .await?;

        Ok(json!({
            "token": token,
            "user": user
        }))
    }

    pub async fn change_password(
        &self,
        user_id: Uuid,
        old_password: String,
        new_password: String,
    ) -> Result<()> {
        // Find user
        let user = match self.user_repo.find_by_id(user_id).await? {
            Some(user) => user,
            None => return Err(anyhow!("User not found")),
        };

        // Verify old password
        if !verify_password(&old_password, &user.password_hash)? {
            return Err(anyhow!("Invalid old password"));
        }

        // Hash new password
        let password_hash = hash_password(&new_password)?;

        // Update password
        self.user_repo
            .update_password(user_id, password_hash)
            .await?;

        Ok(())
    }

    pub async fn deactivate_account(&self, user_id: Uuid) -> Result<()> {
        // Find user
        let user = match self.user_repo.find_by_id(user_id).await? {
            Some(user) => user,
            None => return Err(anyhow!("User not found")),
        };

        // Deactivate account
        self.user_repo.deactivate(user_id).await?;

        Ok(())
    }

    pub async fn activate_account(&self, user_id: Uuid) -> Result<()> {
        // Find user
        let user = match self.user_repo.find_by_id(user_id).await? {
            Some(user) => user,
            None => return Err(anyhow!("User not found")),
        };

        // Activate account
        self.user_repo.activate(user_id).await?;

        Ok(())
    }
}
