use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;

#[tokio::test]
async fn test_database_connection() {
    dotenv().ok();

    let database_url = format!(
        "postgresql://{}:{}@{}:{}/{}",
        std::env::var("DB_USER").expect("DB_USER must be set"),
        std::env::var("DB_PASSWORD").expect("DB_PASSWORD must be set"),
        "localhost",
        std::env::var("DB_PORT").expect("DB_PORT must be set"),
        std::env::var("DB_NAME").expect("DB_NAME must be set"),
    );

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to the database");

    match pool {
        Ok(pool) => {
            // Try a simple query
            let result = sqlx::query!("SELECT 1 as one").fetch_one(&pool).await;

            match result {
                Ok(row) => {
                    assert_eq!(row.one, Some(1));
                    println!("Database connection and query successful!");
                }
                Err(e) => panic!("Query failed: {}", e),
            }
        }
        Err(e) => panic!("Failed to connect to database: {}", e),
    }
}
