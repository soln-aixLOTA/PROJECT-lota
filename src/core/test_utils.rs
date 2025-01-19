use sqlx::PgPool;
use std::sync::Once;
use tokio::runtime::Runtime;

static INIT: Once = Once::new();

pub async fn setup_test_db() -> PgPool {
    INIT.call_once(|| {
        dotenvy::dotenv().ok();
    });

    let database_url =
        std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set for tests");

    // Create connection pool
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to create database pool");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}
