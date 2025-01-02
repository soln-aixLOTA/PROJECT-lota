use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct InferenceRequest {
    pub agent_id: Uuid,
    pub model_id: String,
    pub input: String,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InferenceResponse {
    pub id: Uuid,
    pub agent_id: Uuid,
    pub model_id: String,
    pub input: String,
    pub output: String,
    pub confidence_score: f64,
    pub metadata: serde_json::Value,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct InferenceRecord {
    pub id: Uuid,
    pub agent_id: Uuid,
    pub model_id: String,
    pub input: String,
    pub output: String,
    pub confidence_score: f64,
    pub metadata: serde_json::Value,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchInferenceRequest {
    pub agent_id: Uuid,
    pub model_id: String,
    pub inputs: Vec<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchInferenceResponse {
    pub results: Vec<InferenceResponse>,
    pub metadata: serde_json::Value,
}
