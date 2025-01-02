pub mod auth;
pub mod inference;
pub mod models;
pub mod users;

use actix_web::web;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .configure(auth::routes)
            .configure(users::routes)
            .configure(models::routes)
            .configure(inference::routes),
    );
}
