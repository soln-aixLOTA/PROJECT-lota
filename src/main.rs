use actix_web::{App, HttpServer};
use handlers::configure_routes;
use logging::init_logger;

mod errors;
mod handlers;
mod logging;
mod middleware;
mod resource_management;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_logger();
    
    HttpServer::new(|| {
        App::new()
            .configure(configure_routes)
            .wrap(middleware::auth::Authentication)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
