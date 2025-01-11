use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: Uuid,
    pub name: String,
    pub agent_type: AgentType,
    pub capabilities: Vec<Capability>,
    pub status: AgentStatus,
    pub metadata: HashMap<String, String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentType {
    Reader,
    Analyzer,
    Planner,
    Architect,
    Developer,
    Tester,
    Reviewer,
    Observer,
    Optimizer,
    TaskExecutor,
    Documentation,
    Security,
    Communication,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Capability {
    CodeParsing,
    CodeAnalysis,
    Planning,
    Architecture,
    Development,
    Testing,
    CodeReview,
    Monitoring,
    Optimization,
    TaskExecution,
    Documentation,
    Security,
    Communication,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentStatus {
    Available,
    Busy,
    Error,
    Offline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub task_type: TaskType,
    pub priority: Priority,
    pub status: TaskStatus,
    pub assigned_to: Option<Uuid>,
    pub dependencies: Vec<Uuid>,
    pub metadata: HashMap<String, String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub deadline: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskType {
    CodeParsing,
    Analysis,
    Planning,
    ArchitectureReview,
    Development,
    Testing,
    CodeReview,
    Monitoring,
    Optimization,
    Deployment,
    Documentation,
    SecurityAudit,
    Communication,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRegistration {
    pub name: String,
    pub agent_type: AgentType,
    pub capabilities: Vec<Capability>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAssignment {
    pub task_id: Uuid,
    pub agent_id: Uuid,
    pub priority: Priority,
    pub deadline: Option<chrono::DateTime<chrono::Utc>>,
}
