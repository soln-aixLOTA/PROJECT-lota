use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;

#[tokio::test]
async fn test_db_operations() {
    dotenv().ok();

    let database_url = format!(
        "postgresql://{}:{}@{}:{}/{}",
        std::env::var("DB_USER").expect("DB_USER must be set"),
        std::env::var("DB_PASSWORD").expect("DB_PASSWORD must be set"),
        "localhost",
        std::env::var("DB_PORT").expect("DB_PORT must be set"),
        std::env::var("DB_NAME").expect("DB_NAME must be set"),
    );

    println!("Attempting to connect to database...");
    let pool = match PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
    {
        Ok(pool) => pool,
        Err(e) => panic!("Failed to connect to database: {}", e),
    };

    // Test database operations
    let result = sqlx::query!("SELECT 1 as one").fetch_one(&pool).await;

    match result {
        Ok(_) => println!("Successfully executed test query!"),
        Err(e) => panic!("Query failed: {}", e),
    }
}
