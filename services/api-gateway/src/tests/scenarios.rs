use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

use super::utils::{
    create_test_worker_pool, generate_bot_creation_payload, generate_bot_execution_payload,
    LoadTestConfig, TestRequest, run_load_test,
};

#[tokio::test]
async fn test_high_concurrency_single_endpoint() {
    let concurrency_levels = vec![50, 100, 200];
    let test_duration = Duration::from_secs(300); // 5 minutes
    let ramp_up = Duration::from_secs(60); // 1 minute ramp-up
    let ramp_down = Duration::from_secs(30); // 30 seconds ramp-down

    for concurrency in concurrency_levels {
        let worker_pool = create_test_worker_pool(concurrency * 2, concurrency / 10).await;

        let config = LoadTestConfig {
            concurrency,
            duration: test_duration,
            ramp_up,
            ramp_down,
            request_generator: Box::new(|| {
                TestRequest::new(
                    "/api/bots",
                    "POST",
                    generate_bot_creation_payload(),
                )
            }),
        };

        println!("Running high concurrency test with {} concurrent users", concurrency);
        let results = run_load_test(worker_pool, config).await;

        // Assert test results
        assert!(
            results.successful_requests as f64 / results.total_requests as f64 >= 0.95,
            "Success rate should be at least 95%"
        );
        assert!(
            results.p95_response_time <= 1.0,
            "95th percentile response time should be under 1 second"
        );

        // Print test results
        println!("Test Results for {} concurrent users:", concurrency);
        println!("Total Requests: {}", results.total_requests);
        println!("Successful Requests: {}", results.successful_requests);
        println!("Failed Requests: {}", results.failed_requests);
        println!("Average Response Time: {:.3}s", results.avg_response_time);
        println!("P95 Response Time: {:.3}s", results.p95_response_time);
        println!("P99 Response Time: {:.3}s", results.p99_response_time);
        println!("Error count: {}", results.errors.len());
        if !results.errors.is_empty() {
            println!("Sample errors:");
            for error in results.errors.iter().take(5) {
                println!("  - {}", error);
            }
        }
        println!();

        // Allow system to stabilize between tests
        sleep(Duration::from_secs(30)).await;
    }
}

#[tokio::test]
async fn test_mixed_workload_concurrency() {
    let concurrency = 100;
    let test_duration = Duration::from_secs(600); // 10 minutes
    let ramp_up = Duration::from_secs(120); // 2 minutes ramp-up
    let ramp_down = Duration::from_secs(60); // 1 minute ramp-down

    let worker_pool = create_test_worker_pool(concurrency * 2, concurrency / 5).await;

    // Create a pool of bot IDs for the test
    let bot_ids: Vec<String> = (0..10).map(|_| Uuid::new_v4().to_string()).collect();
    let bot_ids = std::sync::Arc::new(bot_ids);

    let config = LoadTestConfig {
        concurrency,
        duration: test_duration,
        ramp_up,
        ramp_down,
        request_generator: Box::new(move || {
            // Randomly select an endpoint and method
            let rand_num = rand::random::<f64>();
            match rand_num {
                n if n < 0.3 => TestRequest::new(
                    "/api/users/me",
                    "GET",
                    serde_json::json!({}),
                ),
                n if n < 0.5 => TestRequest::new(
                    "/api/bots",
                    "POST",
                    generate_bot_creation_payload(),
                ),
                n if n < 0.7 => TestRequest::new(
                    "/api/tasks",
                    "GET",
                    serde_json::json!({}),
                ),
                _ => {
                    let bot_id = &bot_ids[rand::random::<usize>() % bot_ids.len()];
                    TestRequest::new(
                        &format!("/api/bots/{}/execute", bot_id),
                        "POST",
                        generate_bot_execution_payload(bot_id),
                    )
                }
            }
        }),
    };

    println!("Running mixed workload test with {} concurrent users", concurrency);
    let results = run_load_test(worker_pool, config).await;

    // Assert test results
    assert!(
        results.successful_requests as f64 / results.total_requests as f64 >= 0.95,
        "Success rate should be at least 95%"
    );
    assert!(
        results.p95_response_time <= 2.0,
        "95th percentile response time should be under 2 seconds"
    );

    // Print test results
    println!("Mixed Workload Test Results:");
    println!("Total Requests: {}", results.total_requests);
    println!("Successful Requests: {}", results.successful_requests);
    println!("Failed Requests: {}", results.failed_requests);
    println!("Average Response Time: {:.3}s", results.avg_response_time);
    println!("P95 Response Time: {:.3}s", results.p95_response_time);
    println!("P99 Response Time: {:.3}s", results.p99_response_time);
    println!("Error count: {}", results.errors.len());
    if !results.errors.is_empty() {
        println!("Sample errors:");
        for error in results.errors.iter().take(5) {
            println!("  - {}", error);
        }
    }
}

