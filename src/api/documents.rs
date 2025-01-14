use axum::{
    extract::{Multipart, Query, State},
    response::{IntoResponse, Json},
};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    core::{error::DocumentError, AppState},
    db::documents::DocumentRepository,
    models::document::{Document, DocumentMetadata, DocumentStatus},
};

#[derive(Debug, Deserialize)]
pub struct ListDocumentsQuery {
    pub offset: Option<i64>,
    pub limit: Option<i64>,
    pub status: Option<DocumentStatus>,
}

lazy_static! {
    static ref ALLOWED_MIME_TYPES: HashSet<&'static str> = {
        let mut set = HashSet::new();
        set.insert("application/pdf");
        set.insert("application/msword");
        set.insert("application/vnd.openxmlformats-officedocument.wordprocessingml.document");
        set.insert("text/plain");
        set
    };
}

const MAX_FILE_SIZE: usize = 10 * 1024 * 1024; // 10MB
const MAX_FILENAME_LENGTH: usize = 255;

fn sanitize_filename(filename: &str) -> String {
    filename
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '-' || *c == '_')
        .take(MAX_FILENAME_LENGTH)
        .collect()
}

pub async fn upload_document(
    State(state): State<AppState>,
    auth: AuthUser,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, DocumentError> {
    let mut file_data = Vec::new();
    let mut metadata = DocumentMetadata::default();
    let mut name = String::new();
    let mut content_type = String::new();

    // Extract file data and metadata from multipart form
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| DocumentError::InvalidInput(e.to_string()))?
    {
        let field_name = field.name().unwrap_or("").to_string();
        match field_name.as_str() {
            "file" => {
                name = field
                    .file_name()
                    .ok_or_else(|| DocumentError::InvalidInput("Missing filename".to_string()))?
                    .to_string();

                content_type = field
                    .content_type()
                    .ok_or_else(|| DocumentError::InvalidInput("Missing content type".to_string()))?
                    .to_string();

                // Validate content type
                if !ALLOWED_MIME_TYPES.contains(content_type.as_str()) {
                    return Err(DocumentError::InvalidFileType(content_type));
                }

                file_data = field
                    .bytes()
                    .await
                    .map_err(|e| DocumentError::InvalidInput(e.to_string()))?
                    .to_vec();

                // Check file size
                if file_data.len() > MAX_FILE_SIZE {
                    return Err(DocumentError::FileTooLarge);
                }
            }
            "metadata" => {
                let metadata_str = String::from_utf8(
                    field
                        .bytes()
                        .await
                        .map_err(|e| DocumentError::InvalidInput(e.to_string()))?
                        .to_vec(),
                )?;
                metadata = serde_json::from_str(&metadata_str)?;
            }
            _ => {}
        }
    }

    // Sanitize filename
    let sanitized_name = sanitize_filename(&name);
    if sanitized_name.is_empty() {
        return Err(DocumentError::InvalidInput("Invalid filename".to_string()));
    }

    // Generate storage path
    let storage_path = format!("documents/{}/{}", auth.user_id, Uuid::new_v4());

    // Start transaction
    let mut tx = state.db.begin().await.map_err(DocumentError::Database)?;

    // Create document in database
    let document = Document::new(
        sanitized_name,
        content_type.clone(),
        file_data.len() as i64,
        storage_path.clone(),
        Some(metadata),
    );

    let document = match DocumentRepository::create_document(&mut tx, &document, auth.user_id).await
    {
        Ok(doc) => doc,
        Err(e) => {
            tx.rollback().await.map_err(DocumentError::Database)?;
            return Err(e);
        }
    };

    // Upload file to storage
    match state
        .storage
        .upload_file(&storage_path, &file_data, &content_type)
        .await
    {
        Ok(_) => {
            // Commit transaction if file upload succeeds
            tx.commit().await.map_err(DocumentError::Database)?;
            Ok(Json(document))
        }
        Err(e) => {
            // Rollback transaction if file upload fails
            tx.rollback().await.map_err(DocumentError::Database)?;
            Err(DocumentError::StorageError(e.to_string()))
        }
    }
}

pub async fn list_documents(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<ListDocumentsQuery>,
) -> Result<impl IntoResponse, DocumentError> {
    let documents = DocumentRepository::list_documents(
        &state.db,
        query.offset.unwrap_or(0),
        query.limit.unwrap_or(10),
        query.status,
        auth.user_id,
    )
    .await?;

    Ok(Json(documents))
}

pub async fn get_document(
    State(state): State<AppState>,
    auth: AuthUser,
    id: String,
) -> Result<impl IntoResponse, DocumentError> {
    let id = Uuid::parse_str(&id)
        .map_err(|_| DocumentError::InvalidInput("Invalid document ID".to_string()))?;

    let document = DocumentRepository::get_document(&state.db, id, auth.user_id).await?;
    Ok(Json(document))
}

pub async fn delete_document(
    State(state): State<AppState>,
    auth: AuthUser,
    id: String,
) -> Result<impl IntoResponse, DocumentError> {
    let id = Uuid::parse_str(&id)
        .map_err(|_| DocumentError::InvalidInput("Invalid document ID".to_string()))?;

    DocumentRepository::delete_document(&state.db, id, auth.user_id).await?;
    Ok(Json(()))
}
