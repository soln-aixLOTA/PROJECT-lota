use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    models::{Attestation, AttestationRequest},
    utils::crypto::verify_signature,
};

pub struct AttestationService {
    pool: Arc<PgPool>,
}

impl AttestationService {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn create_attestation(
        &self,
        req: AttestationRequest,
    ) -> Result<Attestation, sqlx::Error> {
        // Verify signature
        let data_str = serde_json::to_string(&req.attestation_data)
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        if !verify_signature(&data_str, &req.signature, &req.public_key) {
            return Err(sqlx::Error::Protocol("Invalid signature".into()));
        }

        // Insert into database
        let attestation = sqlx::query_as!(
            Attestation,
            r#"
            INSERT INTO attestations (
                model_id, version_id, attestation_type, attestation_data,
                signature, public_key, metadata, status
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
            req.model_id,
            req.version_id,
            req.attestation_type,
            req.attestation_data as _,
            req.signature,
            req.public_key,
            req.metadata as _,
            "pending" as _
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(attestation)
    }

    pub async fn get_attestation(&self, id: Uuid) -> Result<Option<Attestation>, sqlx::Error> {
        sqlx::query_as!(Attestation, "SELECT * FROM attestations WHERE id = $1", id)
            .fetch_optional(&*self.pool)
            .await
    }

    pub async fn list_attestations(
        &self,
        model_id: Option<Uuid>,
        version_id: Option<Uuid>,
        status: Option<String>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Attestation>, sqlx::Error> {
        let attestations = sqlx::query_as!(
            Attestation,
            r#"
            SELECT *
            FROM attestations
            WHERE ($1::uuid IS NULL OR model_id = $1)
            AND ($2::uuid IS NULL OR version_id = $2)
            AND ($3::text IS NULL OR status = $3)
            ORDER BY created_at DESC
            LIMIT $4 OFFSET $5
            "#,
            model_id,
            version_id,
            status,
            limit,
            offset
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(attestations)
    }
}
