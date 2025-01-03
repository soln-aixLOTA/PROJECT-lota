use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub tenant_id: Uuid,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing)]
    pub password_hash: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 2))]
    pub name: String,
    pub tenant_id: Uuid,
    #[validate(length(min = 8))]
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateUserRequest {
    #[validate(email)]
    pub email: Option<String>,
    #[validate(length(min = 2))]
    pub name: Option<String>,
    #[validate(length(min = 8))]
    pub password: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Tenant {
    pub id: Uuid,
    pub name: String,
    pub domain: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateTenantRequest {
    #[validate(length(min = 2))]
    pub name: String,
    #[validate(length(min = 3))]
    pub domain: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateTenantRequest {
    #[validate(length(min = 2))]
    pub name: Option<String>,
    #[validate(length(min = 3))]
    pub domain: Option<String>,
    pub is_active: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_user_request_validation() {
        let request = CreateUserRequest {
            email: "invalid-email".to_string(),
            name: "a".to_string(), // too short
            tenant_id: Uuid::new_v4(),
            password: "short".to_string(), // too short
        };

        assert!(request.validate().is_err());
    }

    #[test]
    fn test_create_tenant_request_validation() {
        let request = CreateTenantRequest {
            name: "a".to_string(),         // too short
            domain: Some("a".to_string()), // too short
        };

        assert!(request.validate().is_err());
    }
}
