mod auth;
mod core;
mod db;
mod models;
mod storage;

use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response, Json},
    routing::{get, post, put},
    Router,
};
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use storage::LocalStorage;
use tokio::sync::Mutex;
use tower::BoxError;
use tower_http::limit::RequestBodyLimitLayer;
use futures_core::stream::Stream;

const MAX_FILE_SIZE: usize = 1_000_000; // 1MB limit
const ALLOWED_CONTENT_TYPES: [&str; 2] = ["text/plain", "application/pdf"];

#[derive(Clone)]
struct AppState {
    storage: Arc<Mutex<LocalStorage>>,
}

#[tokio::main]
async fn main() {
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
        .fallback(handle_404)
        .with_state(AppState { storage })
        .layer(RequestBodyLimitLayer::new(MAX_FILE_SIZE));

    // Run server
    let addr = SocketAddr::from(([0, 0, 0, 0], 8081));
    tracing::info!("Listening on {}", addr);
    
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn health() -> &'static str {
    "OK"
}

fn create_error_response(status: StatusCode, message: &str, code: &str) -> (StatusCode, Json<models::ErrorResponse>) {
    (
        status,
        Json(models::ErrorResponse {
            error: models::StandardError {
                status: status.as_u16(),
                message: message.to_string(),
                code: code.to_string(),
            },
        }),
    )
}

async fn upload_document(
    State(AppState { storage }): State<AppState>,
    mut multipart: Multipart,
) -> Response {
    tracing::info!("Processing file upload request");
    
    // Get the file field
    let Some(field) = multipart.next_field().await.map_err(|e| {
        tracing::error!("Error getting field: {}", e);
        create_error_response(
            StatusCode::BAD_REQUEST,
            &format!("Failed to process file upload: {}", e),
            "UPLOAD_ERROR"
        ).into_response()
    }).ok().flatten() else {
        tracing::warn!("No file field found in request");
        return create_error_response(
            StatusCode::BAD_REQUEST,
            "No file field found in request",
            "MISSING_FILE"
        ).into_response();
    };

    // Get and validate content type
    let content_type = match field.content_type() {
        Some(ct) => ct,
        None => {
            tracing::warn!("No content type provided");
            return create_error_response(
                StatusCode::BAD_REQUEST,
                "Content type must be specified",
                "MISSING_CONTENT_TYPE"
            ).into_response();
        }
    };

    tracing::info!("Received file upload with content type: '{}'", content_type);
    
    if !ALLOWED_CONTENT_TYPES.contains(&content_type) {
        tracing::warn!("Unsupported content type: '{}'. Allowed types: {:?}", content_type, ALLOWED_CONTENT_TYPES);
        return create_error_response(
            StatusCode::BAD_REQUEST,
            &format!("Unsupported content type: {}. Allowed types are: {:?}", content_type, ALLOWED_CONTENT_TYPES),
            "UNSUPPORTED_CONTENT_TYPE"
        ).into_response();
    }

    // Validate file extension
    if let Some(filename) = field.file_name() {
        let extension = std::path::Path::new(filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        let valid_extension = match content_type {
            "text/plain" => extension == "txt",
            "application/pdf" => extension == "pdf",
            _ => false,
        };

        if !valid_extension {
            tracing::warn!("Invalid file extension: '{}' for content type: '{}'", extension, content_type);
            return create_error_response(
                StatusCode::BAD_REQUEST,
                &format!("Invalid file extension: '{}' for content type: '{}'", extension, content_type),
                "INVALID_FILE_EXTENSION"
            ).into_response();
        }
    }

    // Read file content
    let content = match field.bytes().await {
        Ok(bytes) => {
            if bytes.len() > MAX_FILE_SIZE {
                tracing::warn!("File size {} exceeds maximum allowed size {}", bytes.len(), MAX_FILE_SIZE);
                return create_error_response(
                    StatusCode::BAD_REQUEST,
                    &format!("File size {} bytes exceeds maximum limit of {} bytes", bytes.len(), MAX_FILE_SIZE),
                    "FILE_TOO_LARGE"
                ).into_response();
            }
            bytes
        }
        Err(e) => {
            tracing::error!("Error reading file content: {}", e);
            return create_error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("Failed to read file content: {}", e),
                "READ_ERROR"
            ).into_response();
        }
    };

    // Save file
    let storage = storage.lock().await;
    match storage.save_file(content.to_vec()).await {
        Ok(document_id) => {
            tracing::info!("File saved successfully with ID: {}", document_id);
            (
                StatusCode::OK,
                Json(models::UploadResponse {
                    document_id,
                    message: "File uploaded successfully".to_string(),
                })
            ).into_response()
        }
        Err(e) => {
            tracing::error!("Error saving file: {}", e);
            create_error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("Failed to save file: {}", e),
                "SAVE_ERROR"
            ).into_response()
        }
    }
}

