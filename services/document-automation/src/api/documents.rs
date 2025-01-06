use axum::{
    extract::{Multipart, Query, State},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    core::{error::DocumentError, AppState},
    db::documents::DocumentRepository,
    models::{Document, DocumentMetadata, DocumentStatus},
};

#[derive(Debug, Deserialize)]
pub struct ListDocumentsQuery {
    pub offset: Option<i64>,
    pub limit: Option<i64>,
    pub status: Option<DocumentStatus>,
}

pub async fn upload_document(
    State(state): State<AppState>,
    _auth: AuthUser,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, DocumentError> {
    let mut file_data = Vec::new();
    let mut metadata = DocumentMetadata::default();

    while let Some(field) = multipart.next_field().await? {
        match field.name() {
            Some("file") => {
                metadata.filename = field.file_name().unwrap_or("unknown").to_string();
                metadata.content_type = field
                    .content_type()
                    .unwrap_or("application/octet-stream")
                    .to_string();
                file_data = field.bytes().await?.to_vec();
            }
            Some("metadata") => {
                let metadata_str = String::from_utf8_lossy(&field.bytes().await?);
                metadata = serde_json::from_str(&metadata_str)?;
            }
            _ => continue,
        }
    }

    let document = Document::new(metadata);

    // Upload file to storage
    state
        .storage
        .upload_file(&document.storage_path, file_data)
        .await?;

    // Save document metadata to database
    let document = DocumentRepository::create_document(&state.db, &document).await?;

    Ok(Json(document))
}

pub async fn list_documents(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(query): Query<ListDocumentsQuery>,
) -> Result<impl IntoResponse, DocumentError> {
    let documents = DocumentRepository::list_documents(
        &state.db,
        query.offset.unwrap_or(0),
        query.limit.unwrap_or(10),
        query.status,
    )
    .await?;

    Ok(Json(documents))
}

pub async fn get_document(
    State(state): State<AppState>,
    _auth: AuthUser,
    id: String,
) -> Result<impl IntoResponse, DocumentError> {
    let id = Uuid::parse_str(&id)?;
    let document = DocumentRepository::get_document(&state.db, id).await?;
    Ok(Json(document))
}

pub async fn delete_document(
    State(state): State<AppState>,
    _auth: AuthUser,
    id: String,
) -> Result<impl IntoResponse, DocumentError> {
    let id = Uuid::parse_str(&id)?;

    // Delete from storage first
    let document = DocumentRepository::get_document(&state.db, id).await?;
    state.storage.delete_file(&document.storage_path).await?;

    // Then delete from database
    DocumentRepository::delete_document(&state.db, id).await?;

    Ok(Json(()))
}
