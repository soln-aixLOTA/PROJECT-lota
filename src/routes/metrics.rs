use actix_web::{get, HttpResponse, Responder};
use prometheus::{Encoder, TextEncoder};

#[get("/metrics")]
pub async fn get_metrics() -> impl Responder {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();

    encoder
        .encode(&metric_families, &mut buffer)
        .unwrap_or_default();

    HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4")
        .body(String::from_utf8(buffer).unwrap_or_default())
}
