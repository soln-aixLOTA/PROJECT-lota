use axum::{
    extract::{Json, Path, Query, State},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    core::{error::DocumentError, AppState},
    db::workflows::WorkflowRepository,
    models::workflow::{Workflow, WorkflowStatus},
};

#[derive(Debug, Deserialize)]
pub struct CreateWorkflowRequest {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct ListWorkflowsQuery {
    pub offset: Option<i64>,
    pub limit: Option<i64>,
    pub status: Option<WorkflowStatus>,
}

pub async fn create_workflow(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<CreateWorkflowRequest>,
) -> Result<impl IntoResponse, DocumentError> {
    let workflow = Workflow::new(request.name, request.description, auth.user_id);
    let workflow = WorkflowRepository::create_workflow(&state.db, &workflow).await?;
    Ok(Json(workflow))
}

pub async fn list_workflows(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(query): Query<ListWorkflowsQuery>,
) -> Result<impl IntoResponse, DocumentError> {
    let workflows = WorkflowRepository::list_workflows(
        &state.db,
        query.offset.unwrap_or(0),
        query.limit.unwrap_or(10),
        query.status,
    )
    .await?;

    Ok(Json(workflows))
}

pub async fn get_workflow(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, DocumentError> {
    let workflow = WorkflowRepository::get_workflow(&state.db, &id).await?;
    Ok(Json(workflow))
}

pub async fn update_workflow_status(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<String>,
    Json(status): Json<WorkflowStatus>,
) -> Result<impl IntoResponse, DocumentError> {
    WorkflowRepository::update_workflow_status(&state.db, &id, status).await?;
    Ok(Json(()))
}

pub async fn delete_workflow(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, DocumentError> {
    WorkflowRepository::delete_workflow(&state.db, &id).await?;
    Ok(Json(()))
}
