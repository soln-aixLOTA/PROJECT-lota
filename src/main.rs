use actix_web::{middleware, web, App, HttpServer};
use document_automation::{
    config::AppConfig,
    handlers,
    middleware::{RequestId, SecurityHeaders},
};
use dotenv::dotenv;
use sqlx::postgres::PgPool;
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to create pool");

    let config = AppConfig::from_env().expect("Failed to load configuration");
    let bind_addr = format!("{}:{}", config.host, config.port);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(SecurityHeaders::new())
            .wrap(RequestId::new())
            .wrap(middleware::Logger::default())
            .configure(handlers::configure_routes)
    })
    .bind(&bind_addr)?
    .run()
    .await
}
