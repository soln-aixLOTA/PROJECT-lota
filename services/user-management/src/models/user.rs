use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub status: UserStatus,
    pub mfa_enabled: bool,
    #[serde(skip_serializing)]
    pub mfa_secret: Option<String>,
    #[serde(skip_serializing)]
    pub password_reset_token: Option<Uuid>,
    #[serde(skip_serializing)]
    pub password_reset_expires_at: Option<DateTime<Utc>>,
    pub login_attempts: i32,
    pub locked_until: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum UserStatus {
    Active,
    Inactive,
    Suspended,
}

impl Default for UserStatus {
    fn default() -> Self {
        UserStatus::Active
    }
}

#[derive(Debug, Validate, Deserialize)]
pub struct CreateUserRequest {
    pub tenant_id: Uuid,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8, max = 100))]
    pub password: String,
    #[validate(length(max = 100))]
    pub first_name: Option<String>,
    #[validate(length(max = 100))]
    pub last_name: Option<String>,
    pub mfa_enabled: Option<bool>,
}

#[derive(Debug, Validate, Deserialize)]
pub struct UpdateUserRequest {
    #[validate(email)]
    pub email: Option<String>,
    #[validate(length(max = 100))]
    pub first_name: Option<String>,
    #[validate(length(max = 100))]
    pub last_name: Option<String>,
    pub mfa_enabled: Option<bool>,
    pub status: Option<UserStatus>,
}

#[derive(Debug, Validate, Deserialize)]
pub struct ChangePasswordRequest {
    #[validate(length(min = 8, max = 100))]
    pub current_password: String,
    #[validate(length(min = 8, max = 100))]
    pub new_password: String,
}

#[derive(Debug, Validate, Deserialize)]
pub struct ResetPasswordRequest {
    pub reset_token: Uuid,
    #[validate(length(min = 8, max = 100))]
    pub new_password: String,
}

#[derive(Debug, Validate, Deserialize)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    pub password: String,
    pub mfa_code: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user: UserProfile,
}

#[derive(Debug, Serialize)]
pub struct UserProfile {
    pub id: Uuid,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub mfa_enabled: bool,
    pub tenant_id: Uuid,
}

impl User {
    pub fn is_locked(&self) -> bool {
        if let Some(locked_until) = self.locked_until {
            locked_until > Utc::now()
        } else {
            false
        }
    }

    pub fn full_name(&self) -> String {
        match (self.first_name.as_ref(), self.last_name.as_ref()) {
            (Some(first), Some(last)) => format!("{} {}", first, last),
            (Some(first), None) => first.clone(),
            (None, Some(last)) => last.clone(),
            (None, None) => String::new(),
        }
    }

    pub fn to_profile(&self, roles: Vec<String>, permissions: Vec<String>) -> UserProfile {
        UserProfile {
            id: self.id,
            email: self.email.clone(),
            first_name: self.first_name.clone(),
            last_name: self.last_name.clone(),
            roles,
            permissions,
            mfa_enabled: self.mfa_enabled,
            tenant_id: self.tenant_id,
        }
    }
} 