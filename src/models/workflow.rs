<<<<<<< HEAD
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Type;
=======
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::Type;
use std::str::FromStr;
>>>>>>> 921251a (fetch)
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "workflow_status", rename_all = "snake_case")]
pub enum WorkflowStatus {
    Draft,
    Active,
<<<<<<< HEAD
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
=======
    Completed,
    Failed,
    Archived,
}

impl FromStr for WorkflowStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "draft" => Ok(WorkflowStatus::Draft),
            "active" => Ok(WorkflowStatus::Active),
            "completed" => Ok(WorkflowStatus::Completed),
            "failed" => Ok(WorkflowStatus::Failed),
            "archived" => Ok(WorkflowStatus::Archived),
            _ => Err(format!("Invalid workflow status: {}", s)),
        }
    }
}

impl From<String> for WorkflowStatus {
    fn from(s: String) -> Self {
        s.parse().unwrap_or(WorkflowStatus::Draft)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "step_type", rename_all = "snake_case")]
pub enum StepType {
    Validation,
    Transformation,
    Approval,
    Notification,
    Integration,
}

impl FromStr for StepType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "validation" => Ok(StepType::Validation),
            "transformation" => Ok(StepType::Transformation),
            "approval" => Ok(StepType::Approval),
            "notification" => Ok(StepType::Notification),
            "integration" => Ok(StepType::Integration),
            _ => Err(format!("Invalid step type: {}", s)),
        }
    }
}

impl From<String> for StepType {
    fn from(s: String) -> Self {
        s.parse().unwrap_or(StepType::Validation)
    }
>>>>>>> 921251a (fetch)
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

<<<<<<< HEAD
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowMetadata {
    pub description: String,
    pub version: String,
    pub tags: Vec<String>,
    pub custom_fields: serde_json::Value,
=======
impl FromStr for StepStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(StepStatus::Pending),
            "in_progress" => Ok(StepStatus::InProgress),
            "completed" => Ok(StepStatus::Completed),
            "failed" => Ok(StepStatus::Failed),
            "skipped" => Ok(StepStatus::Skipped),
            _ => Err(format!("Invalid step status: {}", s)),
        }
    }
}

impl From<String> for StepStatus {
    fn from(s: String) -> Self {
        s.parse().unwrap_or(StepStatus::Pending)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowMetadata {
    pub description: Option<String>,
    pub version: String,
    pub custom_fields: Option<JsonValue>,
}

impl From<JsonValue> for WorkflowMetadata {
    fn from(value: JsonValue) -> Self {
        serde_json::from_value(value).unwrap_or_default()
    }
}

impl From<Option<JsonValue>> for WorkflowMetadata {
    fn from(value: Option<JsonValue>) -> Self {
        value.map(|v| v.into()).unwrap_or_default()
    }
}

impl Default for WorkflowMetadata {
    fn default() -> Self {
        Self {
            description: None,
            version: "1.0".to_string(),
            custom_fields: None,
        }
    }
>>>>>>> 921251a (fetch)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub status: WorkflowStatus,
    pub creator: String,
    pub metadata: WorkflowMetadata,
<<<<<<< HEAD
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
=======
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Workflow {
    pub fn new(name: String, description: String, creator: String) -> Self {
        let id = Uuid::new_v4();
        let now = chrono::Utc::now();
        Self {
            id,
            name,
            description,
            status: WorkflowStatus::Draft,
            creator,
            metadata: WorkflowMetadata::default(),
            created_at: now,
            updated_at: now,
        }
    }
>>>>>>> 921251a (fetch)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub name: String,
    pub step_type: StepType,
<<<<<<< HEAD
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
=======
    pub order: i32,
    pub config: JsonValue,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
>>>>>>> 921251a (fetch)
}

impl WorkflowStep {
    pub fn new(
        workflow_id: Uuid,
        name: String,
        step_type: StepType,
        order: i32,
<<<<<<< HEAD
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
=======
        config: JsonValue,
    ) -> Self {
        let id = Uuid::new_v4();
        let now = chrono::Utc::now();
        Self {
            id,
            workflow_id,
            name,
            step_type,
            order,
            config,
            created_at: now,
            updated_at: now,
>>>>>>> 921251a (fetch)
        }
    }
}

<<<<<<< HEAD
impl WorkflowExecution {
    pub fn new(document_id: Uuid, workflow_id: Uuid) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            document_id,
            workflow_id,
            status: WorkflowStatus::Draft,
=======
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecution {
    pub id: Uuid,
    pub document_id: Uuid,
    pub workflow_id: Uuid,
    pub status: StepStatus,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl WorkflowExecution {
    pub fn new(document_id: Uuid, workflow_id: Uuid) -> Self {
        let id = Uuid::new_v4();
        let now = chrono::Utc::now();
        Self {
            id,
            document_id,
            workflow_id,
            status: StepStatus::Pending,
>>>>>>> 921251a (fetch)
            started_at: now,
            completed_at: None,
            created_at: now,
            updated_at: now,
        }
    }
}

<<<<<<< HEAD
impl ExecutionStep {
    pub fn new(execution_id: Uuid, step_id: Uuid) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
=======
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStep {
    pub id: Uuid,
    pub execution_id: Uuid,
    pub step_id: Uuid,
    pub status: StepStatus,
    pub result: Option<JsonValue>,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl ExecutionStep {
    pub fn new(execution_id: Uuid, step_id: Uuid) -> Self {
        let id = Uuid::new_v4();
        let now = chrono::Utc::now();
        Self {
            id,
>>>>>>> 921251a (fetch)
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
