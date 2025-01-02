use actix_web::{get, web, HttpResponse, Responder};
use serde_json::json;

#[derive(Debug)]
pub struct HealthState {
    pub attestation_client: web::Data<reqwest::Client>,
}

#[get("/health")]
pub async fn health_check(state: web::Data<HealthState>) -> impl Responder {
    // Check attestation service health
    let attestation_health = match state
        .attestation_client
        .get("http://attestation:8080/health")
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
    {
        Ok(response) if response.status().is_success() => true,
        _ => false,
    };

    let status = if attestation_health {
        "healthy"
    } else {
        "degraded"
    };

    HttpResponse::Ok().json(json!({
        "status": status,
        "service": "api_gateway",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "dependencies": {
            "attestation": attestation_health
        }
    }))
}
