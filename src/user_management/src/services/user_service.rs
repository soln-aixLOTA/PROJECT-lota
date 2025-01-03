use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Duration, Utc};
use argon2::{self, Config};
use jsonwebtoken::{encode, EncodingKey, Header};
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::{
    error::ServiceError,
    models::user::{
        User, CreateUserRequest, UpdateUserRequest, UserProfile,
        ChangePasswordRequest, ResetPasswordRequest, LoginRequest,
    },
    repositories::{
        user_repository::UserRepository,
        role_repository::RoleRepository,
        permission_repository::PermissionRepository,
        audit_repository::AuditRepository,
    },
    config::JwtConfig,
};

const MAX_LOGIN_ATTEMPTS: u32 = 5;
const LOCKOUT_DURATION: i64 = 15; // minutes
const ACCESS_TOKEN_EXPIRY: i64 = 60; // minutes
const REFRESH_TOKEN_EXPIRY: i64 = 7 * 24 * 60; // 7 days in minutes

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid, // user_id
    pub tenant_id: Uuid,
    pub exp: i64,
    pub iat: i64,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
}

pub struct UserService {
    user_repository: Arc<dyn UserRepository>,
    role_repository: Arc<dyn RoleRepository>,
    permission_repository: Arc<dyn PermissionRepository>,
    audit_repository: Arc<dyn AuditRepository>,
    jwt_config: JwtConfig,
}

impl UserService {
    pub fn new(
        user_repository: Arc<dyn UserRepository>,
        role_repository: Arc<dyn RoleRepository>,
        permission_repository: Arc<dyn PermissionRepository>,
        audit_repository: Arc<dyn AuditRepository>,
        jwt_config: JwtConfig,
    ) -> Self {
        Self {
            user_repository,
            role_repository,
            permission_repository,
            audit_repository,
            jwt_config,
        }
    }

    pub async fn register_user(
        &self,
        tenant_id: Uuid,
        request: CreateUserRequest,
    ) -> Result<User, ServiceError> {
        // Check if email is unique within tenant
        if let Some(_) = self.user_repository
            .get_user_by_email(tenant_id, &request.email)
            .await?
        {
            return Err(ServiceError::EmailAlreadyExists(request.email));
        }

        // Hash password
        let password_hash = self.hash_password(&request.password)?;

        // Create user with hashed password
        let user = self.user_repository
            .create_user(&CreateUserRequest {
                password: password_hash,
                ..request
            })
            .await?;

        // Log user creation
        self.audit_repository.log_event(
            tenant_id,
            Some(user.id),
            "user_registered",
            serde_json::json!({
                "user_id": user.id,
                "email": user.email,
            }),
        ).await?;

        Ok(user)
    }

    pub async fn authenticate_user(
        &self,
        tenant_id: Uuid,
        request: LoginRequest,
    ) -> Result<(String, String), ServiceError> {
        // Get user by email
        let user = self.user_repository
            .get_user_by_email(tenant_id, &request.email)
            .await?
            .ok_or_else(|| ServiceError::InvalidCredentials)?;

        // Check if account is locked
        if let Some(locked_until) = user.locked_until {
            if locked_until > Utc::now() {
                return Err(ServiceError::AccountLocked(locked_until));
            }
        }

        // Verify password
        if !self.verify_password(&request.password, &user.password_hash)? {
            // Increment failed login attempts
            let attempts = user.failed_login_attempts + 1;
            let locked_until = if attempts >= MAX_LOGIN_ATTEMPTS {
                Some(Utc::now() + Duration::minutes(LOCKOUT_DURATION))
            } else {
                None
            };

            self.user_repository
                .update_login_attempts(user.id, attempts, locked_until)
                .await?;

            // Log failed login attempt
            self.audit_repository.log_event(
                tenant_id,
                Some(user.id),
                "login_failed",
                serde_json::json!({
                    "attempts": attempts,
                    "locked_until": locked_until,
                }),
            ).await?;

            return Err(ServiceError::InvalidCredentials);
        }

        // Reset failed login attempts
        if user.failed_login_attempts > 0 {
            self.user_repository
                .update_login_attempts(user.id, 0, None)
                .await?;
        }

        // Get user roles and permissions
        let roles = self.role_repository.get_user_roles(user.id).await?;
        let permissions = self.permission_repository.get_user_permissions(user.id).await?;

        // Generate tokens
        let access_token = self.generate_token(
            user.id,
            tenant_id,
            ACCESS_TOKEN_EXPIRY,
            &roles,
            &permissions,
        )?;

        let refresh_token = self.generate_token(
            user.id,
            tenant_id,
            REFRESH_TOKEN_EXPIRY,
            &roles,
            &permissions,
        )?;

        // Log successful login
        self.audit_repository.log_event(
            tenant_id,
            Some(user.id),
            "login_successful",
            serde_json::json!({}),
        ).await?;

        Ok((access_token, refresh_token))
    }

