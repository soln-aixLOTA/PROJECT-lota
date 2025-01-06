use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgHasArrayType;
use strum::{Display, EnumString};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkflowMetadata {
    pub tags: Vec<String>,
    pub description: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Display, EnumString, sqlx::Type)]
#[sqlx(type_name = "workflow_status", rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum WorkflowStatus {
    Draft,
    Active,
    Archived,
    Deleted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub status: WorkflowStatus,
    pub creator: String,
    pub metadata: WorkflowMetadata,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Display, EnumString, sqlx::Type)]
#[sqlx(type_name = "step_type", rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum StepType {
    Extract,
    Transform,
    Load,
    Validate,
    Notify,
}

#[derive(Debug, Clone, Serialize, Deserialize, Display, EnumString, sqlx::Type)]
#[sqlx(type_name = "step_status", rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum StepStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub name: String,
    pub step_type: StepType,
    pub status: StepStatus,
    pub order: i32,
    pub config: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecution {
    pub id: Uuid,
    pub document_id: Uuid,
    pub workflow_id: Uuid,
    pub status: StepStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStep {
    pub id: Uuid,
    pub execution_id: Uuid,
    pub step_id: Uuid,
    pub status: StepStatus,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
