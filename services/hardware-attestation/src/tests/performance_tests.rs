use axum::{body::Body, http::Request, Router};
use criterion::{criterion_group, criterion_main, Criterion};
use serde_json::json;
use std::{sync::Arc, time::Duration};
use tokio::runtime::Runtime;
use tower::ServiceExt;
use uuid::Uuid;

use super::integration_tests::setup_test_app;

fn benchmark_attestation_request(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let app = rt.block_on(setup_test_app());
    let app = Arc::new(app);

    c.bench_function("single_attestation_request", |b| {
        b.iter(|| {
            let app = app.clone();
            rt.block_on(async {
                let request_body = json!({
                    "request_id": Uuid::new_v4().to_string(),
                    "hardware_info": {
                        "gpu_count": 2,
                        "driver_version": "535.129.03"
                    }
                });

                app.clone()
                    .oneshot(
                        Request::builder()
                            .uri("/attestation")
                            .method("POST")
                            .header("Content-Type", "application/json")
                            .body(Body::from(serde_json::to_vec(&request_body).unwrap()))
                            .unwrap(),
                    )
                    .await
                    .unwrap()
            })
        })
    });
}

fn benchmark_concurrent_attestation_requests(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let app = rt.block_on(setup_test_app());
    let app = Arc::new(app);

    let mut group = c.benchmark_group("concurrent_attestation_requests");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(20));

    for concurrent_requests in [10, 50, 100].iter() {
        group.bench_function(format!("concurrent_{}", concurrent_requests), |b| {
            b.iter(|| {
                let app = app.clone();
                rt.block_on(async {
                    let mut handles = vec![];

                    for _ in 0..*concurrent_requests {
                        let app = app.clone();
                        let handle = tokio::spawn(async move {
                            let request_body = json!({
                                "request_id": Uuid::new_v4().to_string(),
                                "hardware_info": {
                                    "gpu_count": 2,
                                    "driver_version": "535.129.03"
                                }
                            });

                            app.clone()
                                .oneshot(
                                    Request::builder()
                                        .uri("/attestation")
                                        .method("POST")
                                        .header("Content-Type", "application/json")
                                        .body(Body::from(
                                            serde_json::to_vec(&request_body).unwrap(),
                                        ))
                                        .unwrap(),
                                )
                                .await
                                .unwrap()
                        });

                        handles.push(handle);
                    }

                    for handle in handles {
                        handle.await.unwrap();
                    }
                })
            });
        });
    }

    group.finish();
}

fn benchmark_storage_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let app = rt.block_on(setup_test_app());
    let app = Arc::new(app);

    let mut group = c.benchmark_group("storage_operations");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(20));

    // Benchmark database operations
    group.bench_function("database_write", |b| {
        b.iter(|| {
            let app = app.clone();
            rt.block_on(async {
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

                let body_bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
                let response_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
                response_json["id"].as_str().unwrap().to_string()
            })
        })
    });

    group.finish();
}

fn benchmark_hardware_verification(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let app = rt.block_on(setup_test_app());
    let app = Arc::new(app);

    c.bench_function("hardware_verification", |b| {
        b.iter(|| {
            let app = app.clone();
            rt.block_on(async {
                let request_body = json!({
                    "request_id": Uuid::new_v4().to_string(),
                    "hardware_info": {
                        "gpu_count": 2,
                        "driver_version": "535.129.03"
                    }
                });

                app.clone()
                    .oneshot(
                        Request::builder()
                            .uri("/attestation/verify")
                            .method("POST")
                            .header("Content-Type", "application/json")
                            .body(Body::from(serde_json::to_vec(&request_body).unwrap()))
                            .unwrap(),
                    )
                    .await
                    .unwrap()
            })
        })
    });
}

criterion_group!(
    benches,
    benchmark_attestation_request,
    benchmark_concurrent_attestation_requests,
    benchmark_storage_operations,
    benchmark_hardware_verification
);
criterion_main!(benches);
