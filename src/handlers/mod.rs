use actix_web::web;

pub mod auth;
pub mod documents;
pub mod health;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.configure(auth::config)
        .configure(documents::config)
        .configure(health::config);
}
