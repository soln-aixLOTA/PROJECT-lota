use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    core::{error::AppError, AppState},
    db::documents::DocumentRepository,
    models::document::{Document, DocumentMetadata, DocumentStatus},
};

#[derive(Debug, Deserialize)]
pub struct CreateDocumentRequest {
    pub title: String,
    pub content: String,
    pub metadata: Option<DocumentMetadata>,
}

#[derive(Debug, Serialize)]
pub struct DocumentResponse {
    pub id: Uuid,
    pub title: String,
    pub status: DocumentStatus,
    pub metadata: Option<DocumentMetadata>,
}

pub async fn create_document(
    State(db): State<PgPool>,
    auth_user: AuthUser,
    Json(payload): Json<CreateDocumentRequest>,
) -> Result<(StatusCode, Json<DocumentResponse>), AppError> {
    let document = Document::new(
        payload.title,
        payload.content,
        auth_user.id,
        payload.metadata,
    );

    let repo = DocumentRepository::new(&db);
    let saved_doc = repo.create(document).await?;

    Ok((
        StatusCode::CREATED,
        Json(DocumentResponse {
            id: saved_doc.id,
            title: saved_doc.title,
            status: saved_doc.status,
            metadata: saved_doc.metadata,
        }),
    ))
}

pub async fn get_document(
    State(db): State<PgPool>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<DocumentResponse>, AppError> {
    let repo = DocumentRepository::new(&db);
    let doc = repo.get_by_id(id).await?;

    if doc.user_id != auth_user.id {
        return Err(AppError::AuthenticationError);
    }

    Ok(Json(DocumentResponse {
        id: doc.id,
        title: doc.title,
        status: doc.status,
        metadata: doc.metadata,
    }))
}

pub async fn list_documents(
    State(db): State<PgPool>,
    auth_user: AuthUser,
) -> Result<Json<Vec<DocumentResponse>>, AppError> {
    let repo = DocumentRepository::new(&db);
    let docs = repo.list_by_user(auth_user.id).await?;

    let responses = docs
        .into_iter()
        .map(|doc| DocumentResponse {
            id: doc.id,
            title: doc.title,
            status: doc.status,
            metadata: doc.metadata,
        })
        .collect();

    Ok(Json(responses))
}
