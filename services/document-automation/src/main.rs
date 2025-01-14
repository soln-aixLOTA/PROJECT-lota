mod auth;
mod core;
mod db;
mod models;
mod storage;
mod error;

use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post, put},
    Router,
};
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use storage::LocalStorage;
use tokio::sync::Mutex;
use tower_http::limit::RequestBodyLimitLayer;
use tokio::net::TcpListener;
use error::DocumentError;

const MAX_FILE_SIZE: usize = 1_000_000; // 1MB limit
const ALLOWED_CONTENT_TYPES: [&str; 2] = ["text/plain", "application/pdf"];

#[derive(Clone)]
struct AppState {
    storage: Arc<Mutex<LocalStorage>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create storage
    let storage = Arc::new(Mutex::new(LocalStorage::new(
        PathBuf::from("/workspace/services/document-automation/uploads")
    )));

    // Create router
    let app = Router::new()
        .route("/health", get(health))
        .route("/documents", post(upload_document))
        .route("/documents/:id", get(get_document))
        .route("/documents/:id", put(update_document))
        .route("/documents/:id", delete(delete_document))
        .route("/documents", get(list_documents))
        .fallback(handle_404)
        .with_state(AppState { storage })
        .layer(RequestBodyLimitLayer::new(MAX_FILE_SIZE));

    // Run server
    let addr = SocketAddr::from(([0, 0, 0, 0], 8081));
    tracing::info!("Listening on {}", addr);
    
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health() -> &'static str {
    "OK"
}

async fn handle_404() -> impl IntoResponse {
    DocumentError::NotFound("Resource not found".to_string())
}

fn validate_filename(filename: Option<&str>) -> Result<(), DocumentError> {
    let filename = filename.ok_or_else(|| DocumentError::InvalidFilename("No filename provided".to_string()))?;
    
    if filename.is_empty() {
        return Err(DocumentError::InvalidFilename("Filename cannot be empty".to_string()));
    }

    if filename.contains(|c: char| c.is_control() || c == '/' || c == '\\') {
        return Err(DocumentError::InvalidFilename("Filename contains invalid characters".to_string()));
    }

    Ok(())
}

async fn upload_document(
    State(AppState { storage }): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, DocumentError> {
    tracing::info!("Processing file upload request");
    
    // Get the file field
    let field = multipart
        .next_field()
        .await
        .map_err(|e| DocumentError::ProcessingError(format!("Failed to process multipart: {}", e)))?
        .ok_or_else(|| DocumentError::InvalidFormat("No file field found in request".to_string()))?;

    // Validate filename if provided
    validate_filename(field.file_name())?;

    // Get and validate content type
    let content_type = field
        .content_type()
        .ok_or_else(|| DocumentError::InvalidFormat("No content type provided".to_string()))?;

    if !ALLOWED_CONTENT_TYPES.contains(&content_type) {
        return Err(DocumentError::InvalidContentType {
            found: content_type.to_string(),
            expected: ALLOWED_CONTENT_TYPES.iter().map(|s| s.to_string()).collect(),
        });
    }

    // Read the file data
    let data = field
        .bytes()
        .await
        .map_err(|e| DocumentError::ProcessingError(format!("Failed to read file data: {}", e)))?;

    if data.len() > MAX_FILE_SIZE {
        return Err(DocumentError::SizeExceeded {
            size: (data.len() as u64) / (1024 * 1024),
            limit: (MAX_FILE_SIZE as u64) / (1024 * 1024),
        });
    }

    // Generate a unique ID for the file
    let file_id = uuid::Uuid::new_v4().to_string();

    // Store the file
    storage
        .lock()
        .await
        .store(&file_id, &data)
        .await
        .map_err(|e| match e {
            DocumentError::FileOperation { operation, path, error } => {
                DocumentError::StorageError(format!("Failed to {} file at {}: {}", operation, path, error))
            }
            e => e,
        })?;

    Ok((StatusCode::CREATED, [("Location", format!("/documents/{}", file_id))]))
}

async fn get_document(
    State(AppState { storage }): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, DocumentError> {
    let data = storage
        .lock()
        .await
        .retrieve(&id)
        .await?;

    Ok((StatusCode::OK, [("Content-Type", "application/octet-stream")], data))
}

async fn update_document(
    State(AppState { storage }): State<AppState>,
    Path(id): Path<String>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, DocumentError> {
    // Check if document exists
    if !storage.lock().await.exists(&id).await? {
        return Err(DocumentError::NotFound(format!("Document {} not found", id)));
    }

    // Process the update similar to upload
    let field = multipart
        .next_field()
        .await
        .map_err(|e| DocumentError::ProcessingError(format!("Failed to process multipart: {}", e)))?
        .ok_or_else(|| DocumentError::InvalidFormat("No file field found in request".to_string()))?;

    // Validate filename if provided
    validate_filename(field.file_name())?;

    let content_type = field
        .content_type()
        .ok_or_else(|| DocumentError::InvalidFormat("No content type provided".to_string()))?;

    if !ALLOWED_CONTENT_TYPES.contains(&content_type) {
        return Err(DocumentError::InvalidContentType {
            found: content_type.to_string(),
            expected: ALLOWED_CONTENT_TYPES.iter().map(|s| s.to_string()).collect(),
        });
    }

    let data = field
        .bytes()
        .await
        .map_err(|e| DocumentError::ProcessingError(format!("Failed to read file data: {}", e)))?;

    if data.len() > MAX_FILE_SIZE {
        return Err(DocumentError::SizeExceeded {
            size: (data.len() as u64) / (1024 * 1024),
            limit: (MAX_FILE_SIZE as u64) / (1024 * 1024),
        });
    }

    // Update the file
    storage
        .lock()
        .await
        .store(&id, &data)
        .await
        .map_err(|e| match e {
            DocumentError::FileOperation { operation, path, error } => {
                DocumentError::StorageError(format!("Failed to {} file at {}: {}", operation, path, error))
            }
            e => e,
        })?;

    Ok(StatusCode::NO_CONTENT)
}

async fn delete_document(
    State(AppState { storage }): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, DocumentError> {
    storage.lock().await.delete(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn list_documents(
    State(AppState { storage }): State<AppState>,
) -> Result<impl IntoResponse, DocumentError> {
    let files = storage.lock().await.list().await?;
    Ok((StatusCode::OK, axum::Json(files)))
}
