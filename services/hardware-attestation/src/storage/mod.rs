use crate::{AttestationError, HardwareInfo};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::time::Duration;
use tracing::{error, info};
use uuid::Uuid;

pub struct StorageClient {
    pool: Pool<Postgres>,
}

impl StorageClient {
    pub async fn new(database_url: &str) -> Result<Self, AttestationError> {
        // Add connection timeout and retry logic
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(3))
            .connect_lazy(database_url)
            .map_err(|e| {
                error!("Failed to create connection pool: {}", e);
                AttestationError::StorageError(format!("Failed to create connection pool: {}", e))
            })?;

        // Add migration retry logic
        let max_retries = 3;
        let mut last_error = None;

        for attempt in 0..max_retries {
            match sqlx::migrate!("./migrations").run(&pool).await {
                Ok(_) => break,
                Err(e) => {
                    last_error = Some(e);
                    if attempt < max_retries - 1 {
                        tokio::time::sleep(Duration::from_secs(1 << attempt)).await;
                        continue;
                    }
                    error!(
                        "Failed to run migrations after {} attempts: {}",
                        max_retries, e
                    );
                    return Err(AttestationError::StorageError(format!(
                        "Failed to run migrations: {}",
                        e
                    )));
                }
            }
        }

        Ok(Self { pool })
    }

    pub async fn store_attestation(&self, info: &HardwareInfo) -> Result<Uuid, AttestationError> {
        let mut tx = self.pool.begin().await.map_err(|e| {
            error!("Failed to start transaction: {}", e);
            AttestationError::StorageError(format!("Failed to start transaction: {}", e))
        })?;

        // Insert main attestation record with error handling
        sqlx::query!(
            r#"
            INSERT INTO attestations (id, gpu_count, timestamp)
            VALUES ($1, $2, $3)
            ON CONFLICT (id) DO UPDATE
            SET gpu_count = EXCLUDED.gpu_count,
                timestamp = EXCLUDED.timestamp
            "#,
            info.id,
            info.gpu_count as i32,
            info.timestamp,
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            error!("Failed to insert attestation: {}", e);
            AttestationError::StorageError(format!("Failed to insert attestation: {}", e))
        })?;

        // Insert GPU details with batch processing
        let mut gpu_inserts = Vec::new();
        for gpu in &info.gpus {
            gpu_inserts.push(sqlx::query!(
                r#"
                INSERT INTO gpu_details (
                    attestation_id, gpu_index, name, memory_total,
                    compute_capability, power_usage, temperature
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                ON CONFLICT (attestation_id, gpu_index) DO UPDATE
                SET name = EXCLUDED.name,
                    memory_total = EXCLUDED.memory_total,
                    compute_capability = EXCLUDED.compute_capability,
                    power_usage = EXCLUDED.power_usage,
                    temperature = EXCLUDED.temperature
                "#,
                info.id,
                gpu.index as i32,
                gpu.name,
                gpu.memory_total as i64,
                gpu.compute_capability,
                gpu.power_usage.map(|v| v as i32),
                gpu.temperature.map(|v| v as i32),
            ));
        }

        for query in gpu_inserts {
            query.execute(&mut *tx).await.map_err(|e| {
                error!("Failed to insert GPU details: {}", e);
                AttestationError::StorageError(format!("Failed to insert GPU details: {}", e))
            })?;
        }

        tx.commit().await.map_err(|e| {
            error!("Failed to commit transaction: {}", e);
            AttestationError::StorageError(format!("Failed to commit transaction: {}", e))
        })?;

        info!("Stored attestation record with ID: {}", info.id);
        Ok(info.id)
    }

    pub async fn get_attestation(&self, id: Uuid) -> Result<HardwareInfo, AttestationError> {
        // Add query timeout
        let attestation = sqlx::query!(
            r#"
            SELECT id, gpu_count, timestamp
            FROM attestations
            WHERE id = $1
            "#,
            id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to fetch attestation: {}", e);
            AttestationError::StorageError(format!("Failed to fetch attestation: {}", e))
        })?;

        let gpus = sqlx::query!(
            r#"
            SELECT gpu_index, name, memory_total, compute_capability, power_usage, temperature
            FROM gpu_details
            WHERE attestation_id = $1
            ORDER BY gpu_index
            "#,
            id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to fetch GPU details: {}", e);
            AttestationError::StorageError(format!("Failed to fetch GPU details: {}", e))
        })?;

        let gpu_infos = gpus
            .into_iter()
            .map(|row| crate::GpuInfo {
                index: row.gpu_index as usize,
                name: row.name,
                memory_total: row.memory_total as u64,
                compute_capability: row.compute_capability,
                power_usage: row.power_usage.map(|v| v as u32),
                temperature: row.temperature.map(|v| v as u32),
            })
            .collect();

        Ok(HardwareInfo {
            id: attestation.id,
            gpu_count: attestation.gpu_count as usize,
            gpus: gpu_infos,
            timestamp: attestation.timestamp,
        })
    }
}
