<<<<<<< HEAD
version https://git-lfs.github.com/spec/v1
oid sha256:0a14804e2135d4603bed5ae04e809ac8abbb907cb69b6aedeb046d82cd3a2360
size 908
=======
-- Drop triggers
DROP TRIGGER IF EXISTS update_execution_steps_updated_at ON execution_steps;
DROP TRIGGER IF EXISTS update_document_workflow_executions_updated_at ON document_workflow_executions;
DROP TRIGGER IF EXISTS update_workflow_steps_updated_at ON workflow_steps;
DROP TRIGGER IF EXISTS update_workflows_updated_at ON workflows;
DROP TRIGGER IF EXISTS update_documents_updated_at ON documents;
-- Drop function
DROP FUNCTION IF EXISTS update_updated_at_column();
-- Drop tables
DROP TABLE IF EXISTS execution_steps;
DROP TABLE IF EXISTS document_workflow_executions;
DROP TABLE IF EXISTS workflow_steps;
DROP TABLE IF EXISTS workflows;
DROP TABLE IF EXISTS documents;
-- Drop types
DROP TYPE IF EXISTS step_type;
DROP TYPE IF EXISTS step_status;
DROP TYPE IF EXISTS workflow_status;
DROP TYPE IF EXISTS security_level;
DROP TYPE IF EXISTS document_classification;
DROP TYPE IF EXISTS document_status;
>>>>>>> 921251a (fetch)
