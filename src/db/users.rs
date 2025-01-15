use crate::{AppError, AppResult};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn get_user_by_id(pool: &PgPool, user_id: Uuid) -> AppResult<Option<User>> {
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT id, email, password_hash, role, created_at, updated_at
        FROM users
        WHERE id = $1
        "#,
        user_id
    )
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?;

    Ok(user)
}

#[derive(Debug, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub role: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