    pub async fn refresh_token(
        &self,
        refresh_token: &str,
    ) -> Result<String, ServiceError> {
        // Validate refresh token and get claims
        let claims = self.validate_token(refresh_token)?;

        // Get user roles and permissions
        let roles = self.role_repository.get_user_roles(claims.sub).await?;
        let permissions = self.permission_repository.get_user_permissions(claims.sub).await?;

        // Generate new access token
        let access_token = self.generate_token(
            claims.sub,
            claims.tenant_id,
            ACCESS_TOKEN_EXPIRY,
            &roles,
            &permissions,
        )?;

        Ok(access_token)
    }

    pub async fn get_user_profile(
        &self,
        user_id: Uuid,
    ) -> Result<UserProfile, ServiceError> {
        // Get user
        let user = self.user_repository
            .get_user(user_id)
            .await?
            .ok_or_else(|| ServiceError::UserNotFound(user_id))?;

        // Get user roles and permissions
        let roles = self.role_repository.get_user_roles(user_id).await?;
        let permissions = self.permission_repository.get_user_permissions(user_id).await?;

        Ok(UserProfile {
            user,
            roles,
            permissions,
        })
    }

    pub async fn update_user_profile(
        &self,
        user_id: Uuid,
        request: UpdateUserRequest,
    ) -> Result<User, ServiceError> {
        // Check if user exists
        let existing_user = self.user_repository
            .get_user(user_id)
            .await?
            .ok_or_else(|| ServiceError::UserNotFound(user_id))?;

        // If email is being updated, check for uniqueness
        if let Some(ref email) = request.email {
            if email != &existing_user.email {
                if let Some(_) = self.user_repository
                    .get_user_by_email(existing_user.tenant_id, email)
                    .await?
                {
                    return Err(ServiceError::EmailAlreadyExists(email.clone()));
                }
            }
        }

        // Update user
        let user = self.user_repository.update_user(user_id, &request).await?;

        // Log profile update
        self.audit_repository.log_event(
            user.tenant_id,
            Some(user_id),
            "profile_updated",
            serde_json::json!({
                "updates": request,
            }),
        ).await?;

        Ok(user)
    }

    pub async fn change_password(
        &self,
        user_id: Uuid,
        request: ChangePasswordRequest,
    ) -> Result<(), ServiceError> {
        // Get user
        let user = self.user_repository
            .get_user(user_id)
            .await?
            .ok_or_else(|| ServiceError::UserNotFound(user_id))?;

        // Verify current password
        if !self.verify_password(&request.current_password, &user.password_hash)? {
            return Err(ServiceError::InvalidPassword);
        }

        // Hash new password
        let password_hash = self.hash_password(&request.new_password)?;

        // Update password
        self.user_repository
            .update_password(user_id, &password_hash)
            .await?;

        // Log password change
        self.audit_repository.log_event(
            user.tenant_id,
            Some(user_id),
            "password_changed",
            serde_json::json!({}),
        ).await?;

        Ok(())
    }

    pub async fn initiate_password_reset(
        &self,
        tenant_id: Uuid,
        email: &str,
    ) -> Result<String, ServiceError> {
        // Get user by email
        let user = self.user_repository
            .get_user_by_email(tenant_id, email)
            .await?
            .ok_or_else(|| ServiceError::UserNotFound(Uuid::nil()))?;

        // Generate reset token
        let reset_token: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        // Store reset token with expiry
        let expiry = Utc::now() + Duration::hours(24);
        self.user_repository
            .store_reset_token(user.id, &reset_token, expiry)
            .await?;

        // Log password reset initiation
        self.audit_repository.log_event(
            tenant_id,
            Some(user.id),
            "password_reset_initiated",
            serde_json::json!({}),
        ).await?;

        Ok(reset_token)
    }

