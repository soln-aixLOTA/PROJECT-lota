<<<<<<< HEAD
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
=======
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::postgres::PgHasArrayType;
>>>>>>> 921251a (fetch)
use sqlx::Type;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "document_status", rename_all = "snake_case")]
pub enum DocumentStatus {
    Draft,
<<<<<<< HEAD
    PendingReview,
    Approved,
    Rejected,
=======
    Pending,
    Processing,
    Completed,
    Failed,
>>>>>>> 921251a (fetch)
    Archived,
}

impl FromStr for DocumentStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "draft" => Ok(DocumentStatus::Draft),
<<<<<<< HEAD
            "pending_review" => Ok(DocumentStatus::PendingReview),
            "approved" => Ok(DocumentStatus::Approved),
            "rejected" => Ok(DocumentStatus::Rejected),
=======
            "pending" => Ok(DocumentStatus::Pending),
            "processing" => Ok(DocumentStatus::Processing),
            "completed" => Ok(DocumentStatus::Completed),
            "failed" => Ok(DocumentStatus::Failed),
>>>>>>> 921251a (fetch)
            "archived" => Ok(DocumentStatus::Archived),
            _ => Err(format!("Invalid document status: {}", s)),
        }
    }
}

<<<<<<< HEAD
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "document_classification", rename_all = "snake_case")]
pub enum DocumentClassification {
    Public,
    Internal,
    Confidential,
    Restricted,
}

impl FromStr for DocumentClassification {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "public" => Ok(DocumentClassification::Public),
            "internal" => Ok(DocumentClassification::Internal),
            "confidential" => Ok(DocumentClassification::Confidential),
            "restricted" => Ok(DocumentClassification::Restricted),
            _ => Err(format!("Invalid document classification: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "security_level", rename_all = "snake_case")]
pub enum SecurityLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl FromStr for SecurityLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "low" => Ok(SecurityLevel::Low),
            "medium" => Ok(SecurityLevel::Medium),
            "high" => Ok(SecurityLevel::High),
            "critical" => Ok(SecurityLevel::Critical),
            _ => Err(format!("Invalid security level: {}", s)),
        }
=======
impl From<String> for DocumentStatus {
    fn from(s: String) -> Self {
        s.parse().unwrap_or(DocumentStatus::Draft)
>>>>>>> 921251a (fetch)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
<<<<<<< HEAD
    pub author: String,
    pub department: String,
    pub tags: Vec<String>,
    pub version: String,
    pub custom_fields: serde_json::Value,
=======
    pub classification: Option<String>,
    pub security_level: Option<String>,
    pub custom_fields: Option<JsonValue>,
}

impl From<JsonValue> for DocumentMetadata {
    fn from(value: JsonValue) -> Self {
        serde_json::from_value(value).unwrap_or_default()
    }
}

impl From<Option<JsonValue>> for DocumentMetadata {
    fn from(value: Option<JsonValue>) -> Self {
        value.map(|v| v.into()).unwrap_or_default()
    }
}

impl Default for DocumentMetadata {
    fn default() -> Self {
        Self {
            classification: None,
            security_level: None,
            custom_fields: None,
        }
    }
>>>>>>> 921251a (fetch)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: Uuid,
    pub name: String,
    pub content_type: String,
    pub size: i64,
    pub path: String,
    pub status: DocumentStatus,
<<<<<<< HEAD
    pub classification: DocumentClassification,
    pub security_level: SecurityLevel,
    pub metadata: DocumentMetadata,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
=======
    pub metadata: DocumentMetadata,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
>>>>>>> 921251a (fetch)
}

impl Document {
    pub fn new(
        name: String,
        content_type: String,
        size: i64,
        path: String,
<<<<<<< HEAD
        metadata: DocumentMetadata,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
=======
        metadata: Option<DocumentMetadata>,
    ) -> Self {
        let id = Uuid::new_v4();
        let now = chrono::Utc::now();
        Self {
            id,
>>>>>>> 921251a (fetch)
            name,
            content_type,
            size,
            path,
            status: DocumentStatus::Draft,
<<<<<<< HEAD
            classification: DocumentClassification::Internal,
            security_level: SecurityLevel::Low,
            metadata,
            created_at: Utc::now(),
            updated_at: Utc::now(),
=======
            metadata: metadata.unwrap_or_default(),
            created_at: now,
            updated_at: now,
>>>>>>> 921251a (fetch)
        }
    }
}
