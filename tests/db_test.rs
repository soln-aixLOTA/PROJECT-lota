use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;

#[tokio::test]
async fn test_database_connection() {
    dotenv().ok();
    
    let database_url = format!(
        "postgresql://{}:{}@{}:{}/{}",
        std::env::var("DB_USER").unwrap_or_else(|_| "postgre".to_string()),
        std::env::var("DB_PASSWORD").unwrap_or_else(|_| "Lhl980107".to_string()),
        std::env::var("DB_HOST").unwrap_or_else(|_| "10.87.224.2".to_string()),
        std::env::var("DB_PORT").unwrap_or_else(|_| "5432".to_string()),
        std::env::var("DB_NAME").unwrap_or_else(|_| "postgres".to_string()),
    );

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await;

    match pool {
        Ok(pool) => {
            // Try a simple query
            let result = sqlx::query!("SELECT 1 as one")
                .fetch_one(&pool)
                .await;
            
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