    pub async fn reset_password(
        &self,
        tenant_id: Uuid,
        request: ResetPasswordRequest,
    ) -> Result<(), ServiceError> {
        // Validate reset token
        let user_id = self.user_repository
            .validate_reset_token(&request.reset_token)
            .await?
            .ok_or_else(|| ServiceError::InvalidResetToken)?;

        // Hash new password
        let password_hash = self.hash_password(&request.new_password)?;

        // Update password
        self.user_repository
            .update_password(user_id, &password_hash)
            .await?;

        // Clear reset token
        self.user_repository
            .clear_reset_token(user_id)
            .await?;

        // Log password reset
        self.audit_repository.log_event(
            tenant_id,
            Some(user_id),
            "password_reset_completed",
            serde_json::json!({}),
        ).await?;

        Ok(())
    }

    pub async fn delete_user(
        &self,
        user_id: Uuid,
    ) -> Result<(), ServiceError> {
        // Check if user exists
        let user = self.user_repository
            .get_user(user_id)
            .await?
            .ok_or_else(|| ServiceError::UserNotFound(user_id))?;

        // Delete user
        self.user_repository.delete_user(user_id).await?;

        // Log user deletion
        self.audit_repository.log_event(
            user.tenant_id,
            Some(user_id),
            "user_deleted",
            serde_json::json!({}),
        ).await?;

        Ok(())
    }

    pub async fn list_users(
        &self,
        tenant_id: Uuid,
        page: u32,
        per_page: u32,
    ) -> Result<Vec<User>, ServiceError> {
        let users = self.user_repository.list_users(tenant_id).await?;
        Ok(users)
    }

    // Helper methods

    pub fn hash_password(&self, password: &str) -> Result<String, ServiceError> {
        let salt: [u8; 32] = rand::thread_rng().gen();
        let config = Config::default();
        argon2::hash_encoded(password.as_bytes(), &salt, &config)
            .map_err(|_| ServiceError::InternalError("Password hashing failed".into()))
    }

    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool, ServiceError> {
        argon2::verify_encoded(hash, password.as_bytes())
            .map_err(|_| ServiceError::Unauthorized("Invalid password".into()))
    }

    pub fn generate_jwt(&self, user: &User) -> Result<String, ServiceError> {
        let now = Utc::now().timestamp();
        let claims = Claims {
            sub: user.id,
            tenant_id: user.tenant_id,
            exp: now + ACCESS_TOKEN_EXPIRY * 60,
            iat: now,
            roles: user.roles.clone(),
            permissions: user.permissions.clone(),
        };
        encode(&Header::default(), &claims, &EncodingKey::from_secret(self.jwt_config.secret.as_ref()))
            .map_err(|_| ServiceError::InternalError("JWT generation failed".into()))
    }

    pub fn validate_jwt(&self, token: &str) -> Result<Claims, ServiceError> {
        // Implement JWT validation logic here
        // This is a placeholder for the actual implementation
        Err(ServiceError::Unauthorized("JWT validation not implemented".into()))
    }

    fn generate_token(
        &self,
        user_id: Uuid,
        tenant_id: Uuid,
        expiry_minutes: i64,
        roles: &[String],
        permissions: &[String],
    ) -> Result<String, ServiceError> {
        let now = Utc::now();
        let exp = (now + Duration::minutes(expiry_minutes)).timestamp();
        let claims = Claims {
            sub: user_id,
            tenant_id,
            exp,
            iat: now.timestamp(),
            roles: roles.to_vec(),
            permissions: permissions.to_vec(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_config.secret.as_bytes()),
        )
        .map_err(|_| ServiceError::TokenGenerationError)
    }

    fn validate_token(&self, token: &str) -> Result<Claims, ServiceError> {
        use jsonwebtoken::{decode, DecodingKey, Validation};

        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_config.secret.as_bytes()),
            &Validation::default(),
        )
        .map(|token_data| token_data.claims)
        .map_err(|_| ServiceError::InvalidToken)
    }
} 