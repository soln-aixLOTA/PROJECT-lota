use sqlx::PgPool;
use uuid::Uuid;

use crate::core::error::{DocumentError, DocumentResult};
use crate::models::workflow::{
    ExecutionStep, StepStatus, StepType, Workflow, WorkflowExecution, WorkflowMetadata,
    WorkflowStatus, WorkflowStep,
};

#[derive(Debug, Clone)]
pub struct WorkflowRepository;

impl WorkflowRepository {
    pub async fn create_workflow(pool: &PgPool, workflow: &Workflow) -> DocumentResult<Workflow> {
        let record = sqlx::query!(
            r#"
            INSERT INTO workflows (
                id, name, description, status, creator,
                metadata, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
            workflow.id,
            workflow.name,
            workflow.description,
            workflow.status as WorkflowStatus,
            workflow.creator,
            serde_json::to_value(&workflow.metadata)?,
            workflow.created_at,
            workflow.updated_at,
        )
        .fetch_one(pool)
        .await?;

        Ok(Workflow {
            id: record.id,
            name: record.name,
            description: record.description.unwrap_or_default(),
            status: record.status.parse()?,
            creator: record.creator,
            metadata: serde_json::from_value(record.metadata)?,
            created_at: record.created_at,
            updated_at: record.updated_at,
        })
    }

    pub async fn create_workflow_step(
        pool: &PgPool,
        step: &WorkflowStep,
    ) -> DocumentResult<WorkflowStep> {
        let record = sqlx::query!(
            r#"
            INSERT INTO workflow_steps (
                id, workflow_id, name, step_type,
                order_num, config, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
            step.id,
            step.workflow_id,
            step.name,
            step.step_type as StepType,
            step.order,
            step.config,
            step.created_at,
            step.updated_at,
        )
        .fetch_one(pool)
        .await?;

        Ok(WorkflowStep {
            id: record.id,
            workflow_id: record.workflow_id,
            name: record.name,
            step_type: record.step_type.parse()?,
            order: record.order_num,
            config: record.config,
            created_at: record.created_at,
            updated_at: record.updated_at,
        })
    }

    pub async fn get_workflow(pool: &PgPool, id: &str) -> DocumentResult<Workflow> {
        let record = sqlx::query!(
            r#"
            SELECT 
                id, name, description, status, creator,
                metadata, created_at, updated_at
            FROM workflows
            WHERE id = $1
            "#,
            Uuid::parse_str(id)?
        )
        .fetch_one(pool)
        .await?;

        Ok(Workflow {
            id: record.id,
            name: record.name,
            description: record.description.unwrap_or_default(),
            status: record.status.parse()?,
            creator: record.creator,
            metadata: serde_json::from_value(record.metadata)?,
            created_at: record.created_at,
            updated_at: record.updated_at,
        })
    }

    pub async fn get_workflow_steps(
        pool: &PgPool,
        workflow_id: &str,
    ) -> DocumentResult<Vec<WorkflowStep>> {
        let records = sqlx::query!(
            r#"
            SELECT 
                id, workflow_id, name, step_type,
                order_num, config, created_at, updated_at
            FROM workflow_steps
            WHERE workflow_id = $1
            ORDER BY order_num ASC
            "#,
            Uuid::parse_str(workflow_id)?
        )
        .fetch_all(pool)
        .await?;

        let mut steps = Vec::new();
        for record in records {
            steps.push(WorkflowStep {
                id: record.id,
                workflow_id: record.workflow_id,
                name: record.name,
                step_type: record.step_type.parse()?,
                order: record.order_num,
                config: record.config,
                created_at: record.created_at,
                updated_at: record.updated_at,
            });
        }

        Ok(steps)
    }

    pub async fn update_workflow_status(
        pool: &PgPool,
        id: &str,
        status: WorkflowStatus,
    ) -> DocumentResult<()> {
        sqlx::query!(
            r#"
            UPDATE workflows
            SET status = $1
            WHERE id = $2
            "#,
            status as WorkflowStatus,
            Uuid::parse_str(id)?
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn update_step_status(
        pool: &PgPool,
        step_id: &str,
        status: StepStatus,
        result: Option<serde_json::Value>,
    ) -> DocumentResult<()> {
        sqlx::query!(
            r#"
            UPDATE workflow_steps
            SET status = $1, result = $2
            WHERE id = $3
            "#,
            status as StepStatus,
            result,
            Uuid::parse_str(step_id)?
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn list_workflows(
        pool: &PgPool,
        offset: i64,
        limit: i64,
        status: Option<WorkflowStatus>,
    ) -> DocumentResult<Vec<Workflow>> {
        let records = sqlx::query!(
            r#"
            SELECT 
                id, name, description, status, creator,
                metadata, created_at, updated_at
            FROM workflows
            WHERE ($1::workflow_status IS NULL OR status = $1)
            ORDER BY created_at DESC
            OFFSET $2
            LIMIT $3
            "#,
            status as Option<WorkflowStatus>,
            offset,
            limit
        )
        .fetch_all(pool)
        .await?;

        let mut workflows = Vec::new();
        for record in records {
            workflows.push(Workflow {
                id: record.id,
                name: record.name,
                description: record.description.unwrap_or_default(),
                status: record.status.parse()?,
                creator: record.creator,
                metadata: serde_json::from_value(record.metadata)?,
                created_at: record.created_at,
                updated_at: record.updated_at,
            });
        }

        Ok(workflows)
    }

    pub async fn delete_workflow(pool: &PgPool, id: &str) -> DocumentResult<()> {
        let result = sqlx::query!(
            r#"
            DELETE FROM workflows
            WHERE id = $1
            "#,
            Uuid::parse_str(id)?
        )
        .execute(pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(DocumentError::NotFound("Workflow not found".to_string()));
        }

        Ok(())
    }

    pub async fn create_workflow_execution(
        pool: &PgPool,
        execution: &WorkflowExecution,
    ) -> DocumentResult<WorkflowExecution> {
        let record = sqlx::query!(
            r#"
            INSERT INTO document_workflow_executions (
                id, document_id, workflow_id, status,
                started_at, completed_at, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
            execution.id,
            execution.document_id,
            execution.workflow_id,
            execution.status as StepStatus,
            execution.started_at,
            execution.completed_at,
            execution.created_at,
            execution.updated_at,
        )
        .fetch_one(pool)
        .await?;

        Ok(WorkflowExecution {
            id: record.id,
            document_id: record.document_id,
            workflow_id: record.workflow_id,
            status: record.status.parse()?,
            started_at: record.started_at,
            completed_at: record.completed_at,
            created_at: record.created_at,
            updated_at: record.updated_at,
        })
    }

    pub async fn create_execution_step(
        pool: &PgPool,
        step: &ExecutionStep,
    ) -> DocumentResult<ExecutionStep> {
        let record = sqlx::query!(
            r#"
            INSERT INTO execution_steps (
                id, execution_id, step_id, status,
                result, started_at, completed_at, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
            step.id,
            step.execution_id,
            step.step_id,
            step.status as StepStatus,
            step.result,
            step.started_at,
            step.completed_at,
            step.created_at,
            step.updated_at,
        )
        .fetch_one(pool)
        .await?;

        Ok(ExecutionStep {
            id: record.id,
            execution_id: record.execution_id,
            step_id: record.step_id,
            status: record.status.parse()?,
            result: record.result,
            started_at: record.started_at,
            completed_at: record.completed_at,
            created_at: record.created_at,
            updated_at: record.updated_at,
        })
    }
}
