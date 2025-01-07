use sqlx::Type;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "document_status", rename_all = "snake_case")]
pub enum DocumentStatus {
    Draft,
    Archived,
}

impl FromStr for DocumentStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "draft" => Ok(DocumentStatus::Draft),
            "archived" => Ok(DocumentStatus::Archived),
            _ => Err(format!("Invalid document status: {}", s)),
        }
    }
}

    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: Uuid,
    pub name: String,
    pub content_type: String,
    pub size: i64,
    pub path: String,
    pub status: DocumentStatus,
}

impl Document {
    pub fn new(
        name: String,
        content_type: String,
        size: i64,
        path: String,
            name,
            content_type,
            size,
            path,
            status: DocumentStatus::Draft,
        }
    }
}
