use std::sync::Arc;
use mockall::predicate::*;
use uuid::Uuid;

use crate::{
    error::ServiceError,
    models::role::{Role, CreateRoleRequest, UpdateRoleRequest},
    repositories::{
        role_repository::MockRoleRepository,
        permission_repository::MockPermissionRepository,
        audit_repository::MockAuditRepository,
    },
    services::role_service::RoleService,
};

#[tokio::test]
async fn test_create_role_success() {
    let mut role_repo = MockRoleRepository::new();
    let mut permission_repo = MockPermissionRepository::new();
    let mut audit_repo = MockAuditRepository::new();

    let tenant_id = Uuid::new_v4();
    let role_id = Uuid::new_v4();
    let permission_id = Uuid::new_v4();

    let request = CreateRoleRequest {
        tenant_id,
        name: "admin".to_string(),
        description: Some("Administrator role".to_string()),
        permission_ids: vec![permission_id],
    };

    let role = Role {
        id: role_id,
        tenant_id,
        name: request.name.clone(),
        description: request.description.clone(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    role_repo
        .expect_get_role_by_name()
        .with(eq(tenant_id), eq(&request.name))
        .returning(|_, _| Ok(None));

    permission_repo
        .expect_get_permission()
        .with(eq(permission_id))
        .returning(move |_| Ok(Some(Default::default())));

    role_repo
        .expect_create_role()
        .with(eq(&request))
        .returning(move |_| Ok(role.clone()));

    role_repo
        .expect_assign_permission_to_role()
        .with(eq(role_id), eq(permission_id))
        .returning(|_, _| Ok(()));

    audit_repo
        .expect_log_event()
        .returning(|_, _, _, _| Ok(()));

    let service = RoleService::new(
        Arc::new(role_repo),
        Arc::new(permission_repo),
        Arc::new(audit_repo),
    );

    let result = service.create_role(tenant_id, request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_create_role_duplicate_name() {
    let mut role_repo = MockRoleRepository::new();
    let permission_repo = MockPermissionRepository::new();
    let audit_repo = MockAuditRepository::new();

    let tenant_id = Uuid::new_v4();
    let existing_role = Role {
        id: Uuid::new_v4(),
        tenant_id,
        name: "admin".to_string(),
        description: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let request = CreateRoleRequest {
        tenant_id,
        name: "admin".to_string(),
        description: None,
        permission_ids: vec![],
    };

    role_repo
        .expect_get_role_by_name()
        .with(eq(tenant_id), eq(&request.name))
        .returning(move |_, _| Ok(Some(existing_role.clone())));

    let service = RoleService::new(
        Arc::new(role_repo),
        Arc::new(permission_repo),
        Arc::new(audit_repo),
    );

    let result = service.create_role(tenant_id, request).await;
    assert!(matches!(result, Err(ServiceError::RoleAlreadyExists(_))));
}

#[tokio::test]
async fn test_update_role_success() {
    let mut role_repo = MockRoleRepository::new();
    let mut permission_repo = MockPermissionRepository::new();
    let mut audit_repo = MockAuditRepository::new();

    let tenant_id = Uuid::new_v4();
    let role_id = Uuid::new_v4();
    let permission_id = Uuid::new_v4();

    let existing_role = Role {
        id: role_id,
        tenant_id,
        name: "admin".to_string(),
        description: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let request = UpdateRoleRequest {
        name: Some("super_admin".to_string()),
        description: Some("Super Administrator role".to_string()),
        permission_ids: Some(vec![permission_id]),
    };

    let updated_role = Role {
        id: role_id,
        tenant_id,
        name: request.name.clone().unwrap(),
        description: request.description.clone(),
        created_at: existing_role.created_at,
        updated_at: chrono::Utc::now(),
    };

    role_repo
        .expect_get_role()
        .with(eq(role_id))
        .returning(move |_| Ok(Some(existing_role.clone())));

    role_repo
        .expect_get_role_by_name()
        .with(eq(tenant_id), eq("super_admin"))
        .returning(|_, _| Ok(None));

    permission_repo
        .expect_get_permission()
        .with(eq(permission_id))
        .returning(move |_| Ok(Some(Default::default())));

    role_repo
        .expect_update_role()
        .with(eq(role_id), eq(&request))
        .returning(move |_, _| Ok(updated_role.clone()));

    role_repo
        .expect_remove_role_permissions()
        .with(eq(role_id))
        .returning(|_| Ok(()));

    role_repo
        .expect_assign_permission_to_role()
        .with(eq(role_id), eq(permission_id))
        .returning(|_, _| Ok(()));

    audit_repo
        .expect_log_event()
        .returning(|_, _, _, _| Ok(()));

    let service = RoleService::new(
        Arc::new(role_repo),
        Arc::new(permission_repo),
        Arc::new(audit_repo),
    );

    let result = service.update_role(role_id, request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_delete_role_success() {
    let mut role_repo = MockRoleRepository::new();
    let mut audit_repo = MockAuditRepository::new();
    let permission_repo = MockPermissionRepository::new();

    let tenant_id = Uuid::new_v4();
    let role_id = Uuid::new_v4();

    let role = Role {
        id: role_id,
        tenant_id,
        name: "admin".to_string(),
        description: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    role_repo
        .expect_get_role()
        .with(eq(role_id))
        .returning(move |_| Ok(Some(role.clone())));

    role_repo
        .expect_delete_role()
        .with(eq(role_id))
        .returning(|_| Ok(()));

    audit_repo
        .expect_log_event()
        .returning(|_, _, _, _| Ok(()));

    let service = RoleService::new(
        Arc::new(role_repo),
        Arc::new(permission_repo),
        Arc::new(audit_repo),
    );

    let result = service.delete_role(role_id).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_assign_role_to_user_success() {
    let mut role_repo = MockRoleRepository::new();
    let mut audit_repo = MockAuditRepository::new();
    let permission_repo = MockPermissionRepository::new();

    let tenant_id = Uuid::new_v4();
    let role_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let role = Role {
        id: role_id,
        tenant_id,
        name: "admin".to_string(),
        description: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    role_repo
        .expect_get_role()
        .with(eq(role_id))
        .returning(move |_| Ok(Some(role.clone())));

    role_repo
        .expect_assign_role_to_user()
        .with(eq(user_id), eq(role_id))
        .returning(|_, _| Ok(()));

    audit_repo
        .expect_log_event()
        .returning(|_, _, _, _| Ok(()));

    let service = RoleService::new(
        Arc::new(role_repo),
        Arc::new(permission_repo),
        Arc::new(audit_repo),
    );

    let result = service.assign_role_to_user(role_id, user_id).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_remove_role_from_user_success() {
    let mut role_repo = MockRoleRepository::new();
    let mut audit_repo = MockAuditRepository::new();
    let permission_repo = MockPermissionRepository::new();

    let tenant_id = Uuid::new_v4();
    let role_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let role = Role {
        id: role_id,
        tenant_id,
        name: "admin".to_string(),
        description: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    role_repo
        .expect_get_role()
        .with(eq(role_id))
        .returning(move |_| Ok(Some(role.clone())));

    role_repo
        .expect_remove_role_from_user()
        .with(eq(user_id), eq(role_id))
        .returning(|_, _| Ok(()));

    audit_repo
        .expect_log_event()
        .returning(|_, _, _, _| Ok(()));

    let service = RoleService::new(
        Arc::new(role_repo),
        Arc::new(permission_repo),
        Arc::new(audit_repo),
    );

    let result = service.remove_role_from_user(role_id, user_id).await;
    assert!(result.is_ok());
} 