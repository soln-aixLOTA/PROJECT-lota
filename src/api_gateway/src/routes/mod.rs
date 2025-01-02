pub mod health;
pub mod metrics;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(health::health_check)
        .service(web::resource("/metrics").to(crate::metrics::metrics_handler));
}
