use std::sync::Arc;

use actix_web::{
    http::StatusCode,
    test,
    web::{self, Data},
    App, HttpResponse,
};
use sqlx::PgPool;
use uuid::Uuid;
use lotabots::middleware::auth::Claims;

use crate::{
    middleware::{
        audit_middleware::AuditMiddleware,
        metrics_middleware::MetricsMiddleware,
        rate_limit_middleware::RateLimitMiddleware,
        tenant_middleware::TenantMiddleware,
    },
    models::tenant::{CreateTenantRequest, SubscriptionTier},
    repositories::tenant_repository::PostgresTenantRepository,
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

async fn test_handler() -> HttpResponse {
    HttpResponse::Ok().finish()
}

#[actix_web::test]
async fn test_tenant_middleware() {
    let pool = setup_test_db().await;
    let tenant_repository = Arc::new(PostgresTenantRepository::new(pool.clone()));
    let tenant_service = Arc::new(TenantService::new(tenant_repository));
    let tenant_id = create_test_tenant(&tenant_service).await;

    let app = test::init_service(
        App::new()
            .app_data(Data::new(tenant_service.clone()))
            .wrap(TenantMiddleware::new(tenant_service))
            .route("/test", web::get().to(test_handler)),
    )
    .await;

    // Test with valid tenant ID
    let req = test::TestRequest::get()
        .uri("/test")
        .insert_header(("X-Tenant-ID", tenant_id.to_string()))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Test with invalid tenant ID
    let req = test::TestRequest::get()
        .uri("/test")
        .insert_header(("X-Tenant-ID", Uuid::new_v4().to_string()))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    // Test without tenant ID
    let req = test::TestRequest::get().uri("/test").to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[actix_web::test]
async fn test_rate_limit_middleware() {
    let pool = setup_test_db().await;
    let tenant_repository = Arc::new(PostgresTenantRepository::new(pool.clone()));
    let tenant_service = Arc::new(TenantService::new(tenant_repository));
    let tenant_id = create_test_tenant(&tenant_service).await;

    let app = test::init_service(
        App::new()
            .app_data(Data::new(tenant_service.clone()))
            .wrap(RateLimitMiddleware::new(tenant_service.clone()))
            .wrap(TenantMiddleware::new(tenant_service))
            .route("/test", web::get().to(test_handler)),
    )
    .await;

    // Test within rate limit
    for _ in 0..10 {
        let req = test::TestRequest::get()
            .uri("/test")
            .insert_header(("X-Tenant-ID", tenant_id.to_string()))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        assert!(resp.headers().contains_key("X-RateLimit-Remaining"));
        assert!(resp.headers().contains_key("X-RateLimit-Reset"));
    }

    // Test exceeding rate limit (simulate by making many requests quickly)
    for _ in 0..1000 {
        let req = test::TestRequest::get()
            .uri("/test")
            .insert_header(("X-Tenant-ID", tenant_id.to_string()))
            .to_request();

        let resp = test::call_service(&app, req).await;
        if resp.status() == StatusCode::TOO_MANY_REQUESTS {
            return; // Test passed when rate limit is hit
        }
    }

    panic!("Rate limit was not enforced");
}

#[actix_web::test]
async fn test_metrics_middleware() {
    let pool = setup_test_db().await;
    let tenant_repository = Arc::new(PostgresTenantRepository::new(pool.clone()));
    let tenant_service = Arc::new(TenantService::new(tenant_repository));
    let tenant_id = create_test_tenant(&tenant_service).await;

    let app = test::init_service(
        App::new()
            .app_data(Data::new(tenant_service.clone()))
            .wrap(MetricsMiddleware::new(tenant_service.clone()))
            .wrap(TenantMiddleware::new(tenant_service))
            .route("/test", web::get().to(test_handler)),
    )
    .await;

    // Make a request
    let req = test::TestRequest::get()
        .uri("/test")
        .insert_header(("X-Tenant-ID", tenant_id.to_string()))
        .insert_header(("content-length", "100"))
        .insert_header(("X-GPU-Time-Ms", "500"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Metrics should be recorded (would need a metrics endpoint to verify)
}

#[actix_web::test]
async fn test_audit_middleware() {
    let pool = setup_test_db().await;
    let tenant_repository = Arc::new(PostgresTenantRepository::new(pool.clone()));
    let tenant_service = Arc::new(TenantService::new(tenant_repository));
    let tenant_id = create_test_tenant(&tenant_service).await;

    let app = test::init_service(
        App::new()
            .app_data(Data::new(tenant_service.clone()))
            .wrap(AuditMiddleware::new(tenant_service.clone(), pool.clone()))
            .wrap(TenantMiddleware::new(tenant_service))
            .route("/test", web::get().to(test_handler)),
    )
    .await;

    // Make a request
    let req = test::TestRequest::get()
        .uri("/test")
        .insert_header(("X-Tenant-ID", tenant_id.to_string()))
        .insert_header(("User-Agent", "test-agent"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Verify audit logs were created
    let logs = sqlx::query!(
        r#"
        SELECT COUNT(*) as count
        FROM audit_logs
        WHERE tenant_id = $1
        "#,
        tenant_id
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch audit logs");

    assert_eq!(logs.count.unwrap(), 2); // One log for request, one for response
}

#[actix_web::test]
async fn test_middleware_chain() {
    let pool = setup_test_db().await;
    let tenant_repository = Arc::new(PostgresTenantRepository::new(pool.clone()));
    let tenant_service = Arc::new(TenantService::new(tenant_repository));
    let tenant_id = create_test_tenant(&tenant_service).await;

    let app = test::init_service(
        App::new()
            .app_data(Data::new(tenant_service.clone()))
            .wrap(AuditMiddleware::new(tenant_service.clone(), pool.clone()))
            .wrap(MetricsMiddleware::new(tenant_service.clone()))
            .wrap(RateLimitMiddleware::new(tenant_service.clone()))
            .wrap(TenantMiddleware::new(tenant_service))
            .route("/test", web::get().to(test_handler)),
    )
    .await;

    // Make a request that should pass through all middleware
    let req = test::TestRequest::get()
        .uri("/test")
        .insert_header(("X-Tenant-ID", tenant_id.to_string()))
        .insert_header(("User-Agent", "test-agent"))
        .insert_header(("content-length", "100"))
        .insert_header(("X-GPU-Time-Ms", "500"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Verify rate limit headers
    assert!(resp.headers().contains_key("X-RateLimit-Remaining"));
    assert!(resp.headers().contains_key("X-RateLimit-Reset"));

    // Verify audit logs
    let logs = sqlx::query!(
        r#"
        SELECT COUNT(*) as count
        FROM audit_logs
        WHERE tenant_id = $1
        "#,
        tenant_id
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch audit logs");

    assert_eq!(logs.count.unwrap(), 2);
} 