use crate::error::{AppError, AppResult};
use crate::models::User;
use bcrypt::{hash, verify, DEFAULT_COST};
use sqlx::PgPool;
use uuid::Uuid;

pub struct UserService {
    pool: PgPool,
}

impl UserService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn register_user(&self, username: &str, password: &str, email: &str) -> AppResult<User> {
        // Check if user already exists
        let existing_user = sqlx::query!(
            r#"
            SELECT id FROM users
            WHERE username = $1 OR email = $2
            "#,
            username,
            email
        )
        .fetch_optional(&self.pool)
        .await?;

        if existing_user.is_some() {
            return Err(AppError::Validation("Username or email already exists".into()));
        }

        // Hash password
        let password_hash = hash(password.as_bytes(), DEFAULT_COST)
            .map_err(|e| AppError::Internal(format!("Failed to hash password: {}", e)))?;

        // Create user
        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (username, email, password_hash)
            VALUES ($1, $2, $3)
            RETURNING id, username, email, password_hash, mfa_enabled, mfa_secret, created_at, updated_at
            "#,
            username,
            email,
            password_hash
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn authenticate_user(&self, username: &str, password: &str) -> AppResult<User> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, username, email, password_hash, mfa_enabled, mfa_secret, created_at, updated_at
            FROM users
            WHERE username = $1
            "#,
            username
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::Authentication("Invalid credentials".into()))?;

        let valid = verify(password.as_bytes(), &user.password_hash)
            .map_err(|e| AppError::Internal(format!("Failed to verify password: {}", e)))?;

        if !valid {
            return Err(AppError::Authentication("Invalid credentials".into()));
        }

        Ok(user)
    }

    pub async fn get_user_by_id(&self, id: &Uuid) -> AppResult<Option<User>> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, username, email, password_hash, mfa_enabled, mfa_secret, created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn save_mfa_secret(&self, user_id: &Uuid, secret: &str) -> AppResult<()> {
        sqlx::query!(
            r#"
            UPDATE users
            SET mfa_secret = $1
            WHERE id = $2
            "#,
            secret,
            user_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn enable_mfa(&self, user_id: &Uuid) -> AppResult<()> {
        sqlx::query!(
            r#"
            UPDATE users
            SET mfa_enabled = true
            WHERE id = $1
            "#,
            user_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn disable_mfa(&self, user_id: &Uuid) -> AppResult<()> {
        sqlx::query!(
            r#"
            UPDATE users
            SET mfa_enabled = false, mfa_secret = NULL
            WHERE id = $1
            "#,
            user_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
} 