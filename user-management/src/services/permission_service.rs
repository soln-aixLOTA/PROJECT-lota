use std::sync::Arc;
use uuid::Uuid;

use crate::{
    error::ServiceError,
    models::permission::{Permission, CreatePermissionRequest, UpdatePermissionRequest},
    repositories::{
        permission_repository::PermissionRepository,
        audit_repository::AuditRepository,
    },
};

pub struct PermissionService {
    permission_repository: Arc<dyn PermissionRepository>,
    audit_repository: Arc<dyn AuditRepository>,
}

impl PermissionService {
    pub fn new(
        permission_repository: Arc<dyn PermissionRepository>,
        audit_repository: Arc<dyn AuditRepository>,
    ) -> Self {
        Self {
            permission_repository,
            audit_repository,
        }
    }

    pub async fn create_permission(
        &self,
        tenant_id: Uuid,
        request: CreatePermissionRequest,
    ) -> Result<Permission, ServiceError> {
        // Check if permission name is unique within tenant
        if let Some(_) = self.permission_repository
            .get_permission_by_name(tenant_id, &request.name)
            .await?
        {
            return Err(ServiceError::PermissionAlreadyExists(request.name));
        }

        // Create permission
        let permission = self.permission_repository.create_permission(&request).await?;

        // Log permission creation
        self.audit_repository.log_event(
            tenant_id,
            None,
            "permission_created",
            serde_json::json!({
                "permission_id": permission.id,
                "name": permission.name,
                "description": permission.description,
            }),
        ).await?;

        Ok(permission)
    }

    pub async fn get_permission(
        &self,
        permission_id: Uuid,
    ) -> Result<Option<Permission>, ServiceError> {
        self.permission_repository.get_permission(permission_id).await
    }

    pub async fn update_permission(
        &self,
        permission_id: Uuid,
        request: UpdatePermissionRequest,
    ) -> Result<Permission, ServiceError> {
        // Check if permission exists
        let existing_permission = self.permission_repository
            .get_permission(permission_id)
            .await?
            .ok_or_else(|| ServiceError::PermissionNotFound(permission_id))?;

        // If name is being updated, check for uniqueness
        if let Some(ref name) = request.name {
            if name != &existing_permission.name {
                if let Some(_) = self.permission_repository
                    .get_permission_by_name(existing_permission.tenant_id, name)
                    .await?
                {
                    return Err(ServiceError::PermissionAlreadyExists(name.clone()));
                }
            }
        }

        // Update permission
        let permission = self.permission_repository
            .update_permission(permission_id, &request)
            .await?;

        // Log permission update
        self.audit_repository.log_event(
            permission.tenant_id,
            None,
            "permission_updated",
            serde_json::json!({
                "permission_id": permission.id,
                "updates": request,
            }),
        ).await?;

        Ok(permission)
    }

    pub async fn delete_permission(
        &self,
        permission_id: Uuid,
    ) -> Result<(), ServiceError> {
        // Check if permission exists
        let permission = self.permission_repository
            .get_permission(permission_id)
            .await?
            .ok_or_else(|| ServiceError::PermissionNotFound(permission_id))?;

        // Delete permission
        self.permission_repository.delete_permission(permission_id).await?;

        // Log permission deletion
        self.audit_repository.log_event(
            permission.tenant_id,
            None,
            "permission_deleted",
            serde_json::json!({
                "permission_id": permission_id,
                "name": permission.name,
            }),
        ).await?;

        Ok(())
    }

    pub async fn list_permissions(
        &self,
        tenant_id: Uuid,
        page: u32,
        per_page: u32,
    ) -> Result<Vec<Permission>, ServiceError> {
        let permissions = self.permission_repository.list_permissions(tenant_id).await?;
        Ok(permissions)
    }

    pub async fn get_user_permissions(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<Permission>, ServiceError> {
        self.permission_repository.get_user_permissions(user_id).await
    }

    pub async fn get_role_permissions(
        &self,
        role_id: Uuid,
    ) -> Result<Vec<Permission>, ServiceError> {
        self.permission_repository.get_role_permissions(role_id).await
    }

    pub async fn check_user_has_permission(
        &self,
        user_id: Uuid,
        permission_name: &str,
    ) -> Result<bool, ServiceError> {
        let user_permissions = self.get_user_permissions(user_id).await?;
        Ok(user_permissions.iter().any(|p| p.name == permission_name))
    }

    pub async fn check_user_has_any_permission(
        &self,
        user_id: Uuid,
        permission_names: &[String],
    ) -> Result<bool, ServiceError> {
        let user_permissions = self.get_user_permissions(user_id).await?;
        Ok(user_permissions.iter().any(|p| permission_names.contains(&p.name)))
    }

    pub async fn check_user_has_all_permissions(
        &self,
        user_id: Uuid,
        permission_names: &[String],
    ) -> Result<bool, ServiceError> {
        let user_permissions = self.get_user_permissions(user_id).await?;
        Ok(permission_names.iter().all(|name| {
            user_permissions.iter().any(|p| &p.name == name)
        }))
    }
} 