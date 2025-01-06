use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    core::error::DocumentError,
    db::WorkflowRepository,
    models::{Workflow, WorkflowMetadata, WorkflowStatus},
    AppState,
};

#[derive(Debug, Deserialize)]
pub struct CreateWorkflowRequest {
    pub name: String,
    pub description: Option<String>,
    pub metadata: Option<WorkflowMetadata>,
}

#[derive(Debug, Deserialize)]
pub struct ListWorkflowsQuery {
    pub offset: Option<i64>,
    pub limit: Option<i64>,
}

pub async fn create_workflow(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<CreateWorkflowRequest>,
) -> Result<impl IntoResponse, DocumentError> {
    let workflow = Workflow {
        id: Uuid::new_v4(),
        name: request.name,
        description: request.description,
        status: WorkflowStatus::Active,
        creator: auth.id,
        metadata: request.metadata.unwrap_or_default(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let workflow = WorkflowRepository::create(&state.pool, workflow).await?;
    Ok(Json(workflow))
}

pub async fn list_workflows(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(query): Query<ListWorkflowsQuery>,
) -> Result<impl IntoResponse, DocumentError> {
    let workflows = WorkflowRepository::list(
        &state.pool,
        query.offset.unwrap_or(0),
        query.limit.unwrap_or(10),
    )
    .await?;

    Ok(Json(workflows))
}

pub async fn get_workflow(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, DocumentError> {
    let workflow = WorkflowRepository::get(&state.pool, &id).await?;
    Ok(Json(workflow))
}

pub async fn update_workflow_status(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<String>,
    Json(status): Json<WorkflowStatus>,
) -> Result<impl IntoResponse, DocumentError> {
    WorkflowRepository::update_status(&state.pool, &id, status).await?;
    Ok(Json(()))
}

pub async fn delete_workflow(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, DocumentError> {
    WorkflowRepository::delete(&state.pool, &id).await?;
    Ok(Json(()))
}
