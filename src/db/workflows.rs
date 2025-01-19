use crate::{AppError, AppResult};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn get_workflow_by_id(pool: &PgPool, workflow_id: Uuid) -> AppResult<Option<Workflow>> {
    let workflow = sqlx::query_as!(
        Workflow,
        r#"
        SELECT id, name, description, status, created_at, updated_at
        FROM workflows
        WHERE id = $1
        "#,
        workflow_id
    )
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?;

    Ok(workflow)
}

#[derive(Debug, sqlx::FromRow)]
pub struct Workflow {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
