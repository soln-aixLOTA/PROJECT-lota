use crate::{db::users::User as DbUser, error::AppError};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub role: String,
}

impl From<DbUser> for User {
    fn from(db_user: DbUser) -> Self {
        Self {
            id: db_user.id,
            email: db_user.email,
            role: db_user.role,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub email: Option<String>,
    pub password: Option<String>,
    pub role: Option<String>,
}

impl User {
    pub async fn authenticate(
        pool: &PgPool,
        email: &str,
        password: &str,
    ) -> Result<Self, AppError> {
        let user = sqlx::query_as!(
            DbUser,
            r#"
            SELECT id, email, password_hash, role, created_at, updated_at
            FROM users
            WHERE email = $1
            "#,
            email
        )
        .fetch_one(pool)
        .await
        .map_err(|_| AppError::Authentication("Invalid email or password".to_string()))?;

        if !bcrypt::verify(password, &user.password_hash)
            .map_err(|e| AppError::Internal(format!("Password verification error: {}", e)))?
        {
            return Err(AppError::Authentication(
                "Invalid email or password".to_string(),
            ));
        }

        Ok(user.into())
    }

    pub async fn create(pool: &PgPool, req: CreateUserRequest) -> Result<Self, AppError> {
        let password_hash = bcrypt::hash(req.password.as_bytes(), bcrypt::DEFAULT_COST)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let user = sqlx::query_as!(
            DbUser,
            r#"
            INSERT INTO users (email, password_hash, role)
            VALUES ($1, $2, 'user')
            RETURNING id, email, password_hash, role, created_at, updated_at
            "#,
            req.email,
            password_hash,
        )
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)?;

        Ok(user.into())
    }

    pub async fn get(pool: &PgPool, id: Uuid) -> Result<Self, AppError> {
        let user = sqlx::query_as!(
            DbUser,
            r#"
            SELECT id, email, password_hash, role, created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
            id
        )
        .fetch_one(pool)
        .await
        .map_err(|_| AppError::NotFound(format!("User {} not found", id)))?;

        Ok(user.into())
    }

    pub async fn update(pool: &PgPool, id: Uuid, req: UpdateUserRequest) -> Result<Self, AppError> {
        let password_hash = if let Some(password) = req.password {
            Some(
                bcrypt::hash(password.as_bytes(), bcrypt::DEFAULT_COST)
                    .map_err(|e| AppError::Internal(e.to_string()))?,
            )
        } else {
            None
        };

        let user = sqlx::query_as!(
            DbUser,
            r#"
            UPDATE users
            SET
                email = COALESCE($1, email),
                password_hash = COALESCE($2, password_hash),
                role = COALESCE($3, role),
                updated_at = NOW()
            WHERE id = $4
            RETURNING id, email, password_hash, role, created_at, updated_at
            "#,
            req.email,
            password_hash,
            req.role,
            id
        )
        .fetch_one(pool)
        .await
        .map_err(|_| AppError::NotFound(format!("User {} not found", id)))?;

        Ok(user.into())
    }
}
