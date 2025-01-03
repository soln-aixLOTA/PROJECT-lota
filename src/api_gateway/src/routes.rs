use actix_web::{get, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::{error, info};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    status: String,
    version: String,
    uptime: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MetricsResponse {
    requests_total: u64,
    requests_failed: u64,
    average_response_time: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InferenceRequest {
    model_id: String,
    inputs: Vec<String>,
    parameters: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InferenceResponse {
    request_id: Uuid,
    outputs: Vec<String>,
    metadata: Option<serde_json::Value>,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .service(health_check)
            .service(get_metrics)
            .service(inference)
            .service(
                web::scope("/models")
                    .route("", web::get().to(list_models))
                    .route("/{model_id}", web::get().to(get_model))
                    .route("/{model_id}/versions", web::get().to(list_model_versions)),
            )
            .service(
                web::scope("/tenants")
                    .route("", web::get().to(list_tenants))
                    .route("/{tenant_id}", web::get().to(get_tenant))
                    .route("/{tenant_id}/usage", web::get().to(get_tenant_usage)),
            ),
    );
}

#[get("/health")]
async fn health_check() -> impl Responder {
    let health_info = HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };

    HttpResponse::Ok().json(health_info)
}

#[get("/metrics")]
async fn get_metrics() -> impl Responder {
    let metrics = MetricsResponse {
        requests_total: prometheus::default_registry()
            .gather()
            .iter()
            .find(|m| m.get_name() == "http_requests_total")
            .map(|m| m.get_metric()[0].get_counter().get_value() as u64)
            .unwrap_or(0),
        requests_failed: prometheus::default_registry()
            .gather()
            .iter()
            .find(|m| m.get_name() == "http_requests_total")
            .map(|m| {
                m.get_metric()
                    .iter()
                    .filter(|metric| {
                        metric
                            .get_label()
                            .iter()
                            .any(|l| l.get_name() == "status" && l.get_value().starts_with('5'))
                    })
                    .map(|metric| metric.get_counter().get_value() as u64)
                    .sum()
            })
            .unwrap_or(0),
        average_response_time: prometheus::default_registry()
            .gather()
            .iter()
            .find(|m| m.get_name() == "http_request_duration_seconds")
            .map(|m| m.get_metric()[0].get_histogram().get_sample_sum())
            .unwrap_or(0.0),
    };

    HttpResponse::Ok().json(metrics)
}

#[post("/inference")]
async fn inference(
    pool: web::Data<PgPool>,
    req: web::Json<InferenceRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    info!(
        model_id = %req.model_id,
        inputs_count = req.inputs.len(),
        "Processing inference request"
    );

    // Here you would typically:
    // 1. Validate the model exists and is available
    // 2. Check if the user has permission to use the model
    // 3. Send the request to the inference service
    // 4. Track usage and billing
    // 5. Return the response

    let response = InferenceResponse {
        request_id: Uuid::new_v4(),
        outputs: vec!["Sample output".to_string()], // Replace with actual inference results
        metadata: Some(serde_json::json!({
            "model_version": "1.0",
            "processing_time": 0.1,
        })),
    };

    Ok(HttpResponse::Ok().json(response))
}

async fn list_models(pool: web::Data<PgPool>) -> Result<HttpResponse, actix_web::Error> {
    // Implement model listing logic
    Ok(HttpResponse::Ok().json(vec![]))
}

async fn get_model(
    pool: web::Data<PgPool>,
    model_id: web::Path<String>,
) -> Result<HttpResponse, actix_web::Error> {
    // Implement model retrieval logic
    Ok(HttpResponse::Ok().json({}))
}

async fn list_model_versions(
    pool: web::Data<PgPool>,
    model_id: web::Path<String>,
) -> Result<HttpResponse, actix_web::Error> {
    // Implement model versions listing logic
    Ok(HttpResponse::Ok().json(vec![]))
}

async fn list_tenants(pool: web::Data<PgPool>) -> Result<HttpResponse, actix_web::Error> {
    // Implement tenant listing logic
    Ok(HttpResponse::Ok().json(vec![]))
}

async fn get_tenant(
    pool: web::Data<PgPool>,
    tenant_id: web::Path<Uuid>,
) -> Result<HttpResponse, actix_web::Error> {
    // Implement tenant retrieval logic
    Ok(HttpResponse::Ok().json({}))
}

async fn get_tenant_usage(
    pool: web::Data<PgPool>,
    tenant_id: web::Path<Uuid>,
) -> Result<HttpResponse, actix_web::Error> {
    // Implement tenant usage retrieval logic
    Ok(HttpResponse::Ok().json({}))
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test;
    use sqlx::postgres::PgPoolOptions;

    #[actix_web::test]
    async fn test_health_check() {
        let app = test::init_service(actix_web::App::new().service(health_check)).await;

        let req = test::TestRequest::get().uri("/health").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_metrics() {
        let app = test::init_service(actix_web::App::new().service(get_metrics)).await;

        let req = test::TestRequest::get().uri("/metrics").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_inference() {
        let db_pool = PgPoolOptions::new()
            .max_connections(5)
            .connect("postgres://localhost/test_db")
            .await
            .expect("Failed to connect to database");

        let app = test::init_service(
            actix_web::App::new()
                .app_data(web::Data::new(db_pool.clone()))
                .service(inference),
        )
        .await;

        let payload = InferenceRequest {
            model_id: "test-model".to_string(),
            inputs: vec!["test input".to_string()],
            parameters: None,
        };

        let req = test::TestRequest::post()
            .uri("/inference")
            .set_json(&payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
}
