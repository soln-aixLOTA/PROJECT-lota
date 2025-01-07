use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::Claims;
use crate::core::AppState;
use crate::core::error::AppResult;
use crate::db::documents;
use crate::models::document::Document;

#[derive(Debug, Deserialize)]
pub struct CreateDocumentRequest {
    pub title: String,
    pub content: Option<String>,
    pub file: Option<String>, // Base64 encoded file content
}

#[derive(Debug, Serialize)]
pub struct CreateDocumentResponse {
    pub document: Document,
}

pub async fn create_document(
    state: web::Data<AppState>,
    claims: web::ReqData<Claims>,
    req: web::Json<CreateDocumentRequest>,
) -> AppResult<HttpResponse> {
    let file_path = match &req.file {
        Some(file_content) => {
            // Save file to storage
            let file_path = state
                .storage
                .save_file(&req.title, base64::decode(file_content)?)
                .await?;
            Some(file_path)
        }
        None => None,
    };

    let document = documents::create_document(
        &state.db,
        claims.sub,
        req.title.clone(),
        req.content.clone(),
        file_path,
    )
    .await?;

    Ok(HttpResponse::Created().json(CreateDocumentResponse { document }))
}

pub async fn get_document(
    state: web::Data<AppState>,
    claims: web::ReqData<Claims>,
    path: web::Path<Uuid>,
) -> AppResult<HttpResponse> {
    let document = documents::get_document(&state.db, path.into_inner(), claims.sub).await?;
    Ok(HttpResponse::Ok().json(document))
}
