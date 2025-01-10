use crate::{
    api::{self, routes},
    config::Config,
    storage::{PostgresStorage, S3Storage},
    tests::test_utils::{
        error_scenarios, ErrorConfig, ErrorInjectionLayer, TestCleanup, TestContext,
    },
    AttestationResult, HardwareInfo,
};
use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use serde_json::json;
use sqlx::PgPool;
use std::{net::SocketAddr, time::Duration};
use tower::{ServiceBuilder, ServiceExt};
use uuid::Uuid;

async fn setup_test_app() -> (Router, TestContext) {
    // Load test configuration
    let config = Config::test_config();

    // Setup test database connection
    let db_pool = PgPool::connect(&config.database_url)
        .await
        .expect("Failed to connect to test database");

    // Setup test S3 client
    let s3_client = aws_sdk_s3::Client::new(
        &aws_sdk_s3::Config::builder()
            .endpoint_url("http://localhost:4566")
            .region(aws_sdk_s3::Region::new("us-east-1"))
            .build(),
    );

    // Initialize storage implementations
    let postgres_storage = PostgresStorage::new(db_pool.clone());
    let s3_storage = S3Storage::new(s3_client.clone(), &config.s3_bucket);

    // Create test cleanup
    let cleanup = TestCleanup::new(db_pool, s3_client, config.s3_bucket.clone());
    let test_context = TestContext::new(cleanup);

    // Build the router with test configuration
    let app = Router::new()
        .merge(api::router(config, postgres_storage, s3_storage))
        .layer(ServiceBuilder::new().layer(ErrorInjectionLayer::new(ErrorConfig::default())));

    (app, test_context)
}

#[tokio::test]
async fn test_health_endpoint() {
    let (app, _context) = setup_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert!(
        _context.check_resource_usage().await,
        "Resource usage exceeded thresholds"
    );
}

#[tokio::test]
async fn test_attestation_endpoint_success() {
    let (app, context) = setup_test_app().await;

    let request_body = json!({
        "request_id": Uuid::new_v4().to_string(),
        "hardware_info": {
            "gpu_count": 2,
            "driver_version": "535.129.03"
        }
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/attestation")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&request_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    assert!(response_json["id"].is_string());
    assert!(response_json["timestamp"].is_string());
    assert_eq!(response_json["hardware_info"]["gpu_count"], 2);

    // Verify resource usage
    assert!(
        context.check_resource_usage().await,
        "Resource usage exceeded thresholds"
    );
}

#[tokio::test]
async fn test_attestation_endpoint_with_errors() {
    let (app, context) = setup_test_app().await;

    // Test with network latency
    let app = app.layer(ErrorInjectionLayer::new(error_scenarios::network_latency(
        Duration::from_millis(100),
    )));

    let request_body = json!({
        "request_id": Uuid::new_v4().to_string(),
        "hardware_info": {
            "gpu_count": 2,
            "driver_version": "535.129.03"
        }
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/attestation")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&request_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Even with latency, request should succeed
    assert_eq!(response.status(), StatusCode::OK);

    // Test with connection reset
    let app = app.layer(ErrorInjectionLayer::new(error_scenarios::connection_reset()));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/attestation")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&request_body).unwrap()))
                .unwrap(),
        )
        .await;

    // Connection reset should cause an error
    assert!(response.is_err());

    // Verify resource usage after error tests
    assert!(
        context.check_resource_usage().await,
        "Resource usage exceeded thresholds during error testing"
    );
}

#[tokio::test]
async fn test_concurrent_attestation_requests() {
    let (app, context) = setup_test_app().await;
    let app = std::sync::Arc::new(app);

    let mut handles = vec![];
    let request_count = 10;

    for _ in 0..request_count {
        let app = app.clone();
        let handle = tokio::spawn(async move {
            let request_body = json!({
                "request_id": Uuid::new_v4().to_string(),
                "hardware_info": {
                    "gpu_count": 2,
                    "driver_version": "535.129.03"
                }
            });

            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri("/attestation")
                        .method("POST")
                        .header("Content-Type", "application/json")
                        .body(Body::from(serde_json::to_vec(&request_body).unwrap()))
                        .unwrap(),
                )
                .await
                .unwrap();

            assert_eq!(response.status(), StatusCode::OK);
        });

        handles.push(handle);
    }

    // Wait for all requests to complete
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify resource usage during concurrent requests
    assert!(
        context.check_resource_usage().await,
        "Resource usage exceeded thresholds during concurrent requests"
    );

    // Get and log metrics
    let metrics = context.get_metrics().await;
    tracing::info!("Resource metrics during concurrent requests: {:?}", metrics);
}

#[tokio::test]
async fn test_attestation_endpoint_invalid_request() {
    let (app, context) = setup_test_app().await;

    let invalid_body = json!({
        "invalid_field": "invalid_value"
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/attestation")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&invalid_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Verify resource usage
    assert!(
        context.check_resource_usage().await,
        "Resource usage exceeded thresholds during invalid request"
    );
}