#[tokio::test]
async fn test_stress_beyond_capacity() {
    let base_concurrency = 200;
    let test_duration = Duration::from_secs(180); // 3 minutes
    let ramp_up = Duration::from_secs(30); // 30 seconds ramp-up
    let ramp_down = Duration::from_secs(15); // 15 seconds ramp-down

    let concurrency_multipliers = vec![1.0, 1.5, 2.0, 3.0, 4.0];
    let mut breaking_point = None;

    for multiplier in concurrency_multipliers {
        let concurrency = (base_concurrency as f64 * multiplier) as usize;
        let worker_pool = create_test_worker_pool(concurrency * 2, concurrency / 5).await;

        let config = LoadTestConfig {
            concurrency,
            duration: test_duration,
            ramp_up,
            ramp_down,
            request_generator: Box::new(|| {
                let bot_id = Uuid::new_v4().to_string();
                TestRequest::new(
                    &format!("/api/bots/{}/execute", bot_id),
                    "POST",
                    generate_bot_execution_payload(&bot_id),
                )
            }),
        };

        println!("Running stress test with {} concurrent users", concurrency);
        let results = run_load_test(worker_pool, config).await;

        // Print test results
        println!("Stress Test Results for {} concurrent users:", concurrency);
        println!("Total Requests: {}", results.total_requests);
        println!("Successful Requests: {}", results.successful_requests);
        println!("Failed Requests: {}", results.failed_requests);
        println!("Average Response Time: {:.3}s", results.avg_response_time);
        println!("P95 Response Time: {:.3}s", results.p95_response_time);
        println!("P99 Response Time: {:.3}s", results.p99_response_time);
        println!("Error count: {}", results.errors.len());
        
        // Check if we've hit the breaking point
        let success_rate = results.successful_requests as f64 / results.total_requests as f64;
        if success_rate < 0.90 || results.p95_response_time > 5.0 {
            breaking_point = Some(concurrency);
            println!("Breaking point detected at {} concurrent users", concurrency);
            println!("Success rate: {:.2}%", success_rate * 100.0);
            println!("P95 response time: {:.3}s", results.p95_response_time);
            break;
        }

        // Allow system to stabilize between tests
        sleep(Duration::from_secs(30)).await;
    }

    assert!(breaking_point.is_some(), "Should find a breaking point");
    println!("System breaking point: {} concurrent users", breaking_point.unwrap());
}

#[tokio::test]
async fn test_worker_pool_adaptive_scaling() {
    let initial_concurrency = 50;
    let test_duration = Duration::from_secs(600); // 10 minutes
    let worker_pool = create_test_worker_pool(initial_concurrency * 4, initial_concurrency / 5).await;

    // Start with low load
    let low_load_config = LoadTestConfig {
        concurrency: initial_concurrency,
        duration: Duration::from_secs(120),
        ramp_up: Duration::from_secs(30),
        ramp_down: Duration::from_secs(0),
        request_generator: Box::new(|| {
            TestRequest::new(
                "/api/bots",
                "POST",
                generate_bot_creation_payload(),
            )
        }),
    };

    println!("Running adaptive scaling test - Phase 1: Low Load");
    let low_load_results = run_load_test(worker_pool.clone(), low_load_config).await;
    println!("Low Load Results:");
    println!("Average Response Time: {:.3}s", low_load_results.avg_response_time);
    println!("P95 Response Time: {:.3}s", low_load_results.p95_response_time);

    // Suddenly increase load
    let high_load_config = LoadTestConfig {
        concurrency: initial_concurrency * 3,
        duration: Duration::from_secs(120),
        ramp_up: Duration::from_secs(10), // Quick ramp-up
        ramp_down: Duration::from_secs(0),
        request_generator: Box::new(|| {
            TestRequest::new(
                "/api/bots",
                "POST",
                generate_bot_creation_payload(),
            )
        }),
    };

    println!("Running adaptive scaling test - Phase 2: High Load");
    let high_load_results = run_load_test(worker_pool.clone(), high_load_config).await;
    println!("High Load Results:");
    println!("Average Response Time: {:.3}s", high_load_results.avg_response_time);
    println!("P95 Response Time: {:.3}s", high_load_results.p95_response_time);

    // Verify adaptive scaling behavior
    assert!(
        high_load_results.avg_response_time <= low_load_results.avg_response_time * 2.0,
        "Response time should not increase linearly with load due to adaptive scaling"
    );
} 