use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct AttestationRequest {
    pub model_id: Uuid,
    pub version_id: Uuid,
    pub attestation_type: String,
    pub attestation_data: serde_json::Value,
    pub signature: String,
    pub public_key: String,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Attestation {
    pub id: Uuid,
    pub model_id: Uuid,
    pub version_id: Uuid,
    pub attestation_type: String,
    pub attestation_data: serde_json::Value,
    pub signature: String,
    pub public_key: String,
    pub metadata: Option<serde_json::Value>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub revocation_reason: Option<String>,
}
