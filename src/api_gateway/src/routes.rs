use actix_web::{web, HttpResponse};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .route("/health", web::get().to(health_check))
            .route("/attestations", web::post().to(create_attestation))
            .route("/attestations/{id}", web::get().to(get_attestation))
            .route(
                "/attestations/{id}/verify",
                web::post().to(verify_attestation),
            )
            .route("/attestations", web::get().to(list_attestations)),
    );
}

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "api_gateway",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

async fn create_attestation(
    state: web::Data<crate::state::AppState>,
    payload: web::Json<serde_json::Value>,
) -> HttpResponse {
    let client = &state.http_client;
    let url = format!("{}/attestations", state.config.services.attestation_url);

    match client.post(&url).json(&payload).send().await {
        Ok(response) => {
            let status = response.status();
            match response.json::<serde_json::Value>().await {
                Ok(body) => HttpResponse::build(status).json(body),
                Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Failed to parse attestation service response"
                })),
            }
        }
        Err(_) => HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "error": "Attestation service unavailable"
        })),
    }
}

async fn get_attestation(
    state: web::Data<crate::state::AppState>,
    id: web::Path<String>,
) -> HttpResponse {
    let client = &state.http_client;
    let url = format!(
        "{}/attestations/{}",
        state.config.services.attestation_url, id
    );

    match client.get(&url).send().await {
        Ok(response) => {
            let status = response.status();
            match response.json::<serde_json::Value>().await {
                Ok(body) => HttpResponse::build(status).json(body),
                Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Failed to parse attestation service response"
                })),
            }
        }
        Err(_) => HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "error": "Attestation service unavailable"
        })),
    }
}

async fn verify_attestation(
    state: web::Data<crate::state::AppState>,
    id: web::Path<String>,
) -> HttpResponse {
    let client = &state.http_client;
    let url = format!(
        "{}/attestations/{}/verify",
        state.config.services.attestation_url, id
    );

    match client.post(&url).send().await {
        Ok(response) => {
            let status = response.status();
            match response.json::<serde_json::Value>().await {
                Ok(body) => HttpResponse::build(status).json(body),
                Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Failed to parse attestation service response"
                })),
            }
        }
        Err(_) => HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "error": "Attestation service unavailable"
        })),
    }
}

async fn list_attestations(
    state: web::Data<crate::state::AppState>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> HttpResponse {
    let client = &state.http_client;
    let url = format!("{}/attestations", state.config.services.attestation_url);

    let mut request = client.get(&url);
    for (key, value) in query.iter() {
        request = request.query(&[(key, value)]);
    }

    match request.send().await {
        Ok(response) => {
            let status = response.status();
            match response.json::<serde_json::Value>().await {
                Ok(body) => HttpResponse::build(status).json(body),
                Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Failed to parse attestation service response"
                })),
            }
        }
        Err(_) => HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "error": "Attestation service unavailable"
        })),
    }
}
