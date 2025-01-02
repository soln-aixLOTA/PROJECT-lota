use crate::models::user::{CreateUserRequest, User, UserRole};
use anyhow::Result;
use chrono::prelude::*;
use chrono::{DateTime, Utc};
use sqlx::types::time::OffsetDateTime;
use sqlx::PgPool;
use uuid::Uuid;
use crate::errors::ApiError;
use tracing::{error, instrument};

#[derive(Debug)]
pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[instrument(skip(self))]
    pub async fn create(&self, user: CreateUserRequest, password_hash: String) -> Result<User, ApiError> {
        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (email, password_hash, role)
            VALUES ($1, $2, $3)
            RETURNING id, email, password_hash, role as "role: UserRole",
                      created_at as "created_at: OffsetDateTime",
                      updated_at as "updated_at: OffsetDateTime",
                      last_login as "last_login: Option<OffsetDateTime>",
                      is_active
            "#,
            user.email,
            password_hash,
            user.role as UserRole,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            error!("Database error while creating user: {}", e);
            ApiError::from(e)
        })?;

        let user = User {
            id: user.id,
            email: user.email,
            password_hash: user.password_hash,
            role: user.role,
            created_at: user.created_at.to_offset(Utc.fix()).into(),
            updated_at: user.updated_at.to_offset(Utc.fix()).into(),
            last_login: user.last_login.map(|dt| dt.to_offset(Utc.fix()).into()),
            is_active: user.is_active,
        };

        Ok(user)
    }

    #[instrument(skip(self))]
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, ApiError> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, email, password_hash, role as "role: UserRole",
                   created_at as "created_at: OffsetDateTime",
                   updated_at as "updated_at: OffsetDateTime",
                   last_login as "last_login: Option<OffsetDateTime>",
                   is_active
            FROM users
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            error!("Database error while finding user by id: {}", e);
            ApiError::from(e)
        })?;

        let user = user.map(|user| User {
            id: user.id,
            email: user.email,
            password_hash: user.password_hash,
            role: user.role,
            created_at: user.created_at.to_offset(Utc.fix()).into(),
            updated_at: user.updated_at.to_offset(Utc.fix()).into(),
            last_login: user.last_login.map(|dt| dt.to_offset(Utc.fix()).into()),
            is_active: user.is_active,
        });

        Ok(user)
    }

    #[instrument(skip(self))]
    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, ApiError> {
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
        .await
        .map_err(|e| {
            error!("Database error while finding user by email: {}", e);
            ApiError::from(e)
        })?;

        let user = user.map(|user| {
            let created_at: chrono::DateTime<Utc> = chrono::Utc
                .timestamp_opt(user.created_at.timestamp(), 0)
                .unwrap();

            let updated_at: chrono::DateTime<Utc> = chrono::Utc
                .timestamp_opt(user.updated_at.timestamp(), 0)
                .unwrap();

            User {
                id: user.id,
                email: user.email,
                password_hash: user.password_hash,
                role: user.role,
                created_at,
                updated_at,
                last_login: user.last_login.map(|dt| {
                    DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(dt.timestamp(), 0), Utc)
                }),
                is_active: user.is_active,
            }
        });

        Ok(user)
    }

    #[instrument(skip(self))]
    pub async fn update_last_login(&self, id: Uuid, last_login: DateTime<Utc>) -> Result<(), ApiError> {
        let last_login_offset = OffsetDateTime::from_unix_timestamp(last_login.timestamp())
            .map_err(|e| {
                error!("Database error while updating last login: {}", e);
                ApiError::from(e)
            })?;
        sqlx::query!(
            r#"
            UPDATE users
            SET last_login = $1
            WHERE id = $2
            "#,
            last_login_offset,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!("Database error while updating last login: {}", e);
            ApiError::from(e)
        })?;

        Ok(())
    }
}
