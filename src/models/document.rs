use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, Copy, PartialEq, Eq)]
#[sqlx(type_name = "document_status", rename_all = "lowercase")]
pub enum DocumentStatus {
    pending,
    processing,
    completed,
    failed,
}

impl Default for DocumentStatus {
    fn default() -> Self {
        Self::pending
    }
}

impl From<String> for DocumentStatus {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "pending" => Self::pending,
            "processing" => Self::processing,
            "completed" => Self::completed,
            "failed" => Self::failed,
            _ => Self::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Document {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub content: Option<String>,
    pub file_path: Option<String>,
    pub content_type: Option<String>,
    pub size: Option<i64>,
    pub metadata: Option<serde_json::Value>,
    pub document_type: String,
    pub status: DocumentStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateDocumentRequest {
    #[validate(length(min = 1, max = 255))]
    pub title: String,
    pub content: Option<String>,
    pub file_path: Option<String>,
    pub content_type: Option<String>,
    pub size: Option<i64>,
    pub metadata: Option<DocumentMetadata>,
    #[validate(length(min = 1, max = 50))]
    pub document_type: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateDocumentRequest {
    #[validate(length(min = 1, max = 255))]
    pub title: Option<String>,
    pub content: Option<String>,
    pub file_path: Option<String>,
    pub content_type: Option<String>,
    pub size: Option<i64>,
    pub metadata: Option<DocumentMetadata>,
    #[validate(length(min = 1, max = 50))]
    pub document_type: Option<String>,
    pub status: Option<DocumentStatus>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentResponse {
    pub id: Uuid,
    pub title: String,
    pub content: Option<String>,
    pub file_path: Option<String>,
    pub content_type: Option<String>,
    pub size: Option<i64>,
    pub metadata: Option<serde_json::Value>,
    pub status: DocumentStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Document> for DocumentResponse {
    fn from(doc: Document) -> Self {
        Self {
            id: doc.id,
            title: doc.title,
            content: doc.content,
            file_path: doc.file_path,
            content_type: doc.content_type,
            size: doc.size,
            metadata: doc.metadata,
            status: doc.status,
            created_at: doc.created_at,
            updated_at: doc.updated_at,
        }
    }
}

impl Document {
    pub fn new(
        title: String,
        content_type: String,
        size: i64,
        file_path: String,
        metadata: Option<DocumentMetadata>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(), // This will be overwritten by the DB
            title,
            content: None,
            file_path: Some(file_path),
            content_type: Some(content_type),
            size: Some(size),
            metadata: metadata.map(|m| serde_json::to_value(m).unwrap_or_default()),
            document_type: String::new(),
            status: DocumentStatus::pending,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct DocumentMetadata {
    pub content_type: Option<String>,
    pub size: Option<i64>,
    pub tags: Vec<String>,
}
