use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Type;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "document_status", rename_all = "snake_case")]
pub enum DocumentStatus {
    Draft,
    PendingReview,
    Approved,
    Rejected,
    Archived,
}

impl FromStr for DocumentStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "draft" => Ok(DocumentStatus::Draft),
            "pending_review" => Ok(DocumentStatus::PendingReview),
            "approved" => Ok(DocumentStatus::Approved),
            "rejected" => Ok(DocumentStatus::Rejected),
            "archived" => Ok(DocumentStatus::Archived),
            _ => Err(format!("Invalid document status: {}", s)),
        }
    }
}

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
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub author: String,
    pub department: String,
    pub tags: Vec<String>,
    pub version: String,
    pub custom_fields: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: Uuid,
    pub name: String,
    pub content_type: String,
    pub size: i64,
    pub path: String,
    pub status: DocumentStatus,
    pub classification: DocumentClassification,
    pub security_level: SecurityLevel,
    pub metadata: DocumentMetadata,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Document {
    pub fn new(
        name: String,
        content_type: String,
        size: i64,
        path: String,
        metadata: DocumentMetadata,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            content_type,
            size,
            path,
            status: DocumentStatus::Draft,
            classification: DocumentClassification::Internal,
            security_level: SecurityLevel::Low,
            metadata,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}
