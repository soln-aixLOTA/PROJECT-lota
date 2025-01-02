use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::utils::crypto::verify_signature;

pub struct VerificationService {
    pool: PgPool,
}

impl VerificationService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn verify_attestation(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        // Get attestation data
        let attestation = sqlx::query!(
            r#"
            SELECT attestation_data, attestation_type, signature, public_key,
                   version_id, agent_id, input_hash, output_hash
            FROM attestations
            WHERE id = $1
            "#,
            id
        )
        .fetch_one(&self.pool)
        .await?;

        // Verify signature
        let data = json!({
            "version_id": attestation.version_id,
            "agent_id": attestation.agent_id,
            "attestation_type": attestation.attestation_type,
            "attestation_data": attestation.attestation_data,
            "input_hash": attestation.input_hash,
            "output_hash": attestation.output_hash,
        });
        let data_str = data.to_string();

        if !verify_signature(
            data_str.as_bytes(),
            &attestation.signature,
            &attestation.public_key,
        ) {
            sqlx::query!(
                r#"
                UPDATE attestations
                SET status = 'failed', updated_at = NOW()
                WHERE id = $1
                "#,
                id
            )
            .execute(&self.pool)
            .await?;
            return Ok(false);
        }

        // Additional verification logic can be added here
        // For example, verifying input/output hashes, checking attestation data format, etc.
        if false {
            sqlx::query!(
                r#"
                UPDATE attestations
                SET status = 'failed', updated_at = NOW()
                WHERE id = $1
                "#,
                id
            )
            .execute(&self.pool)
            .await?;
            return Ok(false);
        }

        // Mark as verified
        sqlx::query!(
            r#"
            UPDATE attestations
            SET status = 'verified', updated_at = NOW(), verified_at = NOW()
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(true)
    }
}
