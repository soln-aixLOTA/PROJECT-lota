use async_trait::async_trait;
use sqlx::{Error as SqlxError, PgPool};
use uuid::Uuid;
use chrono::{DateTime, Duration, Utc};

use crate::models::user::{
    User, UserStatus, CreateUserRequest, UpdateUserRequest,
    ChangePasswordRequest, ResetPasswordRequest,
};

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create_user(&self, request: &CreateUserRequest) -> Result<User, SqlxError>;
    async fn get_user_by_id(&self, user_id: Uuid) -> Result<Option<User>, SqlxError>;
    async fn get_user_by_email(&self, tenant_id: Uuid, email: &str) -> Result<Option<User>, SqlxError>;
    async fn update_user(&self, user_id: Uuid, request: &UpdateUserRequest) -> Result<User, SqlxError>;
    async fn delete_user(&self, user_id: Uuid) -> Result<(), SqlxError>;
    async fn list_users(&self, tenant_id: Uuid) -> Result<Vec<User>, SqlxError>;
    async fn change_password(&self, user_id: Uuid, new_password_hash: &str) -> Result<(), SqlxError>;
    async fn set_reset_token(&self, user_id: Uuid, token: Uuid, expires_at: DateTime<Utc>) -> Result<(), SqlxError>;
    async fn reset_password(&self, user_id: Uuid, new_password_hash: &str, token: Uuid) -> Result<(), SqlxError>;
    async fn get_user_by_reset_token(&self, token: Uuid) -> Result<Option<User>, SqlxError>;
    async fn update_login_attempts(&self, user_id: Uuid, success: bool) -> Result<Option<User>, SqlxError>;
    async fn lock_user(&self, user_id: Uuid, duration: Duration) -> Result<(), SqlxError>;
    async fn unlock_user(&self, user_id: Uuid) -> Result<(), SqlxError>;
    async fn get_user_roles(&self, user_id: Uuid) -> Result<Vec<String>, SqlxError>;
    async fn get_user_permissions(&self, user_id: Uuid) -> Result<Vec<String>, SqlxError>;
}

pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn create_user(&self, request: &CreateUserRequest) -> Result<User, SqlxError> {
        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (
                tenant_id, email, password_hash, first_name, last_name, 
                mfa_enabled, status
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING 
                id, tenant_id, email, password_hash, first_name, last_name,
                created_at, updated_at, last_login_at, 
                status as "status: UserStatus",
                mfa_enabled, mfa_secret,
                password_reset_token, password_reset_expires_at,
                login_attempts, locked_until
            "#,
            request.tenant_id,
            request.email,
            request.password,  // Note: Password hashing should be done in service layer
            request.first_name,
            request.last_name,
            request.mfa_enabled.unwrap_or(false),
            UserStatus::Active as _,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    async fn get_user_by_id(&self, user_id: Uuid) -> Result<Option<User>, SqlxError> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT 
                id, tenant_id, email, password_hash, first_name, last_name,
                created_at, updated_at, last_login_at, 
                status as "status: UserStatus",
                mfa_enabled, mfa_secret,
                password_reset_token, password_reset_expires_at,
                login_attempts, locked_until
            FROM users
            WHERE id = $1
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    async fn get_user_by_email(&self, tenant_id: Uuid, email: &str) -> Result<Option<User>, SqlxError> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT 
                id, tenant_id, email, password_hash, first_name, last_name,
                created_at, updated_at, last_login_at, 
                status as "status: UserStatus",
                mfa_enabled, mfa_secret,
                password_reset_token, password_reset_expires_at,
                login_attempts, locked_until
            FROM users
            WHERE tenant_id = $1 AND email = $2
            "#,
            tenant_id,
            email
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    async fn update_user(&self, user_id: Uuid, request: &UpdateUserRequest) -> Result<User, SqlxError> {
        let user = sqlx::query_as!(
            User,
            r#"
            UPDATE users
            SET 
                email = COALESCE($2, email),
                first_name = COALESCE($3, first_name),
                last_name = COALESCE($4, last_name),
                mfa_enabled = COALESCE($5, mfa_enabled),
                status = COALESCE($6, status),
                updated_at = NOW()
            WHERE id = $1
            RETURNING 
                id, tenant_id, email, password_hash, first_name, last_name,
                created_at, updated_at, last_login_at, 
                status as "status: UserStatus",
                mfa_enabled, mfa_secret,
                password_reset_token, password_reset_expires_at,
                login_attempts, locked_until
            "#,
            user_id,
            request.email,
            request.first_name,
            request.last_name,
            request.mfa_enabled,
            request.status as _,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    async fn delete_user(&self, user_id: Uuid) -> Result<(), SqlxError> {
        // Soft delete by updating status
        sqlx::query!(
            r#"
            UPDATE users
            SET status = $1, updated_at = NOW()
            WHERE id = $2
            "#,
            UserStatus::Inactive as _,
            user_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn list_users(&self, tenant_id: Uuid) -> Result<Vec<User>, SqlxError> {
        let users = sqlx::query_as!(
            User,
            r#"
            SELECT 
                id, tenant_id, email, password_hash, first_name, last_name,
                created_at, updated_at, last_login_at, 
                status as "status: UserStatus",
                mfa_enabled, mfa_secret,
                password_reset_token, password_reset_expires_at,
                login_attempts, locked_until
            FROM users
            WHERE tenant_id = $1
            ORDER BY created_at DESC
            "#,
            tenant_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(users)
    }

    async fn change_password(&self, user_id: Uuid, new_password_hash: &str) -> Result<(), SqlxError> {
        sqlx::query!(
            r#"
            UPDATE users
            SET 
                password_hash = $2,
                updated_at = NOW(),
                password_reset_token = NULL,
                password_reset_expires_at = NULL
            WHERE id = $1
            "#,
            user_id,
            new_password_hash
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn set_reset_token(
        &self,
        user_id: Uuid,
        token: Uuid,
        expires_at: DateTime<Utc>,
    ) -> Result<(), SqlxError> {
        sqlx::query!(
            r#"
            UPDATE users
            SET 
                password_reset_token = $2,
                password_reset_expires_at = $3,
                updated_at = NOW()
            WHERE id = $1
            "#,
            user_id,
            token,
            expires_at
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn reset_password(
        &self,
        user_id: Uuid,
        new_password_hash: &str,
        token: Uuid,
    ) -> Result<(), SqlxError> {
        let result = sqlx::query!(
            r#"
            UPDATE users
            SET 
                password_hash = $2,
                password_reset_token = NULL,
                password_reset_expires_at = NULL,
                updated_at = NOW()
            WHERE id = $1 
              AND password_reset_token = $3
              AND password_reset_expires_at > NOW()
            "#,
            user_id,
            new_password_hash,
            token
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(SqlxError::RowNotFound);
        }

        Ok(())
    }

    async fn get_user_by_reset_token(&self, token: Uuid) -> Result<Option<User>, SqlxError> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT 
                id, tenant_id, email, password_hash, first_name, last_name,
                created_at, updated_at, last_login_at, 
                status as "status: UserStatus",
                mfa_enabled, mfa_secret,
                password_reset_token, password_reset_expires_at,
                login_attempts, locked_until
            FROM users
            WHERE password_reset_token = $1
              AND password_reset_expires_at > NOW()
            "#,
            token
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    async fn update_login_attempts(&self, user_id: Uuid, success: bool) -> Result<Option<User>, SqlxError> {
        let mut tx = self.pool.begin().await?;

        if success {
            sqlx::query!(
                r#"
                UPDATE users
                SET 
                    login_attempts = 0,
                    last_login_at = NOW(),
                    locked_until = NULL,
                    updated_at = NOW()
                WHERE id = $1
                "#,
                user_id
            )
            .execute(&mut *tx)
            .await?;
        } else {
            sqlx::query!(
                r#"
                UPDATE users
                SET 
                    login_attempts = login_attempts + 1,
                    updated_at = NOW()
                WHERE id = $1
                "#,
                user_id
            )
            .execute(&mut *tx)
            .await?;
        }

        let user = sqlx::query_as!(
            User,
            r#"
            SELECT 
                id, tenant_id, email, password_hash, first_name, last_name,
                created_at, updated_at, last_login_at, 
                status as "status: UserStatus",
                mfa_enabled, mfa_secret,
                password_reset_token, password_reset_expires_at,
                login_attempts, locked_until
            FROM users
            WHERE id = $1
            "#,
            user_id
        )
        .fetch_optional(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(user)
    }

    async fn lock_user(&self, user_id: Uuid, duration: Duration) -> Result<(), SqlxError> {
        let locked_until = Utc::now() + duration;

        sqlx::query!(
            r#"
            UPDATE users
            SET 
                locked_until = $2,
                updated_at = NOW()
            WHERE id = $1
            "#,
            user_id,
            locked_until
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn unlock_user(&self, user_id: Uuid) -> Result<(), SqlxError> {
        sqlx::query!(
            r#"
            UPDATE users
            SET 
                login_attempts = 0,
                locked_until = NULL,
                updated_at = NOW()
            WHERE id = $1
            "#,
            user_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_user_roles(&self, user_id: Uuid) -> Result<Vec<String>, SqlxError> {
        let roles = sqlx::query!(
            r#"
            SELECT r.name
            FROM roles r
            INNER JOIN user_roles ur ON r.id = ur.role_id
            WHERE ur.user_id = $1
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|row| row.name)
        .collect();

        Ok(roles)
    }

    async fn get_user_permissions(&self, user_id: Uuid) -> Result<Vec<String>, SqlxError> {
        let permissions = sqlx::query!(
            r#"
            SELECT DISTINCT p.name
            FROM permissions p
            INNER JOIN role_permissions rp ON p.id = rp.permission_id
            INNER JOIN roles r ON rp.role_id = r.id
            INNER JOIN user_roles ur ON r.id = ur.role_id
            WHERE ur.user_id = $1
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|row| row.name)
        .collect();

        Ok(permissions)
    }
} 