async fn get_document(
    State(AppState { storage }): State<AppState>,
    Path(id): Path<String>,
) -> Response {
    let storage = storage.lock().await;
    match storage.get_file(&id).await {
        Ok(content) => {
            tracing::info!("Successfully retrieved document: {}", id);
            content.into_response()
        }
        Err(e) => {
            tracing::error!("Error retrieving document {}: {}", id, e);
            create_error_response(
                StatusCode::NOT_FOUND,
                "Document not found",
                "DOCUMENT_NOT_FOUND"
            ).into_response()
        }
    }
}

async fn update_document(
    State(AppState { storage }): State<AppState>,
    Path(id): Path<String>,
    mut multipart: Multipart,
) -> Response {
    tracing::info!("Updating document with id: {}", id);
    
    // Get the file field
    let Some(field) = multipart.next_field().await.map_err(|e| {
        tracing::error!("Error getting field: {}", e);
        create_error_response(
            StatusCode::BAD_REQUEST,
            &format!("Failed to process file upload: {}", e),
            "UPLOAD_ERROR"
        ).into_response()
    }).ok().flatten() else {
        tracing::warn!("No file field found in request");
        return create_error_response(
            StatusCode::BAD_REQUEST,
            "No file field found in request",
            "MISSING_FILE"
        ).into_response();
    };

    // Get and validate content type
    let content_type = match field.content_type() {
        Some(ct) => ct,
        None => {
            tracing::warn!("No content type provided");
            return create_error_response(
                StatusCode::BAD_REQUEST,
                "Content type must be specified",
                "MISSING_CONTENT_TYPE"
            ).into_response();
        }
    };

    tracing::info!("Received file upload with content type: '{}'", content_type);
    
    if !ALLOWED_CONTENT_TYPES.contains(&content_type) {
        tracing::warn!("Unsupported content type: '{}'. Allowed types: {:?}", content_type, ALLOWED_CONTENT_TYPES);
        return create_error_response(
            StatusCode::BAD_REQUEST,
            &format!("Unsupported content type: {}. Allowed types are: {:?}", content_type, ALLOWED_CONTENT_TYPES),
            "UNSUPPORTED_CONTENT_TYPE"
        ).into_response();
    }

    // Read file content
    let content = match field.bytes().await {
        Ok(bytes) => {
            if bytes.len() > MAX_FILE_SIZE {
                tracing::warn!("File size {} exceeds maximum allowed size {}", bytes.len(), MAX_FILE_SIZE);
                return create_error_response(
                    StatusCode::BAD_REQUEST,
                    &format!("File size {} bytes exceeds maximum limit of {} bytes", bytes.len(), MAX_FILE_SIZE),
                    "FILE_TOO_LARGE"
                ).into_response();
            }
            bytes
        }
        Err(e) => {
            tracing::error!("Error reading file content: {}", e);
            return create_error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("Failed to read file content: {}", e),
                "READ_ERROR"
            ).into_response();
        }
    };

    // Update file
    let storage = storage.lock().await;
    match storage.update_file(&id, content.to_vec()).await {
        Ok(_) => {
            tracing::info!("File updated successfully: {}", id);
            (
                StatusCode::OK,
                Json(models::UploadResponse {
                    document_id: id,
                    message: "File updated successfully".to_string(),
                })
            ).into_response()
        }
        Err(e) => {
            tracing::error!("Error updating file: {}", e);
            if e.to_string().contains("not found") {
                create_error_response(
                    StatusCode::NOT_FOUND,
                    "Document not found",
                    "DOCUMENT_NOT_FOUND"
                ).into_response()
            } else {
                create_error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to update file",
                    "UPDATE_ERROR"
                ).into_response()
            }
        }
    }
}

async fn handle_404() -> Response {
    create_error_response(
        StatusCode::NOT_FOUND,
        "Endpoint not found",
        "ENDPOINT_NOT_FOUND"
    ).into_response()
}
