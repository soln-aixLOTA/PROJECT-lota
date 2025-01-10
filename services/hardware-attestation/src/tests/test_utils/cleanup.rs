use aws_sdk_s3::Client as S3Client;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info};

/// Manages test data cleanup
pub struct TestCleanup {
    db_pool: Arc<PgPool>,
    s3_client: Arc<S3Client>,
    bucket_name: String,
}

impl TestCleanup {
    pub fn new(db_pool: PgPool, s3_client: S3Client, bucket_name: String) -> Self {
        Self {
            db_pool: Arc::new(db_pool),
            s3_client: Arc::new(s3_client),
            bucket_name,
        }
    }

    /// Cleans up all test data from both database and S3
    pub async fn cleanup_all(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting test data cleanup");

        // Cleanup database
        self.cleanup_database().await?;

        // Cleanup S3
        self.cleanup_s3().await?;

        info!("Test data cleanup completed");
        Ok(())
    }

    /// Cleans up database test data
    async fn cleanup_database(&self) -> Result<(), sqlx::Error> {
        info!("Cleaning up database test data");

        sqlx::query!("TRUNCATE TABLE attestations CASCADE")
            .execute(&*self.db_pool)
            .await?;

        Ok(())
    }

    /// Cleans up S3 test data
    async fn cleanup_s3(&self) -> Result<(), aws_sdk_s3::Error> {
        info!("Cleaning up S3 test data");

        let objects = self
            .s3_client
            .list_objects_v2()
            .bucket(&self.bucket_name)
            .send()
            .await?;

        if let Some(contents) = objects.contents() {
            for obj in contents {
                if let Some(key) = obj.key() {
                    info!("Deleting S3 object: {}", key);
                    self.s3_client
                        .delete_object()
                        .bucket(&self.bucket_name)
                        .key(key)
                        .send()
                        .await?;
                }
            }
        }

        Ok(())
    }
}

/// RAII guard for test cleanup
pub struct CleanupGuard {
    cleanup: Arc<TestCleanup>,
}

impl CleanupGuard {
    pub fn new(cleanup: TestCleanup) -> Self {
        Self {
            cleanup: Arc::new(cleanup),
        }
    }
}

impl Drop for CleanupGuard {
    fn drop(&mut self) {
        let cleanup = self.cleanup.clone();
        tokio::spawn(async move {
            if let Err(e) = cleanup.cleanup_all().await {
                error!("Failed to cleanup test data: {}", e);
            }
        });
    }
}
