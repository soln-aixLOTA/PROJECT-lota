use actix_web::{test, web, App};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    handlers::tenant_handlers,
    models::tenant::{
        CreateTenantRequest, SubscriptionTier, SupportLevel, TenantStatus, UpdateTenantRequest,
    },
    repositories::tenant_repository::PostgresTenantRepository,
    services::tenant_service::TenantService,
};

async fn setup_test_app() -> (test::TestApp, PgPool) {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/lotabots_test".to_string());

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    let tenant_repository = web::Data::new(PostgresTenantRepository::new(pool.clone()));
    let tenant_service = web::Data::new(TenantService::new(tenant_repository.into_inner()));

    let app = test::init_service(
        App::new()
            .app_data(tenant_service.clone())
            .configure(tenant_handlers::configure),
    )
    .await;

    (app, pool)
}

#[actix_web::test]
async fn test_create_tenant_endpoint() {
    let (app, _) = setup_test_app().await;

    let request = CreateTenantRequest {
        name: "Test Tenant".to_string(),
        subscription_tier: SubscriptionTier::Professional,
        billing_email: "billing@test.com".to_string(),
        technical_contact_email: "tech@test.com".to_string(),
        custom_domain: Some("test.lotabots.ai".to_string()),
    };

    let req = test::TestRequest::post()
        .uri("/")
        .set_json(&request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let tenant: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(tenant["name"], "Test Tenant");
    assert_eq!(tenant["subscription_tier"], "professional");
    assert_eq!(tenant["status"], "active");
}

#[actix_web::test]
async fn test_get_tenant_endpoint() {
    let (app, _) = setup_test_app().await;

    // First create a tenant
    let request = CreateTenantRequest {
        name: "Test Tenant".to_string(),
        subscription_tier: SubscriptionTier::Professional,
        billing_email: "billing@test.com".to_string(),
        technical_contact_email: "tech@test.com".to_string(),
        custom_domain: Some("test.lotabots.ai".to_string()),
    };

    let req = test::TestRequest::post()
        .uri("/")
        .set_json(&request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    let tenant: serde_json::Value = test::read_body_json(resp).await;
    let tenant_id = tenant["id"].as_str().unwrap();

    // Then get the tenant
    let req = test::TestRequest::get()
        .uri(&format!("/{}", tenant_id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let tenant: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(tenant["name"], "Test Tenant");
}

#[actix_web::test]
async fn test_get_tenant_by_domain_endpoint() {
    let (app, _) = setup_test_app().await;

    // First create a tenant
    let request = CreateTenantRequest {
        name: "Test Tenant".to_string(),
        subscription_tier: SubscriptionTier::Professional,
        billing_email: "billing@test.com".to_string(),
        technical_contact_email: "tech@test.com".to_string(),
        custom_domain: Some("test.lotabots.ai".to_string()),
    };

    let req = test::TestRequest::post()
        .uri("/")
        .set_json(&request)
        .to_request();

    test::call_service(&app, req).await;

    // Then get the tenant by domain
    let req = test::TestRequest::get()
        .uri("/domain/test.lotabots.ai")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let tenant: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(tenant["name"], "Test Tenant");
}

#[actix_web::test]
async fn test_update_tenant_endpoint() {
    let (app, _) = setup_test_app().await;

    // First create a tenant
    let request = CreateTenantRequest {
        name: "Test Tenant".to_string(),
        subscription_tier: SubscriptionTier::Professional,
        billing_email: "billing@test.com".to_string(),
        technical_contact_email: "tech@test.com".to_string(),
        custom_domain: Some("test.lotabots.ai".to_string()),
    };

    let req = test::TestRequest::post()
        .uri("/")
        .set_json(&request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    let tenant: serde_json::Value = test::read_body_json(resp).await;
    let tenant_id = tenant["id"].as_str().unwrap();

    // Then update the tenant
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

    let req = test::TestRequest::put()
        .uri(&format!("/{}", tenant_id))
        .set_json(&update_request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let tenant: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(tenant["name"], "Updated Tenant");
    assert_eq!(tenant["subscription_tier"], "enterprise");
}

#[actix_web::test]
async fn test_delete_tenant_endpoint() {
    let (app, _) = setup_test_app().await;

    // First create a tenant
    let request = CreateTenantRequest {
        name: "Test Tenant".to_string(),
        subscription_tier: SubscriptionTier::Professional,
        billing_email: "billing@test.com".to_string(),
        technical_contact_email: "tech@test.com".to_string(),
        custom_domain: Some("test.lotabots.ai".to_string()),
    };

    let req = test::TestRequest::post()
        .uri("/")
        .set_json(&request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    let tenant: serde_json::Value = test::read_body_json(resp).await;
    let tenant_id = tenant["id"].as_str().unwrap();

    // Then delete the tenant
    let req = test::TestRequest::delete()
        .uri(&format!("/{}", tenant_id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 204);

    // Verify tenant is deleted
    let req = test::TestRequest::get()
        .uri(&format!("/{}", tenant_id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}

#[actix_web::test]
async fn test_list_tenants_endpoint() {
    let (app, _) = setup_test_app().await;

    // Create multiple tenants
    for i in 1..=3 {
        let request = CreateTenantRequest {
            name: format!("Test Tenant {}", i),
            subscription_tier: SubscriptionTier::Professional,
            billing_email: format!("billing{}@test.com", i),
            technical_contact_email: format!("tech{}@test.com", i),
            custom_domain: Some(format!("test{}.lotabots.ai", i)),
        };

        let req = test::TestRequest::post()
            .uri("/")
            .set_json(&request)
            .to_request();

        test::call_service(&app, req).await;
    }

    // List tenants
    let req = test::TestRequest::get().uri("/").to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let tenants: Vec<serde_json::Value> = test::read_body_json(resp).await;
    assert!(tenants.len() >= 3);
}

#[actix_web::test]
async fn test_get_tenant_usage_endpoint() {
    let (app, _) = setup_test_app().await;

    // First create a tenant
    let request = CreateTenantRequest {
        name: "Test Tenant".to_string(),
        subscription_tier: SubscriptionTier::Professional,
        billing_email: "billing@test.com".to_string(),
        technical_contact_email: "tech@test.com".to_string(),
        custom_domain: Some("test.lotabots.ai".to_string()),
    };

    let req = test::TestRequest::post()
        .uri("/")
        .set_json(&request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    let tenant: serde_json::Value = test::read_body_json(resp).await;
    let tenant_id = tenant["id"].as_str().unwrap();

    // Get tenant usage
    let req = test::TestRequest::get()
        .uri(&format!("/{}/usage", tenant_id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let usage: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(usage["tenant_id"], tenant_id);
}

#[actix_web::test]
async fn test_get_tenant_quota_endpoint() {
    let (app, _) = setup_test_app().await;

    // First create a tenant
    let request = CreateTenantRequest {
        name: "Test Tenant".to_string(),
        subscription_tier: SubscriptionTier::Professional,
        billing_email: "billing@test.com".to_string(),
        technical_contact_email: "tech@test.com".to_string(),
        custom_domain: Some("test.lotabots.ai".to_string()),
    };

    let req = test::TestRequest::post()
        .uri("/")
        .set_json(&request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    let tenant: serde_json::Value = test::read_body_json(resp).await;
    let tenant_id = tenant["id"].as_str().unwrap();

    // Get tenant quota
    let req = test::TestRequest::get()
        .uri(&format!("/{}/quota", tenant_id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let quota: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(quota["max_users"], 20); // Professional tier
    assert_eq!(quota["max_bots"], 10);
    assert_eq!(quota["max_requests_per_day"], 10000);
    assert_eq!(quota["gpu_quota_minutes"], 300);
}

#[actix_web::test]
async fn test_check_tenant_limits_endpoint() {
    let (app, _) = setup_test_app().await;

    // First create a tenant
    let request = CreateTenantRequest {
        name: "Test Tenant".to_string(),
        subscription_tier: SubscriptionTier::Professional,
        billing_email: "billing@test.com".to_string(),
        technical_contact_email: "tech@test.com".to_string(),
        custom_domain: Some("test.lotabots.ai".to_string()),
    };

    let req = test::TestRequest::post()
        .uri("/")
        .set_json(&request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    let tenant: serde_json::Value = test::read_body_json(resp).await;
    let tenant_id = tenant["id"].as_str().unwrap();

    // Check tenant limits
    let req = test::TestRequest::get()
        .uri(&format!("/{}/check-limits", tenant_id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let within_limits: bool = test::read_body_json(resp).await;
    assert!(within_limits);
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};
    use user_management::services::user_service::UserService;
    use user_management::handlers::user_handlers::{register_user, login_user};
    use user_management::models::user::{CreateUserRequest, LoginRequest};

    #[actix_rt::test]
    async fn test_register_user() {
        let user_service = UserService::new(/* mock dependencies */);
        let app = test::init_service(App::new().data(user_service.clone()).service(register_user)).await;

        let req = test::TestRequest::post()
            .uri("/api/v1/users/register")
            .set_json(&CreateUserRequest {
                tenant_id: Uuid::new_v4(),
                email: "test@example.com".to_string(),
                password: "StrongPassword123!".to_string(),
                first_name: Some("Test".to_string()),
                last_name: Some("User".to_string()),
                mfa_enabled: false,
            })
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);
    }

    #[actix_rt::test]
    async fn test_login_user() {
        let user_service = UserService::new(/* mock dependencies */);
        let app = test::init_service(App::new().data(user_service.clone()).service(login_user)).await;

        let req = test::TestRequest::post()
            .uri("/api/v1/auth/login")
            .set_json(&LoginRequest {
                email: "test@example.com".to_string(),
                password: "StrongPassword123!".to_string(),
                mfa_code: None,
            })
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
    }

    // Add more tests as needed
} 