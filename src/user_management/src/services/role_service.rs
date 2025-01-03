use std::sync::Arc;
use uuid::Uuid;

use crate::{
    error::ServiceError,
    models::role::{Role, CreateRoleRequest, UpdateRoleRequest, RoleWithPermissions},
    repositories::{
        role_repository::RoleRepository,
        permission_repository::PermissionRepository,
        audit_repository::AuditRepository,
    },
};

pub struct RoleService {
    role_repository: Arc<dyn RoleRepository>,
    permission_repository: Arc<dyn PermissionRepository>,
    audit_repository: Arc<dyn AuditRepository>,
}

impl RoleService {
    pub fn new(
        role_repository: Arc<dyn RoleRepository>,
        permission_repository: Arc<dyn PermissionRepository>,
        audit_repository: Arc<dyn AuditRepository>,
    ) -> Self {
        Self {
            role_repository,
            permission_repository,
            audit_repository,
        }
    }

    pub async fn create_role(
        &self,
        tenant_id: Uuid,
        request: CreateRoleRequest,
    ) -> Result<Role, ServiceError> {
        // Check if role name is unique within tenant
        if let Some(_) = self.role_repository
            .get_role_by_name(tenant_id, &request.name)
            .await?
        {
            return Err(ServiceError::RoleAlreadyExists(request.name));
        }

        // Validate that all permissions exist
        for permission_id in &request.permission_ids {
            if let None = self.permission_repository.get_permission(*permission_id).await? {
                return Err(ServiceError::PermissionNotFound(*permission_id));
            }
        }

        // Create role
        let role = self.role_repository.create_role(&request).await?;

        // Assign permissions to role
        for permission_id in request.permission_ids {
            self.role_repository
                .assign_permission_to_role(role.id, permission_id)
                .await?;
        }

        // Log role creation
        self.audit_repository.log_event(
            tenant_id,
            None,
            "role_created",
            serde_json::json!({
                "role_id": role.id,
                "name": role.name,
                "permission_ids": request.permission_ids,
            }),
        ).await?;

        Ok(role)
    }

    pub async fn get_role(
        &self,
        role_id: Uuid,
    ) -> Result<Option<RoleWithPermissions>, ServiceError> {
        // Get role
        let role = match self.role_repository.get_role(role_id).await? {
            Some(role) => role,
            None => return Ok(None),
        };

        // Get role permissions
        let permissions = self.role_repository.get_role_permissions(role_id).await?;

        Ok(Some(RoleWithPermissions {
            role,
            permissions,
        }))
    }

    pub async fn update_role(
        &self,
        role_id: Uuid,
        request: UpdateRoleRequest,
    ) -> Result<Role, ServiceError> {
        // Check if role exists
        let existing_role = self.role_repository
            .get_role(role_id)
            .await?
            .ok_or_else(|| ServiceError::RoleNotFound(role_id))?;

        // If name is being updated, check for uniqueness
        if let Some(ref name) = request.name {
            if name != &existing_role.name {
                if let Some(_) = self.role_repository
                    .get_role_by_name(existing_role.tenant_id, name)
                    .await?
                {
                    return Err(ServiceError::RoleAlreadyExists(name.clone()));
                }
            }
        }

        // If permissions are being updated, validate they exist
        if let Some(ref permission_ids) = request.permission_ids {
            for permission_id in permission_ids {
                if let None = self.permission_repository.get_permission(*permission_id).await? {
                    return Err(ServiceError::PermissionNotFound(*permission_id));
                }
            }
        }

        // Update role
        let role = self.role_repository.update_role(role_id, &request).await?;

        // Update permissions if provided
        if let Some(permission_ids) = request.permission_ids {
            // Remove existing permissions
            self.role_repository.remove_role_permissions(role_id).await?;

            // Assign new permissions
            for permission_id in permission_ids {
                self.role_repository
                    .assign_permission_to_role(role_id, permission_id)
                    .await?;
            }
        }

        // Log role update
        self.audit_repository.log_event(
            role.tenant_id,
            None,
            "role_updated",
            serde_json::json!({
                "role_id": role.id,
                "updates": request,
            }),
        ).await?;

        Ok(role)
    }

    pub async fn delete_role(
        &self,
        role_id: Uuid,
    ) -> Result<(), ServiceError> {
        // Check if role exists
        let role = self.role_repository
            .get_role(role_id)
            .await?
            .ok_or_else(|| ServiceError::RoleNotFound(role_id))?;

        // Delete role
        self.role_repository.delete_role(role_id).await?;

        // Log role deletion
        self.audit_repository.log_event(
            role.tenant_id,
            None,
            "role_deleted",
            serde_json::json!({
                "role_id": role_id,
                "name": role.name,
            }),
        ).await?;

        Ok(())
    }

    pub async fn list_roles(
        &self,
        tenant_id: Uuid,
        page: u32,
        per_page: u32,
    ) -> Result<Vec<Role>, ServiceError> {
        let roles = self.role_repository.list_roles(tenant_id).await?;
        Ok(roles)
    }

    pub async fn assign_role_to_user(
        &self,
        role_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), ServiceError> {
        // Check if role exists
        let role = self.role_repository
            .get_role(role_id)
            .await?
            .ok_or_else(|| ServiceError::RoleNotFound(role_id))?;

        // Assign role to user
        self.role_repository.assign_role_to_user(user_id, role_id).await?;

        // Log role assignment
        self.audit_repository.log_event(
            role.tenant_id,
            Some(user_id),
            "role_assigned",
            serde_json::json!({
                "role_id": role_id,
                "role_name": role.name,
            }),
        ).await?;

        Ok(())
    }

    pub async fn remove_role_from_user(
        &self,
        role_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), ServiceError> {
        // Check if role exists
        let role = self.role_repository
            .get_role(role_id)
            .await?
            .ok_or_else(|| ServiceError::RoleNotFound(role_id))?;

        // Remove role from user
        self.role_repository.remove_role_from_user(user_id, role_id).await?;

        // Log role removal
        self.audit_repository.log_event(
            role.tenant_id,
            Some(user_id),
            "role_removed",
            serde_json::json!({
                "role_id": role_id,
                "role_name": role.name,
            }),
        ).await?;

        Ok(())
    }

    pub async fn get_user_roles(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<Role>, ServiceError> {
        self.role_repository.get_user_roles(user_id).await
    }

    pub async fn get_role_permissions(
        &self,
        role_id: Uuid,
    ) -> Result<Vec<String>, ServiceError> {
        self.role_repository.get_role_permissions(role_id).await
    }
} 