use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use crate::error::AppResult;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub mfa_enabled: bool,
    pub mfa_secret: Option<String>,
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub async fn create_user(
    pool: &PgPool,
    username: String,
    email: String,
    password_hash: String,
) -> AppResult<User> {
    let user = sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (username, email, password_hash)
        VALUES ($1, $2, $3)
        RETURNING id, username, email, password_hash, mfa_enabled, mfa_secret, role, created_at, updated_at
        "#,
        username,
        email,
        password_hash
    )
    .fetch_one(pool)
    .await?;

    Ok(user)
}

pub async fn get_user_by_username(pool: &PgPool, username: &str) -> AppResult<User> {
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT id, username, email, password_hash, mfa_enabled, mfa_secret, role, created_at, updated_at
        FROM users
        WHERE username = $1
        "#,
        username
    )
    .fetch_one(pool)
    .await?;

    Ok(user)
}

pub async fn get_user_by_id(pool: &PgPool, id: Uuid) -> AppResult<User> {
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT id, username, email, password_hash, mfa_enabled, mfa_secret, role, created_at, updated_at
        FROM users
        WHERE id = $1
        "#,
        id
    )
    .fetch_one(pool)
    .await?;

    Ok(user)
} 