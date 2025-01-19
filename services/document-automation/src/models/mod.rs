use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub id: String,
    pub name: String,
    pub content_type: String,
    pub size: usize,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadResponse {
    pub document_id: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct StandardError {
    pub status: u16,
    pub message: String,
    pub code: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: StandardError,
} 