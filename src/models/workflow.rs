use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "workflow_status", rename_all = "snake_case")]
pub enum WorkflowStatus {
    Draft,
    Active,
    Paused,
    Completed,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "step_type", rename_all = "snake_case")]
pub enum StepType {
    Review,
    Approval,
    Notification,
    Integration,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "step_status", rename_all = "snake_case")]
pub enum StepStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowMetadata {
    pub description: String,
    pub version: String,
    pub tags: Vec<String>,
    pub custom_fields: serde_json::Value,
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
    pub status: WorkflowStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
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
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Workflow {
    pub fn new(
        name: String,
        description: String,
        creator: String,
        metadata: WorkflowMetadata,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            status: WorkflowStatus::Draft,
            creator,
            metadata,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

impl WorkflowStep {
    pub fn new(
        workflow_id: Uuid,
        name: String,
        step_type: StepType,
        order: i32,
        config: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            workflow_id,
            name,
            step_type,
            status: StepStatus::Pending,
            order,
            config,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

impl WorkflowExecution {
    pub fn new(document_id: Uuid, workflow_id: Uuid) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            document_id,
            workflow_id,
            status: WorkflowStatus::Draft,
            started_at: now,
            completed_at: None,
            created_at: now,
            updated_at: now,
        }
    }
}

impl ExecutionStep {
    pub fn new(execution_id: Uuid, step_id: Uuid) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            execution_id,
            step_id,
            status: StepStatus::Pending,
            result: None,
            started_at: now,
            completed_at: None,
            created_at: now,
            updated_at: now,
        }
    }
}
