use sqlx::PgPool;
use uuid::Uuid;

use crate::core::error::AppResult;
use crate::models::document::Document;

pub async fn create_document(
    pool: &PgPool,
    user_id: Uuid,
    title: String,
    content: Option<String>,
    file_path: Option<String>,
) -> AppResult<Document> {
    let document = sqlx::query_as!(
        Document,
        r#"
        INSERT INTO documents (user_id, title, content, file_path)
        VALUES ($1, $2, $3, $4)
        RETURNING id, user_id, title, content, file_path, created_at, updated_at
        "#,
        user_id,
        title,
        content,
        file_path
    )
    .fetch_one(pool)
    .await?;

    Ok(document)
}

pub async fn get_document(pool: &PgPool, id: Uuid, user_id: Uuid) -> AppResult<Document> {
    let document = sqlx::query_as!(
        Document,
        r#"
        SELECT id, user_id, title, content, file_path, created_at, updated_at
        FROM documents
        WHERE id = $1 AND user_id = $2
        "#,
        id,
        user_id
    )
    .fetch_one(pool)
    .await?;

    Ok(document)
}
