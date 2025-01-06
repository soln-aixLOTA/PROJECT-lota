use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    core::error::DocumentResult,
    models::{Document, DocumentStatus},
};

pub struct DocumentRepository;

impl DocumentRepository {
    pub async fn create_document(db: &PgPool, document: &Document) -> DocumentResult<Document> {
        let record = sqlx::query_as!(
            Document,
            r#"
            INSERT INTO documents (
                id, name, content_type, size, storage_path, status,
                metadata, created_by, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#,
            document.id,
            document.name,
            document.content_type,
            document.size,
            document.storage_path,
            document.status as _,
            serde_json::to_value(&document.metadata)? as _,
            document.created_by,
            document.created_at,
            document.updated_at,
        )
        .fetch_one(db)
        .await?;

        Ok(record)
    }

    pub async fn get_document(db: &PgPool, id: Uuid) -> DocumentResult<Document> {
        let record = sqlx::query_as!(
            Document,
            r#"
            SELECT *
            FROM documents
            WHERE id = $1
            "#,
            id
        )
        .fetch_one(db)
        .await?;

        Ok(record)
    }

    pub async fn update_document_status(
        db: &PgPool,
        id: Uuid,
        status: DocumentStatus,
    ) -> DocumentResult<()> {
        sqlx::query!(
            r#"
            UPDATE documents
            SET status = $1, updated_at = NOW()
            WHERE id = $2
            "#,
            status as _,
            id
        )
        .execute(db)
        .await?;

        Ok(())
    }

    pub async fn list_documents(
        db: &PgPool,
        offset: i64,
        limit: i64,
        status: Option<DocumentStatus>,
    ) -> DocumentResult<Vec<Document>> {
        let records = sqlx::query_as!(
            Document,
            r#"
            SELECT *
            FROM documents
            WHERE ($3::document_status IS NULL OR status = $3)
            ORDER BY created_at DESC
            LIMIT $1
            OFFSET $2
            "#,
            limit,
            offset,
            status as _
        )
        .fetch_all(db)
        .await?;

        Ok(records)
    }

    pub async fn delete_document(db: &PgPool, id: Uuid) -> DocumentResult<()> {
        let result = sqlx::query!(
            r#"
            DELETE FROM documents
            WHERE id = $1
            "#,
            id
        )
        .execute(db)
        .await?;

        if result.rows_affected() == 0 {
            return Err(crate::core::error::DocumentError::NotFound);
        }

        Ok(())
    }
}
