use actix_web::{HttpResponse, Responder};
use sqlx::postgres::PgPoolOptions;
use std::env;

// Test handler for auth tests
pub async fn test_handler() -> impl Responder {
    HttpResponse::Ok().body("test")
}

// Common test utilities
pub mod common {
    use sqlx::postgres::PgPoolOptions;
    use std::env;

    pub async fn setup_test_db() -> sqlx::PgPool {
        dotenv::dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to create pool")
    }
}

pub mod auth_tests;
pub mod user_tests;
