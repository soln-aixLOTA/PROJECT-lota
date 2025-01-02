use std::sync::Arc;
use mockall::predicate::*;
use uuid::Uuid;
use chrono::{Duration, Utc};

use crate::{
    error::ServiceError,
    models::user::{
        User, CreateUserRequest, UpdateUserRequest,
        ChangePasswordRequest, ResetPasswordRequest, LoginRequest,
    },
    repositories::{
        user_repository::MockUserRepository,
        role_repository::MockRoleRepository,
        permission_repository::MockPermissionRepository,
        audit_repository::MockAuditRepository,
    },
    services::user_service::UserService,
    config::JwtConfig,
};

#[tokio::test]
async fn test_register_user_success() {
    let mut user_repo = MockUserRepository::new();
    let role_repo = MockRoleRepository::new();
    let permission_repo = MockPermissionRepository::new();
    let mut audit_repo = MockAuditRepository::new();

    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let request = CreateUserRequest {
        tenant_id,
        email: "test@example.com".to_string(),
        password: "Password123!".to_string(),
        first_name: "Test".to_string(),
        last_name: "User".to_string(),
    };

    let user = User {
        id: user_id,
        tenant_id,
        email: request.email.clone(),
        password_hash: "hashed_password".to_string(),
        first_name: request.first_name.clone(),
        last_name: request.last_name.clone(),
        failed_login_attempts: 0,
        locked_until: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    user_repo
        .expect_get_user_by_email()
        .with(eq(tenant_id), eq(&request.email))
        .returning(|_, _| Ok(None));

    user_repo
        .expect_create_user()
        .returning(move |_| Ok(user.clone()));

    audit_repo
        .expect_log_event()
        .returning(|_, _, _, _| Ok(()));

    let service = UserService::new(
        Arc::new(user_repo),
        Arc::new(role_repo),
        Arc::new(permission_repo),
        Arc::new(audit_repo),
        JwtConfig {
            secret: "test_secret".to_string(),
            issuer: "test_issuer".to_string(),
        },
    );

    let result = service.register_user(tenant_id, request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_register_user_duplicate_email() {
    let mut user_repo = MockUserRepository::new();
    let role_repo = MockRoleRepository::new();
    let permission_repo = MockPermissionRepository::new();
    let audit_repo = MockAuditRepository::new();

    let tenant_id = Uuid::new_v4();
    let existing_user = User {
        id: Uuid::new_v4(),
        tenant_id,
        email: "test@example.com".to_string(),
        password_hash: "hashed_password".to_string(),
        first_name: "Test".to_string(),
        last_name: "User".to_string(),
        failed_login_attempts: 0,
        locked_until: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let request = CreateUserRequest {
        tenant_id,
        email: "test@example.com".to_string(),
        password: "Password123!".to_string(),
        first_name: "Test".to_string(),
        last_name: "User".to_string(),
    };

    user_repo
        .expect_get_user_by_email()
        .with(eq(tenant_id), eq(&request.email))
        .returning(move |_, _| Ok(Some(existing_user.clone())));

    let service = UserService::new(
        Arc::new(user_repo),
        Arc::new(role_repo),
        Arc::new(permission_repo),
        Arc::new(audit_repo),
        JwtConfig {
            secret: "test_secret".to_string(),
            issuer: "test_issuer".to_string(),
        },
    );

    let result = service.register_user(tenant_id, request).await;
    assert!(matches!(result, Err(ServiceError::EmailAlreadyExists(_))));
}

#[tokio::test]
async fn test_authenticate_user_success() {
    let mut user_repo = MockUserRepository::new();
    let mut role_repo = MockRoleRepository::new();
    let mut permission_repo = MockPermissionRepository::new();
    let mut audit_repo = MockAuditRepository::new();

    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let password = "Password123!";
    let password_hash = argon2::hash_encoded(
        password.as_bytes(),
        &rand::random::<[u8; 32]>(),
        &argon2::Config::default(),
    ).unwrap();

    let user = User {
        id: user_id,
        tenant_id,
        email: "test@example.com".to_string(),
        password_hash: password_hash.clone(),
        first_name: "Test".to_string(),
        last_name: "User".to_string(),
        failed_login_attempts: 0,
        locked_until: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let request = LoginRequest {
        email: "test@example.com".to_string(),
        password: password.to_string(),
    };

    user_repo
        .expect_get_user_by_email()
        .with(eq(tenant_id), eq(&request.email))
        .returning(move |_, _| Ok(Some(user.clone())));

    user_repo
        .expect_update_login_attempts()
        .returning(|_, _, _| Ok(()));

    role_repo
        .expect_get_user_roles()
        .returning(|_| Ok(vec![]));

    permission_repo
        .expect_get_user_permissions()
        .returning(|_| Ok(vec![]));

    audit_repo
        .expect_log_event()
        .returning(|_, _, _, _| Ok(()));

    let service = UserService::new(
        Arc::new(user_repo),
        Arc::new(role_repo),
        Arc::new(permission_repo),
        Arc::new(audit_repo),
        JwtConfig {
            secret: "test_secret".to_string(),
            issuer: "test_issuer".to_string(),
        },
    );

    let result = service.authenticate_user(tenant_id, request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_authenticate_user_invalid_credentials() {
    let mut user_repo = MockUserRepository::new();
    let role_repo = MockRoleRepository::new();
    let permission_repo = MockPermissionRepository::new();
    let mut audit_repo = MockAuditRepository::new();

    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let user = User {
        id: user_id,
        tenant_id,
        email: "test@example.com".to_string(),
        password_hash: "hashed_password".to_string(),
        first_name: "Test".to_string(),
        last_name: "User".to_string(),
        failed_login_attempts: 0,
        locked_until: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let request = LoginRequest {
        email: "test@example.com".to_string(),
        password: "wrong_password".to_string(),
    };

    user_repo
        .expect_get_user_by_email()
        .with(eq(tenant_id), eq(&request.email))
        .returning(move |_, _| Ok(Some(user.clone())));

    user_repo
        .expect_update_login_attempts()
        .returning(|_, _, _| Ok(()));

    audit_repo
        .expect_log_event()
        .returning(|_, _, _, _| Ok(()));

    let service = UserService::new(
        Arc::new(user_repo),
        Arc::new(role_repo),
        Arc::new(permission_repo),
        Arc::new(audit_repo),
        JwtConfig {
            secret: "test_secret".to_string(),
            issuer: "test_issuer".to_string(),
        },
    );

    let result = service.authenticate_user(tenant_id, request).await;
    assert!(matches!(result, Err(ServiceError::InvalidCredentials)));
}

#[tokio::test]
async fn test_change_password_success() {
    let mut user_repo = MockUserRepository::new();
    let role_repo = MockRoleRepository::new();
    let permission_repo = MockPermissionRepository::new();
    let mut audit_repo = MockAuditRepository::new();

    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let current_password = "Password123!";
    let password_hash = argon2::hash_encoded(
        current_password.as_bytes(),
        &rand::random::<[u8; 32]>(),
        &argon2::Config::default(),
    ).unwrap();

    let user = User {
        id: user_id,
        tenant_id,
        email: "test@example.com".to_string(),
        password_hash: password_hash.clone(),
        first_name: "Test".to_string(),
        last_name: "User".to_string(),
        failed_login_attempts: 0,
        locked_until: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let request = ChangePasswordRequest {
        current_password: current_password.to_string(),
        new_password: "NewPassword123!".to_string(),
    };

    user_repo
        .expect_get_user()
        .with(eq(user_id))
        .returning(move |_| Ok(Some(user.clone())));

    user_repo
        .expect_update_password()
        .returning(|_, _| Ok(()));

    audit_repo
        .expect_log_event()
        .returning(|_, _, _, _| Ok(()));

    let service = UserService::new(
        Arc::new(user_repo),
        Arc::new(role_repo),
        Arc::new(permission_repo),
        Arc::new(audit_repo),
        JwtConfig {
            secret: "test_secret".to_string(),
            issuer: "test_issuer".to_string(),
        },
    );

    let result = service.change_password(user_id, request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_reset_password_success() {
    let mut user_repo = MockUserRepository::new();
    let role_repo = MockRoleRepository::new();
    let permission_repo = MockPermissionRepository::new();
    let mut audit_repo = MockAuditRepository::new();

    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let reset_token = "valid_reset_token".to_string();

    let request = ResetPasswordRequest {
        reset_token: reset_token.clone(),
        new_password: "NewPassword123!".to_string(),
    };

    user_repo
        .expect_validate_reset_token()
        .with(eq(&reset_token))
        .returning(move |_| Ok(Some(user_id)));

    user_repo
        .expect_update_password()
        .returning(|_, _| Ok(()));

    user_repo
        .expect_clear_reset_token()
        .returning(|_| Ok(()));

    audit_repo
        .expect_log_event()
        .returning(|_, _, _, _| Ok(()));

    let service = UserService::new(
        Arc::new(user_repo),
        Arc::new(role_repo),
        Arc::new(permission_repo),
        Arc::new(audit_repo),
        JwtConfig {
            secret: "test_secret".to_string(),
            issuer: "test_issuer".to_string(),
        },
    );

    let result = service.reset_password(tenant_id, request).await;
    assert!(result.is_ok());
} 