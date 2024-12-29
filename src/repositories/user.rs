use crate::models::user::{CreateUserRequest, User, UserRole};
use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug)]
pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, user: CreateUserRequest, password_hash: String) -> Result<User> {
        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (email, password_hash, role)
            VALUES ($1, $2, $3)
            RETURNING id, email, password_hash, role as "role: UserRole", 
                      created_at, updated_at, last_login, is_active
            "#,
            user.email,
            password_hash,
            user.role as UserRole,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<User>> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, email, password_hash, role as "role: UserRole",
                   created_at, updated_at, last_login, is_active
            FROM users
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, email, password_hash, role as "role: UserRole",
                   created_at, updated_at, last_login, is_active
            FROM users
            WHERE email = $1
            "#,
            email
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn update_last_login(&self, id: Uuid, last_login: DateTime<Utc>) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE users
            SET last_login = $1
            WHERE id = $2
            "#,
            last_login,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_password(&self, id: Uuid, password_hash: String) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE users
            SET password_hash = $1
            WHERE id = $2
            "#,
            password_hash,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn deactivate(&self, id: Uuid) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE users
            SET is_active = false
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn activate(&self, id: Uuid) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE users
            SET is_active = true
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
