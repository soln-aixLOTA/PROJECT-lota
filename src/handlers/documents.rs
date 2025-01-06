use axum::{
    extract::{Multipart, Path, Query, State},
    response::Json,
    routing::{delete, get, post},
    Router,
};
use serde::Deserialize;
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

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", post(upload_document))
        .route("/", get(list_documents))
        .route("/:id", get(get_document))
        .route("/:id", delete(delete_document))
}

async fn upload_document(
    State(state): State<AppState>,
    _auth: AuthUser,
    mut multipart: Multipart,
) -> Result<Json<Document>, DocumentError> {
    let mut file_data = Vec::new();
    let mut metadata = DocumentMetadata::default();
    let mut name = String::new();
    let mut content_type = String::new();

    while let Some(field) = multipart.next_field().await? {
        let field_name = field.name().unwrap_or("").to_string();
        match field_name.as_str() {
            "file" => {
                name = field.file_name().unwrap_or("").to_string();
                content_type = field.content_type().unwrap_or("").to_string();
                file_data = field.bytes().await?.to_vec();
            }
            "metadata" => {
                let metadata_str = String::from_utf8(field.bytes().await?.to_vec())?;
                metadata = serde_json::from_str(&metadata_str)?;
            }
            _ => {}
        }
    }

    let document = Document::new(
        name,
        content_type,
        file_data.len() as i64,
        format!("documents/{}", Uuid::new_v4()),
        Some(metadata),
    );

    let document = DocumentRepository::create_document(&state.db, &document).await?;

    Ok(Json(document))
}

async fn list_documents(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(query): Query<ListDocumentsQuery>,
) -> Result<Json<Vec<Document>>, DocumentError> {
    let documents = DocumentRepository::list_documents(
        &state.db,
        query.offset.unwrap_or(0),
        query.limit.unwrap_or(10),
        query.status,
    )
    .await?;

    Ok(Json(documents))
}

async fn get_document(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<Document>, DocumentError> {
    let document = DocumentRepository::get_document(&state.db, &id).await?;
    Ok(Json(document))
}

async fn delete_document(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<()>, DocumentError> {
    DocumentRepository::delete_document(&state.db, &id).await?;
    Ok(Json(()))
}
