use sqlx::PgPool;
use uuid::Uuid;

<<<<<<< HEAD
use crate::core::error::{DocumentError, DocumentResult};
use crate::models::document::{
    Document, DocumentClassification, DocumentMetadata, DocumentStatus, SecurityLevel,
};

pub struct DocumentRepository {
    pool: PgPool,
}

impl DocumentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, document: &Document) -> DocumentResult<Document> {
        let metadata_json = serde_json::to_value(&document.metadata)?;
        let record = sqlx::query_as!(
            Document,
            r#"
            INSERT INTO documents (
                id, name, content_type, size, path,
                status, classification, security_level, metadata,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING 
                id, name, content_type, size, path,
                status as "status: _",
                classification as "classification: _",
                security_level as "security_level: _",
                metadata,
                created_at, updated_at
=======
use crate::core::error::DocumentResult;
use crate::models::document::{Document, DocumentMetadata, DocumentStatus};

#[derive(Debug, Clone)]
pub struct DocumentRepository;

impl DocumentRepository {
    pub async fn create_document(pool: &PgPool, document: &Document) -> DocumentResult<Document> {
        let record = sqlx::query!(
            r#"
            INSERT INTO documents (
                id, name, content_type, size, path,
                status, metadata, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
>>>>>>> 921251a (fetch)
            "#,
            document.id,
            document.name,
            document.content_type,
            document.size,
            document.path,
<<<<<<< HEAD
            document.status as _,
            document.classification as _,
            document.security_level as _,
            metadata_json,
            document.created_at,
            document.updated_at,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(record)
    }

    pub async fn get(&self, id: &str) -> DocumentResult<Document> {
        let record = sqlx::query_as!(
            Document,
            r#"
            SELECT 
                id, name, content_type, size, path,
                status as "status: _",
                classification as "classification: _",
                security_level as "security_level: _",
                metadata,
                created_at, updated_at
=======
            document.status as DocumentStatus,
            document.metadata as serde_json::Value,
            document.created_at,
            document.updated_at,
        )
        .fetch_one(pool)
        .await?;

        Ok(Document {
            id: record.id,
            name: record.name,
            content_type: record.content_type,
            size: record.size,
            path: record.path,
            status: record.status.parse()?,
            metadata: serde_json::from_value(record.metadata)?,
            created_at: record.created_at,
            updated_at: record.updated_at,
        })
    }

    pub async fn get_document(pool: &PgPool, id: &str) -> DocumentResult<Document> {
        let record = sqlx::query!(
            r#"
            SELECT *
>>>>>>> 921251a (fetch)
            FROM documents
            WHERE id = $1
            "#,
            Uuid::parse_str(id)?
        )
<<<<<<< HEAD
        .fetch_one(&self.pool)
        .await?;

        Ok(record)
    }

    pub async fn list(
        &self,
        limit: i64,
        offset: i64,
        status: Option<String>,
        classification: Option<String>,
    ) -> DocumentResult<Vec<Document>> {
        let records = sqlx::query_as!(
            Document,
            r#"
            SELECT 
                id, name, content_type, size, path,
                status as "status: _",
                classification as "classification: _",
                security_level as "security_level: _",
                metadata,
                created_at, updated_at
            FROM documents
            WHERE ($3::text IS NULL OR status::text = $3)
              AND ($4::text IS NULL OR classification::text = $4)
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset,
            status,
            classification,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    pub async fn update(&self, document: &Document) -> DocumentResult<Document> {
        let metadata_json = serde_json::to_value(&document.metadata)?;
        let record = sqlx::query_as!(
            Document,
            r#"
            UPDATE documents
            SET name = $2,
                content_type = $3,
                size = $4,
                path = $5,
                status = $6,
                classification = $7,
                security_level = $8,
                metadata = $9,
                updated_at = $10
            WHERE id = $1
            RETURNING 
                id, name, content_type, size, path,
                status as "status: _",
                classification as "classification: _",
                security_level as "security_level: _",
                metadata,
                created_at, updated_at
            "#,
            document.id,
            document.name,
            document.content_type,
            document.size,
            document.path,
            document.status as _,
            document.classification as _,
            document.security_level as _,
            metadata_json,
            document.updated_at,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(record)
    }

    pub async fn delete(&self, id: &str) -> DocumentResult<()> {
=======
        .fetch_one(pool)
        .await?;

        Ok(Document {
            id: record.id,
            name: record.name,
            content_type: record.content_type,
            size: record.size,
            path: record.path,
            status: record.status.parse()?,
            metadata: serde_json::from_value(record.metadata)?,
            created_at: record.created_at,
            updated_at: record.updated_at,
        })
    }

    pub async fn update_document_status(
        pool: &PgPool,
        id: &str,
        status: DocumentStatus,
    ) -> DocumentResult<()> {
        sqlx::query!(
            r#"
            UPDATE documents
            SET status = $1, updated_at = NOW()
            WHERE id = $2
            "#,
            status as DocumentStatus,
            Uuid::parse_str(id)?
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn list_documents(
        pool: &PgPool,
        offset: i64,
        limit: i64,
        status: Option<DocumentStatus>,
    ) -> DocumentResult<Vec<Document>> {
        let records = sqlx::query!(
            r#"
            SELECT *
            FROM documents
            WHERE ($1::document_status IS NULL OR status = $1)
            ORDER BY created_at DESC
            OFFSET $2
            LIMIT $3
            "#,
            status as Option<DocumentStatus>,
            offset,
            limit,
            status as _
        )
        .fetch_all(pool)
        .await?;

        let mut documents = Vec::new();
        for record in records {
            documents.push(Document {
                id: record.id,
                name: record.name,
                content_type: record.content_type,
                size: record.size,
                path: record.path,
                status: record.status.parse()?,
                metadata: serde_json::from_value(record.metadata)?,
                created_at: record.created_at,
                updated_at: record.updated_at,
            });
        }

        Ok(documents)
    }

    pub async fn delete_document(pool: &PgPool, id: &str) -> DocumentResult<()> {
>>>>>>> 921251a (fetch)
        let result = sqlx::query!(
            r#"
            DELETE FROM documents
            WHERE id = $1
            "#,
            Uuid::parse_str(id)?
        )
<<<<<<< HEAD
        .execute(&self.pool)
=======
        .execute(pool)
>>>>>>> 921251a (fetch)
        .await?;

        if result.rows_affected() == 0 {
            return Err(DocumentError::NotFound("Document not found".to_string()));
        }

        Ok(())
    }
}
