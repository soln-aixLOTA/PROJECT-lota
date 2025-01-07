use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "workflow_status", rename_all = "snake_case")]
pub enum WorkflowStatus {
    Draft,
    Active,
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

}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub status: WorkflowStatus,
    pub creator: String,
    pub metadata: WorkflowMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub name: String,
    pub step_type: StepType,
}

impl WorkflowStep {
    pub fn new(
        workflow_id: Uuid,
        name: String,
        step_type: StepType,
        order: i32,
        }
    }
}

            started_at: now,
            completed_at: None,
            created_at: now,
            updated_at: now,
        }
    }
}

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
