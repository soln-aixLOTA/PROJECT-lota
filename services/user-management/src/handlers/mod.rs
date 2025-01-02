pub mod auth;
pub mod tenants;
pub mod users;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .configure(users::configure)
            .configure(tenants::configure)
            .configure(auth::configure),
    );
}
