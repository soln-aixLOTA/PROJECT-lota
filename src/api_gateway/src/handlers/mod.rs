use actix_web::web;

mod health;
mod inference;
mod models;
mod users;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .configure(health::configure_routes)
            .configure(models::configure_routes)
            .configure(inference::configure_routes)
            .configure(users::configure_routes),
    );
}
