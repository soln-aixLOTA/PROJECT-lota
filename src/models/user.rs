use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::db::users::User as DbUser;
use crate::models::auth::UserRole;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub mfa_enabled: bool,
    #[serde(skip_serializing)]
    pub mfa_secret: Option<String>,
    pub role: UserRole,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn is_mfa_enabled(&self) -> bool {
        self.mfa_enabled
    }

    pub fn get_mfa_secret(&self) -> Option<&str> {
        self.mfa_secret.as_deref()
    }
}

impl From<DbUser> for User {
    fn from(db_user: DbUser) -> Self {
        let role = match db_user.role.as_str() {
            "admin" => UserRole::Admin,
            _ => UserRole::User,
        };

        Self {
            id: db_user.id,
            username: db_user.username,
            email: db_user.email,
            password_hash: db_user.password_hash,
            mfa_enabled: db_user.mfa_enabled,
            mfa_secret: db_user.mfa_secret,
            role,
            created_at: db_user.created_at,
            updated_at: db_user.updated_at,
        }
    }
}
