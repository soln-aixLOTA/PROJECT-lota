use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Role {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_system_role: bool,
}

#[derive(Debug, Validate, Deserialize)]
pub struct CreateRoleRequest {
    pub tenant_id: Uuid,
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    pub description: Option<String>,
    pub permission_ids: Vec<Uuid>,
}

#[derive(Debug, Validate, Deserialize)]
pub struct UpdateRoleRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,
    pub description: Option<String>,
    pub permission_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Serialize)]
pub struct RoleWithPermissions {
    #[serde(flatten)]
    pub role: Role,
    pub permissions: Vec<String>,
}

impl Role {
    pub fn is_reserved_name(name: &str) -> bool {
        matches!(
            name,
            "tenant_admin" | "user_manager" | "bot_manager" | "bot_user"
        )
    }

    pub fn validate_name(name: &str) -> bool {
        // Role names should be lowercase alphanumeric with underscores
        name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    }
} 