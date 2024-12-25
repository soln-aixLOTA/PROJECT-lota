use std::sync::Arc;
use mockall::predicate::*;
use uuid::Uuid;

use crate::{
    error::ServiceError,
    models::permission::{Permission, CreatePermissionRequest, UpdatePermissionRequest},
    repositories::{
        permission_repository::MockPermissionRepository,
        audit_repository::MockAuditRepository,
    },
    services::permission_service::PermissionService,
};

#[tokio::test]
async fn test_create_permission_success() {
    let mut permission_repo = MockPermissionRepository::new();
    let mut audit_repo = MockAuditRepository::new();

    let tenant_id = Uuid::new_v4();
    let permission_id = Uuid::new_v4();

    let request = CreatePermissionRequest {
        tenant_id,
        name: "create_user".to_string(),
        description: Some("Permission to create users".to_string()),
        resource: "users".to_string(),
        action: "create".to_string(),
    };

    let permission = Permission {
        id: permission_id,
        tenant_id,
        name: request.name.clone(),
        description: request.description.clone(),
        resource: request.resource.clone(),
        action: request.action.clone(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    permission_repo
        .expect_get_permission_by_name()
        .with(eq(tenant_id), eq(&request.name))
        .returning(|_, _| Ok(None));

    permission_repo
        .expect_create_permission()
        .with(eq(&request))
        .returning(move |_| Ok(permission.clone()));

    audit_repo
        .expect_log_event()
        .returning(|_, _, _, _| Ok(()));

    let service = PermissionService::new(
        Arc::new(permission_repo),
        Arc::new(audit_repo),
    );

    let result = service.create_permission(tenant_id, request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_create_permission_duplicate_name() {
    let mut permission_repo = MockPermissionRepository::new();
    let audit_repo = MockAuditRepository::new();

    let tenant_id = Uuid::new_v4();
    let existing_permission = Permission {
        id: Uuid::new_v4(),
        tenant_id,
        name: "create_user".to_string(),
        description: None,
        resource: "users".to_string(),
        action: "create".to_string(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let request = CreatePermissionRequest {
        tenant_id,
        name: "create_user".to_string(),
        description: None,
        resource: "users".to_string(),
        action: "create".to_string(),
    };

    permission_repo
        .expect_get_permission_by_name()
        .with(eq(tenant_id), eq(&request.name))
        .returning(move |_, _| Ok(Some(existing_permission.clone())));

    let service = PermissionService::new(
        Arc::new(permission_repo),
        Arc::new(audit_repo),
    );

    let result = service.create_permission(tenant_id, request).await;
    assert!(matches!(result, Err(ServiceError::PermissionAlreadyExists(_))));
}

#[tokio::test]
async fn test_update_permission_success() {
    let mut permission_repo = MockPermissionRepository::new();
    let mut audit_repo = MockAuditRepository::new();

    let tenant_id = Uuid::new_v4();
    let permission_id = Uuid::new_v4();

    let existing_permission = Permission {
        id: permission_id,
        tenant_id,
        name: "create_user".to_string(),
        description: None,
        resource: "users".to_string(),
        action: "create".to_string(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let request = UpdatePermissionRequest {
        name: Some("manage_user".to_string()),
        description: Some("Permission to manage users".to_string()),
        resource: Some("users".to_string()),
        action: Some("manage".to_string()),
    };

    let updated_permission = Permission {
        id: permission_id,
        tenant_id,
        name: request.name.clone().unwrap(),
        description: request.description.clone(),
        resource: request.resource.clone().unwrap(),
        action: request.action.clone().unwrap(),
        created_at: existing_permission.created_at,
        updated_at: chrono::Utc::now(),
    };

    permission_repo
        .expect_get_permission()
        .with(eq(permission_id))
        .returning(move |_| Ok(Some(existing_permission.clone())));

    permission_repo
        .expect_get_permission_by_name()
        .with(eq(tenant_id), eq("manage_user"))
        .returning(|_, _| Ok(None));

    permission_repo
        .expect_update_permission()
        .with(eq(permission_id), eq(&request))
        .returning(move |_, _| Ok(updated_permission.clone()));

    audit_repo
        .expect_log_event()
        .returning(|_, _, _, _| Ok(()));

    let service = PermissionService::new(
        Arc::new(permission_repo),
        Arc::new(audit_repo),
    );

    let result = service.update_permission(permission_id, request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_delete_permission_success() {
    let mut permission_repo = MockPermissionRepository::new();
    let mut audit_repo = MockAuditRepository::new();

    let tenant_id = Uuid::new_v4();
    let permission_id = Uuid::new_v4();

    let permission = Permission {
        id: permission_id,
        tenant_id,
        name: "create_user".to_string(),
        description: None,
        resource: "users".to_string(),
        action: "create".to_string(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    permission_repo
        .expect_get_permission()
        .with(eq(permission_id))
        .returning(move |_| Ok(Some(permission.clone())));

    permission_repo
        .expect_delete_permission()
        .with(eq(permission_id))
        .returning(|_| Ok(()));

    audit_repo
        .expect_log_event()
        .returning(|_, _, _, _| Ok(()));

    let service = PermissionService::new(
        Arc::new(permission_repo),
        Arc::new(audit_repo),
    );

    let result = service.delete_permission(permission_id).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_check_user_has_permission_success() {
    let mut permission_repo = MockPermissionRepository::new();
    let audit_repo = MockAuditRepository::new();

    let user_id = Uuid::new_v4();
    let permission_name = "create_user";

    let permission = Permission {
        id: Uuid::new_v4(),
        tenant_id: Uuid::new_v4(),
        name: permission_name.to_string(),
        description: None,
        resource: "users".to_string(),
        action: "create".to_string(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    permission_repo
        .expect_get_user_permissions()
        .with(eq(user_id))
        .returning(move |_| Ok(vec![permission.clone()]));

    let service = PermissionService::new(
        Arc::new(permission_repo),
        Arc::new(audit_repo),
    );

    let result = service.check_user_has_permission(user_id, permission_name).await;
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[tokio::test]
async fn test_check_user_has_any_permission_success() {
    let mut permission_repo = MockPermissionRepository::new();
    let audit_repo = MockAuditRepository::new();

    let user_id = Uuid::new_v4();
    let permission_names = vec!["create_user".to_string(), "delete_user".to_string()];

    let permission = Permission {
        id: Uuid::new_v4(),
        tenant_id: Uuid::new_v4(),
        name: "create_user".to_string(),
        description: None,
        resource: "users".to_string(),
        action: "create".to_string(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    permission_repo
        .expect_get_user_permissions()
        .with(eq(user_id))
        .returning(move |_| Ok(vec![permission.clone()]));

    let service = PermissionService::new(
        Arc::new(permission_repo),
        Arc::new(audit_repo),
    );

    let result = service.check_user_has_any_permission(user_id, &permission_names).await;
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[tokio::test]
async fn test_check_user_has_all_permissions_success() {
    let mut permission_repo = MockPermissionRepository::new();
    let audit_repo = MockAuditRepository::new();

    let user_id = Uuid::new_v4();
    let permission_names = vec!["create_user".to_string(), "delete_user".to_string()];

    let permissions = vec![
        Permission {
            id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            name: "create_user".to_string(),
            description: None,
            resource: "users".to_string(),
            action: "create".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
        Permission {
            id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            name: "delete_user".to_string(),
            description: None,
            resource: "users".to_string(),
            action: "delete".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
    ];

    permission_repo
        .expect_get_user_permissions()
        .with(eq(user_id))
        .returning(move |_| Ok(permissions.clone()));

    let service = PermissionService::new(
        Arc::new(permission_repo),
        Arc::new(audit_repo),
    );

    let result = service.check_user_has_all_permissions(user_id, &permission_names).await;
    assert!(result.is_ok());
    assert!(result.unwrap());
} 