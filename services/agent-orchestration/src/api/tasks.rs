use crate::models::TaskStatus;
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

pub async fn list_tasks(
    State(state): State<AppState>,
) -> Result<impl axum::response::IntoResponse, StatusCode> {
    let tasks = state.agent_manager.list_tasks().await;
    Ok((StatusCode::OK, Json(tasks)))
}

pub async fn get_task(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl axum::response::IntoResponse, StatusCode> {
    let task = state
        .agent_manager
        .get_task(id)
        .await
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok((StatusCode::OK, Json(task)))
}

pub async fn update_task_status(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(status): Json<TaskStatus>,
) -> Result<impl axum::response::IntoResponse, StatusCode> {
    let task = state
        .agent_manager
        .update_task_status(id, status)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::OK, Json(task)))
}
