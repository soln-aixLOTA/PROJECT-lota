use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Attestation {
    pub id: Uuid,
    pub version_id: Uuid,
    pub agent_id: Uuid,
    pub attestation_type: String,
    pub attestation_data: serde_json::Value,
    pub input_hash: String,
    pub output_hash: String,
    pub signature: String,
    pub public_key: String,
    pub confidence_score: Option<f64>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub verified_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct AttestationRequest {
    pub version_id: Uuid,
    pub agent_id: Uuid,
    #[validate(length(min = 1, max = 50))]
    pub attestation_type: String,
    pub attestation_data: serde_json::Value,
    pub input_hash: String,
    pub output_hash: String,
    pub signature: String,
    pub public_key: String,
    #[validate(range(min = 0.0, max = 100.0))]
    pub confidence_score: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListAttestationsQuery {
    pub version_id: Option<Uuid>,
    pub agent_id: Option<Uuid>,
    pub attestation_type: Option<String>,
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
