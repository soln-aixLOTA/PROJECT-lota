use crate::models::{AgentRegistration, TaskAssignment};
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

pub async fn register_agent(
    State(state): State<AppState>,
    Json(registration): Json<AgentRegistration>,
) -> Result<impl axum::response::IntoResponse, StatusCode> {
    let agent = state
        .agent_manager
        .register_agent(registration)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::CREATED, Json(agent)))
}

pub async fn get_agent(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl axum::response::IntoResponse, StatusCode> {
    let agent = state
        .agent_manager
        .get_agent(id)
        .await
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok((StatusCode::OK, Json(agent)))
}

pub async fn assign_task(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(assignment): Json<TaskAssignment>,
) -> Result<impl axum::response::IntoResponse, StatusCode> {
    let task = state
        .agent_manager
        .assign_task(assignment)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::CREATED, Json(task)))
}
