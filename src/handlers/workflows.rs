use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::Claims;
use crate::core::AppState;
use crate::core::error::AppResult;
use crate::db::workflows;
use crate::models::workflow::Workflow;

#[derive(Debug, Deserialize)]
pub struct CreateWorkflowRequest {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct CreateWorkflowResponse {
    pub workflow: Workflow,
}

pub async fn create_workflow(
    state: web::Data<AppState>,
    claims: web::ReqData<Claims>,
    req: web::Json<CreateWorkflowRequest>,
) -> AppResult<HttpResponse> {
    let workflow = workflows::create_workflow(&state.db, claims.sub, req.name.clone()).await?;
    Ok(HttpResponse::Created().json(CreateWorkflowResponse { workflow }))
}

pub async fn get_workflow(
    state: web::Data<AppState>,
    claims: web::ReqData<Claims>,
    path: web::Path<Uuid>,
) -> AppResult<HttpResponse> {
    let workflow = workflows::get_workflow(&state.db, path.into_inner(), claims.sub).await?;
    Ok(HttpResponse::Ok().json(workflow))
}
