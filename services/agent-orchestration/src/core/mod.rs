use crate::models::{Agent, AgentRegistration, AgentStatus, Task, TaskAssignment, TaskStatus};
use anyhow::Result;
use async_nats::Client as NatsClient;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

pub struct AgentManager {
    nats: NatsClient,
    agents: Arc<Mutex<Vec<Agent>>>,
    tasks: Arc<Mutex<Vec<Task>>>,
}

impl AgentManager {
    pub fn new(nats: NatsClient) -> Self {
        Self {
            nats,
            agents: Arc::new(Mutex::new(Vec::new())),
            tasks: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn register_agent(&self, registration: AgentRegistration) -> Result<Agent> {
        let agent = Agent {
            id: Uuid::new_v4(),
            name: registration.name,
            agent_type: registration.agent_type,
            capabilities: registration.capabilities,
            status: AgentStatus::Available,
            metadata: registration.metadata,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // Publish agent registration event
        self.nats
            .publish("agents.registered", serde_json::to_string(&agent)?.into())
            .await?;

        // Store agent
        self.agents.lock().await.push(agent.clone());

        Ok(agent)
    }

    pub async fn get_agent(&self, id: Uuid) -> Option<Agent> {
        self.agents
            .lock()
            .await
            .iter()
            .find(|a| a.id == id)
            .cloned()
    }

    pub async fn assign_task(&self, assignment: TaskAssignment) -> Result<Task> {
        let mut agents = self.agents.lock().await;
        let mut tasks = self.tasks.lock().await;

        // Find the agent
        let agent = agents
            .iter_mut()
            .find(|a| a.id == assignment.agent_id)
            .ok_or_else(|| anyhow::anyhow!("Agent not found"))?;

        // Check if agent is available
        if agent.status != AgentStatus::Available {
            return Err(anyhow::anyhow!("Agent is not available"));
        }

        // Create task
        let task = Task {
            id: assignment.task_id,
            title: "New Task".to_string(), // This should come from the assignment
            description: "Task Description".to_string(), // This should come from the assignment
            task_type: match agent.agent_type {
                crate::models::AgentType::Reader => crate::models::TaskType::CodeParsing,
                crate::models::AgentType::Analyzer => crate::models::TaskType::Analysis,
                // Add other mappings...
                _ => crate::models::TaskType::Development,
            },
            priority: assignment.priority,
            status: TaskStatus::Pending,
            assigned_to: Some(agent.id),
            dependencies: Vec::new(),
            metadata: Default::default(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            deadline: assignment.deadline,
        };

        // Update agent status
        agent.status = AgentStatus::Busy;

        // Publish task assignment event
        self.nats
            .publish("tasks.assigned", serde_json::to_string(&task)?.into())
            .await?;

        // Store task
        tasks.push(task.clone());

        Ok(task)
    }

    pub async fn update_task_status(&self, task_id: Uuid, status: TaskStatus) -> Result<Task> {
        let mut tasks = self.tasks.lock().await;
        let mut agents = self.agents.lock().await;

        // Find and update task
        let task = tasks
            .iter_mut()
            .find(|t| t.id == task_id)
            .ok_or_else(|| anyhow::anyhow!("Task not found"))?;

        task.status = status.clone();
        task.updated_at = chrono::Utc::now();

        // If task is completed or failed, update agent status
        if let Some(agent_id) = task.assigned_to {
            if matches!(status, TaskStatus::Completed | TaskStatus::Failed) {
                if let Some(agent) = agents.iter_mut().find(|a| a.id == agent_id) {
                    agent.status = AgentStatus::Available;
                }
            }
        }

        // Publish task update event
        self.nats
            .publish("tasks.updated", serde_json::to_string(&task)?.into())
            .await?;

        Ok(task.clone())
    }

    pub async fn list_tasks(&self) -> Vec<Task> {
        self.tasks.lock().await.clone()
    }

    pub async fn get_task(&self, id: Uuid) -> Option<Task> {
        self.tasks.lock().await.iter().find(|t| t.id == id).cloned()
    }
}
