use std::sync::Arc;

use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    models::tenant::{
        CreateTenantRequest, SubscriptionTier, SupportLevel, TenantStatus, UpdateTenantRequest,
    },
    repositories::tenant_repository::{PostgresTenantRepository, TenantRepository},
    services::tenant_service::TenantService,
};

async fn setup_test_db() -> PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/lotabots_test".to_string());

    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database")
}

async fn create_test_tenant(tenant_service: &TenantService) -> Uuid {
    let request = CreateTenantRequest {
        name: "Test Tenant".to_string(),
        subscription_tier: SubscriptionTier::Professional,
        billing_email: "billing@test.com".to_string(),
        technical_contact_email: "tech@test.com".to_string(),
        custom_domain: Some("test.lotabots.ai".to_string()),
    };

    let tenant = tenant_service
        .create_tenant(request)
        .await
        .expect("Failed to create test tenant");

    tenant.id
}

#[tokio::test]
async fn test_create_tenant() {
    let pool = setup_test_db().await;
    let tenant_repository = Arc::new(PostgresTenantRepository::new(pool));
    let tenant_service = TenantService::new(tenant_repository);

    let request = CreateTenantRequest {
        name: "New Tenant".to_string(),
        subscription_tier: SubscriptionTier::Professional,
        billing_email: "billing@example.com".to_string(),
        technical_contact_email: "tech@example.com".to_string(),
        custom_domain: Some("example.lotabots.ai".to_string()),
    };

    let tenant = tenant_service
        .create_tenant(request)
        .await
        .expect("Failed to create tenant");

    assert_eq!(tenant.name, "New Tenant");
    assert_eq!(tenant.subscription_tier, SubscriptionTier::Professional);
    assert_eq!(tenant.status, TenantStatus::Active);
    assert_eq!(tenant.support_level, SupportLevel::Standard);
}

#[tokio::test]
async fn test_get_tenant_by_id() {
    let pool = setup_test_db().await;
    let tenant_repository = Arc::new(PostgresTenantRepository::new(pool));
    let tenant_service = TenantService::new(tenant_repository);

    let tenant_id = create_test_tenant(&tenant_service).await;

    let tenant = tenant_service
        .get_tenant_by_id(tenant_id)
        .await
        .expect("Failed to get tenant")
        .expect("Tenant not found");

    assert_eq!(tenant.id, tenant_id);
}

#[tokio::test]
async fn test_get_tenant_by_domain() {
    let pool = setup_test_db().await;
    let tenant_repository = Arc::new(PostgresTenantRepository::new(pool));
    let tenant_service = TenantService::new(tenant_repository);

    let tenant_id = create_test_tenant(&tenant_service).await;

    let tenant = tenant_service
        .get_tenant_by_domain("test.lotabots.ai")
        .await
        .expect("Failed to get tenant")
        .expect("Tenant not found");

    assert_eq!(tenant.id, tenant_id);
}

#[tokio::test]
async fn test_update_tenant() {
    let pool = setup_test_db().await;
    let tenant_repository = Arc::new(PostgresTenantRepository::new(pool));
    let tenant_service = TenantService::new(tenant_repository);

    let tenant_id = create_test_tenant(&tenant_service).await;

    let update_request = UpdateTenantRequest {
        name: Some("Updated Tenant".to_string()),
        subscription_tier: Some(SubscriptionTier::Enterprise),
        status: None,
        max_users: Some(50),
        max_bots: Some(20),
        max_requests_per_day: Some(50000),
        gpu_quota_minutes: Some(500),
        custom_domain: None,
        support_level: Some(SupportLevel::Premium),
        billing_email: None,
        technical_contact_email: None,
    };

    let updated_tenant = tenant_service
        .update_tenant(tenant_id, update_request)
        .await
        .expect("Failed to update tenant");

    assert_eq!(updated_tenant.name, "Updated Tenant");
    assert_eq!(updated_tenant.subscription_tier, SubscriptionTier::Enterprise);
    assert_eq!(updated_tenant.support_level, SupportLevel::Premium);
}

#[tokio::test]
async fn test_delete_tenant() {
    let pool = setup_test_db().await;
    let tenant_repository = Arc::new(PostgresTenantRepository::new(pool));
    let tenant_service = TenantService::new(tenant_repository);

    let tenant_id = create_test_tenant(&tenant_service).await;

    tenant_service
        .delete_tenant(tenant_id)
        .await
        .expect("Failed to delete tenant");

    let tenant = tenant_service
        .get_tenant_by_id(tenant_id)
        .await
        .expect("Failed to get tenant");

    assert!(tenant.is_none());
}

#[tokio::test]
async fn test_list_tenants() {
    let pool = setup_test_db().await;
    let tenant_repository = Arc::new(PostgresTenantRepository::new(pool));
    let tenant_service = TenantService::new(tenant_repository);

    // Create multiple tenants
    create_test_tenant(&tenant_service).await;
    create_test_tenant(&tenant_service).await;

    let tenants = tenant_service
        .list_tenants()
        .await
        .expect("Failed to list tenants");

    assert!(tenants.len() >= 2);
}

#[tokio::test]
async fn test_get_tenant_usage() {
    let pool = setup_test_db().await;
    let tenant_repository = Arc::new(PostgresTenantRepository::new(pool));
    let tenant_service = TenantService::new(tenant_repository);

    let tenant_id = create_test_tenant(&tenant_service).await;

    let usage = tenant_service
        .get_tenant_usage(tenant_id)
        .await
        .expect("Failed to get tenant usage")
        .expect("Usage not found");

    assert_eq!(usage.tenant_id, tenant_id);
    assert_eq!(usage.current_user_count, 0);
    assert_eq!(usage.current_bot_count, 0);
    assert_eq!(usage.requests_today, 0);
    assert_eq!(usage.gpu_minutes_used, 0);
}

#[tokio::test]
async fn test_check_tenant_limits() {
    let pool = setup_test_db().await;
    let tenant_repository = Arc::new(PostgresTenantRepository::new(pool));
    let tenant_service = TenantService::new(tenant_repository);

    let tenant_id = create_test_tenant(&tenant_service).await;

    let within_limits = tenant_service
        .check_tenant_limits(tenant_id)
        .await
        .expect("Failed to check tenant limits");

    assert!(within_limits);
}

#[tokio::test]
async fn test_get_tenant_quota() {
    let pool = setup_test_db().await;
    let tenant_repository = Arc::new(PostgresTenantRepository::new(pool));
    let tenant_service = TenantService::new(tenant_repository);

    let tenant_id = create_test_tenant(&tenant_service).await;

    let quota = tenant_service
        .get_tenant_quota(tenant_id)
        .await
        .expect("Failed to get tenant quota");

    assert_eq!(quota.max_users, 20); // Professional tier
    assert_eq!(quota.max_bots, 10);
    assert_eq!(quota.max_requests_per_day, 10000);
    assert_eq!(quota.gpu_quota_minutes, 300);
} 