use actix_web::{get, web, HttpResponse, Responder};
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::info;

#[derive(Serialize)]
pub struct HealthResponse {
    status: String,
    version: String,
    timestamp: u64,
    uptime: u64,
    services: ServiceStatus,
}

#[derive(Serialize)]
pub struct ServiceStatus {
    database: String,
    redis: String,
    auth: String,
}

#[derive(Serialize)]
pub struct MetricsResponse {
    requests_total: u64,
    requests_per_second: f64,
    average_response_time: f64,
    memory_usage: f64,
    cpu_usage: f64,
}

static START_TIME: once_cell::sync::Lazy<SystemTime> = once_cell::sync::Lazy::new(SystemTime::now);

/// Simple health check endpoint for UptimeRobot
#[get("/health")]
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

/// Detailed health status endpoint
#[get("/health/status")]
pub async fn health_status(
    db_pool: web::Data<sqlx::PgPool>,
    redis_pool: web::Data<deadpool_redis::Pool>,
) -> impl Responder {
    let now = SystemTime::now();
    let timestamp = now.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    let uptime = now
        .duration_since(*START_TIME)
        .unwrap_or_default()
        .as_secs();

    // Check database connection
    let db_status = match sqlx::query("SELECT 1").execute(&**db_pool).await {
        Ok(_) => "healthy",
        Err(e) => {
            info!("Database health check failed: {}", e);
            "unhealthy"
        }
    };

    // Check Redis connection
    let redis_status = match redis_pool.get().await {
        Ok(_) => "healthy",
        Err(e) => {
            info!("Redis health check failed: {}", e);
            "unhealthy"
        }
    };

    let response = HealthResponse {
        status: "operational".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp,
        uptime,
        services: ServiceStatus {
            database: db_status.to_string(),
            redis: redis_status.to_string(),
            auth: "healthy".to_string(),
        },
    };

    HttpResponse::Ok().json(response)
}

/// Metrics endpoint for monitoring
#[get("/metrics")]
pub async fn metrics(metrics_data: web::Data<prometheus::Registry>) -> impl Responder {
    use prometheus::Encoder;
    let encoder = prometheus::TextEncoder::new();
    let mut buffer = Vec::new();

    if let Err(e) = encoder.encode(&metrics_data.gather(), &mut buffer) {
        info!("Error encoding metrics: {}", e);
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4")
        .body(buffer)
}

/// SSL certificate status endpoint
#[get("/health/ssl")]
pub async fn ssl_status() -> impl Responder {
    use openssl::x509::X509;
    use std::fs::File;
    use std::io::Read;

    let cert_path = std::env::var("TLS_CERT_PATH").unwrap_or_default();
    if cert_path.is_empty() {
        return HttpResponse::NotFound().json(serde_json::json!({
            "status": "not_configured",
            "message": "SSL certificate path not configured"
        }));
    }

    let mut cert_file = match File::open(&cert_path) {
        Ok(file) => file,
        Err(e) => {
            info!("Failed to open certificate file: {}", e);
            return HttpResponse::NotFound().json(serde_json::json!({
                "status": "error",
                "message": "Certificate file not found"
            }));
        }
    };

    let mut cert_data = Vec::new();
    if let Err(e) = cert_file.read_to_end(&mut cert_data) {
        info!("Failed to read certificate data: {}", e);
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "status": "error",
            "message": "Failed to read certificate"
        }));
    }

    let cert = match X509::from_pem(&cert_data) {
        Ok(cert) => cert,
        Err(e) => {
            info!("Failed to parse certificate: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": "Invalid certificate format"
            }));
        }
    };

    let not_after = cert.not_after().to_string();
    let issuer = cert
        .issuer_name()
        .entries()
        .next()
        .map(|e| e.data().as_utf8().unwrap().to_string())
        .unwrap_or_default();

    HttpResponse::Ok().json(serde_json::json!({
        "status": "valid",
        "expires": not_after,
        "issuer": issuer
    }))
}

/// Domain expiration check endpoint
#[get("/health/domain")]
pub async fn domain_status() -> impl Responder {
    use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};
    use trust_dns_resolver::Resolver;

    let domain = std::env::var("DOMAIN_NAME").unwrap_or_default();
    if domain.is_empty() {
        return HttpResponse::NotFound().json(serde_json::json!({
            "status": "not_configured",
            "message": "Domain name not configured"
        }));
    }

    let resolver = match Resolver::new(ResolverConfig::default(), ResolverOpts::default()) {
        Ok(r) => r,
        Err(e) => {
            info!("Failed to create DNS resolver: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": "DNS resolver error"
            }));
        }
    };

    match resolver.lookup_ip(&domain) {
        Ok(response) => {
            let ips: Vec<String> = response.iter().map(|ip| ip.to_string()).collect();

            HttpResponse::Ok().json(serde_json::json!({
                "status": "active",
                "domain": domain,
                "ips": ips
            }))
        }
        Err(e) => {
            info!("Domain lookup failed: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": "Domain lookup failed"
            }))
        }
    }
}
