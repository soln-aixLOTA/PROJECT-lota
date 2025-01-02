use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::errors::Error;
use crate::models::{InferenceRecord, InferenceRequest};

pub struct InferenceRepository {
    pool: PgPool,
}

impl InferenceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        request: &InferenceRequest,
        output: String,
        confidence_score: f64,
    ) -> Result<InferenceRecord, Error> {
        let now = OffsetDateTime::now_utc();
        let id = Uuid::new_v4();
        let metadata = request.metadata.clone().unwrap_or(serde_json::json!({}));

        let record = sqlx::query_as!(
            InferenceRecord,
            r#"
            INSERT INTO inference_records (
                id, agent_id, model_id, input, output,
                confidence_score, metadata, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, agent_id, model_id, input, output,
                      confidence_score, metadata as "metadata: serde_json::Value",
                      created_at
            "#,
            id,
            request.agent_id,
            request.model_id,
            request.input,
            output,
            confidence_score,
            metadata,
            now
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::Database)?;

        Ok(record)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<InferenceRecord>, Error> {
        let record = sqlx::query_as!(
            InferenceRecord,
            r#"
            SELECT id, agent_id, model_id, input, output,
                   confidence_score, metadata as "metadata: serde_json::Value",
                   created_at
            FROM inference_records
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(Error::Database)?;

        Ok(record)
    }

    pub async fn find_by_agent(&self, agent_id: Uuid) -> Result<Vec<InferenceRecord>, Error> {
        let records = sqlx::query_as!(
            InferenceRecord,
            r#"
            SELECT id, agent_id, model_id, input, output,
                   confidence_score, metadata as "metadata: serde_json::Value",
                   created_at
            FROM inference_records
            WHERE agent_id = $1
            ORDER BY created_at DESC
            "#,
            agent_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Error::Database)?;

        Ok(records)
    }
}
