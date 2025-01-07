use sqlx::PgPool;
use uuid::Uuid;

use crate::core::error::AppResult;
use crate::models::user::User;

pub async fn create_user(
    pool: &PgPool,
    username: String,
    password_hash: String,
) -> AppResult<User> {
    let user = sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (username, password_hash)
        VALUES ($1, $2)
        RETURNING id, username, password_hash, created_at, updated_at
        "#,
        username,
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
        SELECT id, username, password_hash, created_at, updated_at
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
        SELECT id, username, password_hash, created_at, updated_at
        FROM users
        WHERE id = $1
        "#,
        id
    )
    .fetch_one(pool)
    .await?;

    Ok(user)
} 