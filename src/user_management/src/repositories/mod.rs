use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::errors::Error;
use crate::models::{CreateUser, UpdateUser, User};

pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, user: CreateUser) -> Result<User, Error> {
        let now = OffsetDateTime::now_utc();
        let id = Uuid::new_v4();

        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (id, email, name, password_hash, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, email, name, password_hash, created_at, updated_at
            "#,
            id,
            user.email,
            user.name,
            user.password, // Note: This should be hashed in production
            now,
            now
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::Database)?;

        Ok(user)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, Error> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, email, name, password_hash, created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(Error::Database)?;

        Ok(user)
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, Error> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, email, name, password_hash, created_at, updated_at
            FROM users
            WHERE email = $1
            "#,
            email
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(Error::Database)?;

        Ok(user)
    }

    pub async fn update(&self, id: Uuid, user: UpdateUser) -> Result<User, Error> {
        let now = OffsetDateTime::now_utc();

        let user = sqlx::query_as!(
            User,
            r#"
            UPDATE users
            SET 
                name = COALESCE($1, name),
                email = COALESCE($2, email),
                password_hash = COALESCE($3, password_hash),
                updated_at = $4
            WHERE id = $5
            RETURNING id, email, name, password_hash, created_at, updated_at
            "#,
            user.name,
            user.email,
            user.password, // Note: This should be hashed in production
            now,
            id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::Database)?;

        Ok(user)
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), Error> {
        sqlx::query!(
            r#"
            DELETE FROM users
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(Error::Database)?;

        Ok(())
    }
}
