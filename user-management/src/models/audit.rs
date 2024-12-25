use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::net::IpAddr;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuditLog {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub user_id: Option<Uuid>,
    pub event_type: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub action: AuditAction,
    pub details: Option<Value>,
    pub ip_address: Option<IpAddr>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AuditAction {
    Create,
    Read,
    Update,
    Delete,
    Login,
    Logout,
    FailedLogin,
}

#[derive(Debug)]
pub struct AuditLogBuilder {
    tenant_id: Uuid,
    user_id: Option<Uuid>,
    event_type: String,
    resource_type: String,
    resource_id: Option<Uuid>,
    action: AuditAction,
    details: Option<Value>,
    ip_address: Option<IpAddr>,
    user_agent: Option<String>,
}

impl AuditLogBuilder {
    pub fn new(tenant_id: Uuid, event_type: &str, resource_type: &str, action: AuditAction) -> Self {
        Self {
            tenant_id,
            user_id: None,
            event_type: event_type.to_string(),
            resource_type: resource_type.to_string(),
            resource_id: None,
            action,
            details: None,
            ip_address: None,
            user_agent: None,
        }
    }

    pub fn with_user_id(mut self, user_id: Uuid) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_resource_id(mut self, resource_id: Uuid) -> Self {
        self.resource_id = Some(resource_id);
        self
    }

    pub fn with_details(mut self, details: Value) -> Self {
        self.details = Some(details);
        self
    }

    pub fn with_ip_address(mut self, ip_address: IpAddr) -> Self {
        self.ip_address = Some(ip_address);
        self
    }

    pub fn with_user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = Some(user_agent);
        self
    }

    pub fn build(self) -> AuditLog {
        AuditLog {
            id: Uuid::new_v4(),
            tenant_id: self.tenant_id,
            user_id: self.user_id,
            event_type: self.event_type,
            resource_type: self.resource_type,
            resource_id: self.resource_id,
            action: self.action,
            details: self.details,
            ip_address: self.ip_address,
            user_agent: self.user_agent,
            created_at: Utc::now(),
        }
    }
}

// Helper functions for common audit events
impl AuditLog {
    pub fn user_login(
        tenant_id: Uuid,
        user_id: Uuid,
        success: bool,
        ip_address: Option<IpAddr>,
        user_agent: Option<String>,
    ) -> Self {
        let action = if success {
            AuditAction::Login
        } else {
            AuditAction::FailedLogin
        };

        let mut builder = AuditLogBuilder::new(tenant_id, "auth", "user", action)
            .with_user_id(user_id);

        if let Some(ip) = ip_address {
            builder = builder.with_ip_address(ip);
        }

        if let Some(ua) = user_agent {
            builder = builder.with_user_agent(ua);
        }

        builder.build()
    }

    pub fn user_logout(tenant_id: Uuid, user_id: Uuid) -> Self {
        AuditLogBuilder::new(tenant_id, "auth", "user", AuditAction::Logout)
            .with_user_id(user_id)
            .build()
    }

    pub fn resource_access(
        tenant_id: Uuid,
        user_id: Uuid,
        resource_type: &str,
        resource_id: Uuid,
        action: AuditAction,
        details: Option<Value>,
    ) -> Self {
        let mut builder = AuditLogBuilder::new(tenant_id, "access", resource_type, action)
            .with_user_id(user_id)
            .with_resource_id(resource_id);

        if let Some(d) = details {
            builder = builder.with_details(d);
        }

        builder.build()
    }
} 