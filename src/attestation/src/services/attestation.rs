use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    models::{Attestation, AttestationRequest},
    utils::crypto::verify_signature,
};

pub struct AttestationService {
    pool: PgPool,
}

impl AttestationService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_attestation(
        &self,
        req: AttestationRequest,
    ) -> Result<Attestation, sqlx::Error> {
        // Verify signature
        let data = json!({
            "version_id": req.version_id,
            "agent_id": req.agent_id,
            "attestation_type": req.attestation_type,
            "attestation_data": req.attestation_data,
            "input_hash": req.input_hash,
            "output_hash": req.output_hash,
        });
        let data_str = data.to_string();

        if !verify_signature(data_str.as_bytes(), &req.signature, &req.public_key) {
            return Err(sqlx::Error::Protocol("Invalid signature".into()));
        }

        let attestation = sqlx::query_as!(
            Attestation,
            r#"
            INSERT INTO attestations (
                version_id,
                agent_id,
                attestation_type,
                attestation_data,
                input_hash,
                output_hash,
                signature,
                public_key,
                confidence_score,
                status
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#,
            req.version_id,
            req.agent_id,
            req.attestation_type,
            req.attestation_data as _,
            req.input_hash,
            req.output_hash,
            req.signature,
            req.public_key,
            req.confidence_score,
            "pending" as _
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(attestation)
    }

    pub async fn get_attestation(&self, id: Uuid) -> Result<Option<Attestation>, sqlx::Error> {
        sqlx::query_as!(Attestation, "SELECT * FROM attestations WHERE id = $1", id)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn list_attestations(
        &self,
        version_id: Option<Uuid>,
        agent_id: Option<Uuid>,
        attestation_type: Option<String>,
        status: Option<String>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<Attestation>, sqlx::Error> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);

        let attestations = sqlx::query_as!(
            Attestation,
            r#"
            SELECT *
            FROM attestations
            WHERE
                ($1::uuid IS NULL OR version_id = $1) AND
                ($2::uuid IS NULL OR agent_id = $2) AND
                ($3::text IS NULL OR attestation_type = $3) AND
                ($4::text IS NULL OR status = $4)
            ORDER BY created_at DESC
            LIMIT $5 OFFSET $6
            "#,
            version_id,
            agent_id,
            attestation_type,
            status,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(attestations)
    }
}
