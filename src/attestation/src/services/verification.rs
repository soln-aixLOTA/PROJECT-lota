use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::utils::{crypto::verify_signature, validation::validate_attestation_data};

pub struct VerificationService {
    pool: Arc<PgPool>,
}

impl VerificationService {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn verify_attestation(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        // Get attestation from database
        let attestation = sqlx::query!(
            r#"
            SELECT attestation_data, attestation_type, signature, public_key
            FROM attestations
            WHERE id = $1 AND status = 'pending'
            "#,
            id
        )
        .fetch_optional(&*self.pool)
        .await?;

        let Some(attestation) = attestation else {
            return Ok(false);
        };

        // Verify signature
        let data_str = serde_json::to_string(&attestation.attestation_data)
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        if !verify_signature(&data_str, &attestation.signature, &attestation.public_key) {
            // Update status to failed
            sqlx::query!(
                r#"
                UPDATE attestations
                SET status = 'failed', updated_at = NOW()
                WHERE id = $1
                "#,
                id
            )
            .execute(&*self.pool)
            .await?;

            return Ok(false);
        }

        // Validate attestation data
        if let Err(_) =
            validate_attestation_data(&attestation.attestation_data, &attestation.attestation_type)
        {
            // Update status to failed
            sqlx::query!(
                r#"
                UPDATE attestations
                SET status = 'failed', updated_at = NOW()
                WHERE id = $1
                "#,
                id
            )
            .execute(&*self.pool)
            .await?;

            return Ok(false);
        }

        // Update status to verified
        sqlx::query!(
            r#"
            UPDATE attestations
            SET status = 'verified', updated_at = NOW()
            WHERE id = $1
            "#,
            id
        )
        .execute(&*self.pool)
        .await?;

        Ok(true)
    }
}
