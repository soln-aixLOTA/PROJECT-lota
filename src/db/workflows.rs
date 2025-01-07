use sqlx::PgPool;
use uuid::Uuid;

use crate::core::error::AppResult;
use crate::models::workflow::Workflow;

pub async fn create_workflow(
    pool: &PgPool,
    user_id: Uuid,
    name: String,
) -> AppResult<Workflow> {
    let workflow = sqlx::query_as!(
        Workflow,
        r#"
        INSERT INTO workflows (user_id, name, status)
        VALUES ($1, $2, 'draft')
        RETURNING id, user_id, name, status, created_at, updated_at
        "#,
        user_id,
        name
    )
    .fetch_one(pool)
    .await?;

    Ok(workflow)
}

pub async fn get_workflow(pool: &PgPool, id: Uuid, user_id: Uuid) -> AppResult<Workflow> {
    let workflow = sqlx::query_as!(
        Workflow,
        r#"
        SELECT id, user_id, name, status, created_at, updated_at
        FROM workflows
        WHERE id = $1 AND user_id = $2
        "#,
        id,
        user_id
    )
    .fetch_one(pool)
    .await?;

    Ok(workflow)
}
