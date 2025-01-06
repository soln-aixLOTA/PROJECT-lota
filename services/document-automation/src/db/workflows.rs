use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    core::error::{DocumentError, DocumentResult},
    models::{
        ExecutionStep, StepStatus, StepType, Workflow, WorkflowExecution, WorkflowMetadata,
        WorkflowStatus, WorkflowStep,
    },
};

pub struct WorkflowRepository;

impl WorkflowRepository {
    pub async fn create_workflow(pool: &PgPool, workflow: Workflow) -> DocumentResult<Workflow> {
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
            workflow.status as _,
            workflow.creator,
            serde_json::to_value(&workflow.metadata)? as _,
            workflow.created_at,
            workflow.updated_at,
        )
        .fetch_one(pool)
        .await?;

        Ok(Workflow {
            id: record.id,
            name: record.name,
            description: record.description,
            status: record.status.parse()?,
            creator: record.creator,
            metadata: serde_json::from_value(record.metadata)?,
            created_at: record.created_at,
            updated_at: record.updated_at,
        })
    }

    pub async fn create_workflow_step(
        pool: &PgPool,
        step: WorkflowStep,
    ) -> DocumentResult<WorkflowStep> {
        let record = sqlx::query!(
            r#"
            INSERT INTO workflow_steps (
                id, workflow_id, name, step_type,
                status, order_num, config,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
            step.id,
            step.workflow_id,
            step.name,
            step.step_type as _,
            step.status as _,
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
            status: record.status.parse()?,
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
            description: record.description,
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
                status, order_num, config,
                created_at, updated_at
            FROM workflow_steps
            WHERE workflow_id = $1
            ORDER BY order_num ASC
            "#,
            Uuid::parse_str(workflow_id)?
        )
        .fetch_all(pool)
        .await?;

        let steps = records
            .into_iter()
            .map(|record| WorkflowStep {
                id: record.id,
                workflow_id: record.workflow_id,
                name: record.name,
                step_type: record.step_type.parse().unwrap(),
                status: record.status.parse().unwrap(),
                order: record.order_num,
                config: record.config,
                created_at: record.created_at,
                updated_at: record.updated_at,
            })
            .collect();

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
            status as _,
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
            status as _,
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
            WHERE ($3::workflow_status IS NULL OR status = $3)
            ORDER BY created_at DESC
            LIMIT $1
            OFFSET $2
            "#,
            limit,
            offset,
            status as _
        )
        .fetch_all(pool)
        .await?;

        let workflows = records
            .into_iter()
            .map(|record| Workflow {
                id: record.id,
                name: record.name,
                description: record.description,
                status: record.status.parse().unwrap(),
                creator: record.creator,
                metadata: serde_json::from_value(record.metadata).unwrap(),
                created_at: record.created_at,
                updated_at: record.updated_at,
            })
            .collect();

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
            return Err(DocumentError::NotFound);
        }

        Ok(())
    }

    pub async fn create_workflow_execution(
        pool: &PgPool,
        execution: WorkflowExecution,
    ) -> DocumentResult<WorkflowExecution> {
        let record = sqlx::query!(
            r#"
            INSERT INTO document_workflow_executions (
                id, document_id, workflow_id, status,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
            execution.id,
            execution.document_id,
            execution.workflow_id,
            execution.status as _,
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
            created_at: record.created_at,
            updated_at: record.updated_at,
        })
    }

    pub async fn create_execution_step(
        pool: &PgPool,
        step: ExecutionStep,
    ) -> DocumentResult<ExecutionStep> {
        let record = sqlx::query!(
            r#"
            INSERT INTO execution_steps (
                id, execution_id, step_id, status,
                result, error, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
            step.id,
            step.execution_id,
            step.step_id,
            step.status as _,
            step.result,
            step.error,
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
            error: record.error,
            created_at: record.created_at,
            updated_at: record.updated_at,
        })
    }
}
