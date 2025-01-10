use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use crate::error::AppResult;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Workflow {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub user_id: Uuid,
    pub document_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub async fn create_workflow(
    pool: &PgPool,
    name: String,
    description: Option<String>,
    user_id: Uuid,
    document_id: Uuid,
) -> AppResult<Workflow> {
    let workflow = sqlx::query_as!(
        Workflow,
        r#"
        INSERT INTO workflows (name, description, status, user_id, document_id)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, name, description, status, user_id, document_id, created_at, updated_at
        "#,
        name,
        description,
        "pending",  // Default status
        user_id,
        document_id,
    )
    .fetch_one(pool)
    .await?;

    Ok(workflow)
}

pub async fn get_workflow(
    pool: &PgPool,
    id: Uuid,
    user_id: Uuid,
) -> AppResult<Workflow> {
    let workflow = sqlx::query_as!(
        Workflow,
        r#"
        SELECT id, name, description, status, user_id, document_id, created_at, updated_at
        FROM workflows
        WHERE id = $1 AND user_id = $2
        "#,
        id,
        user_id,
    )
    .fetch_one(pool)
    .await?;

    Ok(workflow)
}

pub async fn list_workflows(
    pool: &PgPool,
    user_id: Uuid,
    limit: i64,
    offset: i64,
) -> AppResult<Vec<Workflow>> {
    let workflows = sqlx::query_as!(
        Workflow,
        r#"
        SELECT id, name, description, status, user_id, document_id, created_at, updated_at
        FROM workflows
        WHERE user_id = $1
        ORDER BY created_at DESC
        LIMIT $2
        OFFSET $3
        "#,
        user_id,
        limit,
        offset,
    )
    .fetch_all(pool)
    .await?;

    Ok(workflows)
}

pub async fn update_workflow_status(
    pool: &PgPool,
    id: Uuid,
    user_id: Uuid,
    status: String,
) -> AppResult<Workflow> {
    let workflow = sqlx::query_as!(
        Workflow,
        r#"
        UPDATE workflows
        SET status = $1
        WHERE id = $2 AND user_id = $3
        RETURNING id, name, description, status, user_id, document_id, created_at, updated_at
        "#,
        status,
        id,
        user_id,
    )
    .fetch_one(pool)
    .await?;

    Ok(workflow)
}
