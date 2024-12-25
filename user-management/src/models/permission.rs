use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Permission {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub resource_type: String,
    pub action: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Validate, Deserialize)]
pub struct CreatePermissionRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    pub description: Option<String>,
    #[validate(length(min = 1, max = 100))]
    pub resource_type: String,
    #[validate(length(min = 1, max = 50))]
    pub action: String,
}

#[derive(Debug, Validate, Deserialize)]
pub struct UpdatePermissionRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,
    pub description: Option<String>,
    #[validate(length(min = 1, max = 100))]
    pub resource_type: Option<String>,
    #[validate(length(min = 1, max = 50))]
    pub action: Option<String>,
}

impl Permission {
    pub fn validate_name(name: &str) -> bool {
        // Permission names should be in format: resource.action (e.g., user.create)
        let parts: Vec<&str> = name.split('.').collect();
        if parts.len() != 2 {
            return false;
        }

        let (resource, action) = (parts[0], parts[1]);
        resource.chars().all(|c| c.is_ascii_lowercase() || c == '_')
            && action.chars().all(|c| c.is_ascii_lowercase() || c == '_')
    }

    pub fn is_reserved_permission(name: &str) -> bool {
        matches!(
            name,
            "user.create" | "user.read" | "user.update" | "user.delete" |
            "role.create" | "role.read" | "role.update" | "role.delete" |
            "bot.create" | "bot.read" | "bot.update" | "bot.delete"
        )
    }
}

// Helper struct for permission checks
#[derive(Debug, Clone, PartialEq)]
pub struct PermissionCheck {
    pub resource_type: String,
    pub action: String,
    pub resource_id: Option<Uuid>,
    pub tenant_id: Uuid,
}

impl PermissionCheck {
    pub fn new(resource_type: &str, action: &str, tenant_id: Uuid) -> Self {
        Self {
            resource_type: resource_type.to_string(),
            action: action.to_string(),
            resource_id: None,
            tenant_id,
        }
    }

    pub fn with_resource_id(mut self, resource_id: Uuid) -> Self {
        self.resource_id = Some(resource_id);
        self
    }

    pub fn to_permission_name(&self) -> String {
        format!("{}.{}", self.resource_type, self.action)
    }
} 