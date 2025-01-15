use crate::error::AppError;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Document {
    pub id: Uuid,
    pub title: String,
    pub content: Option<String>,
    pub file_path: String,
    pub content_type: String,
    pub size: i64,
    pub metadata: Option<serde_json::Value>,
    pub user_id: Uuid,
    pub document_type: String,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateDocumentRequest {
    pub title: String,
    pub content: Option<String>,
    pub file_path: String,
    pub content_type: String,
    pub size: i64,
    pub metadata: Option<serde_json::Value>,
    pub document_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateDocumentRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub file_path: Option<String>,
    pub content_type: Option<String>,
    pub size: Option<i64>,
    pub metadata: Option<serde_json::Value>,
    pub document_type: Option<String>,
    pub status: Option<String>,
}

impl Document {
    pub async fn create(
        pool: &PgPool,
        user_id: Uuid,
        req: CreateDocumentRequest,
    ) -> Result<Self, AppError> {
        sqlx::query_as::<_, Document>(
            r#"
            INSERT INTO documents (
                title, content, file_path, content_type, size,
                metadata, user_id, document_type, status
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'pending')
            RETURNING id, title, content, file_path, content_type, size,
                      metadata, user_id, document_type, status, created_at, updated_at
            "#,
        )
        .bind(&req.title)
        .bind(&req.content)
        .bind(&req.file_path)
        .bind(&req.content_type)
        .bind(req.size)
        .bind(&req.metadata)
        .bind(user_id)
        .bind(&req.document_type)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)
    }

    pub async fn get(pool: &PgPool, user_id: Uuid, id: Uuid) -> Result<Self, AppError> {
        sqlx::query_as::<_, Document>(
            r#"
            SELECT id, title, content, file_path, content_type, size,
                   metadata, user_id, document_type, status, created_at, updated_at
            FROM documents
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(id)
        .bind(user_id)
        .fetch_one(pool)
        .await
        .map_err(|_| AppError::NotFound(format!("Document {} not found", id)))
    }

    pub async fn list(
        pool: &PgPool,
        user_id: Uuid,
        page: i64,
        per_page: i64,
    ) -> Result<Vec<Self>, AppError> {
        let offset = (page - 1) * per_page;

        sqlx::query_as::<_, Document>(
            r#"
            SELECT id, title, content, file_path, content_type, size,
                   metadata, user_id, document_type, status, created_at, updated_at
            FROM documents
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(user_id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)
    }

    pub async fn update(
        pool: &PgPool,
        user_id: Uuid,
        id: Uuid,
        req: UpdateDocumentRequest,
    ) -> Result<Self, AppError> {
        sqlx::query_as::<_, Document>(
            r#"
            UPDATE documents
            SET
                title = COALESCE($1, title),
                content = COALESCE($2, content),
                file_path = COALESCE($3, file_path),
                content_type = COALESCE($4, content_type),
                size = COALESCE($5, size),
                metadata = COALESCE($6, metadata),
                document_type = COALESCE($7, document_type),
                status = COALESCE($8, status),
                updated_at = NOW()
            WHERE id = $9 AND user_id = $10
            RETURNING id, title, content, file_path, content_type, size,
                      metadata, user_id, document_type, status, created_at, updated_at
            "#,
        )
        .bind(&req.title)
        .bind(&req.content)
        .bind(&req.file_path)
        .bind(&req.content_type)
        .bind(req.size)
        .bind(&req.metadata)
        .bind(&req.document_type)
        .bind(&req.status)
        .bind(id)
        .bind(user_id)
        .fetch_one(pool)
        .await
        .map_err(|_| AppError::NotFound(format!("Document {} not found", id)))
    }
}
