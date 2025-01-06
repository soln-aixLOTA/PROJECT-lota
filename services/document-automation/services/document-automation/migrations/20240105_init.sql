<<<<<<< HEAD
version https://git-lfs.github.com/spec/v1
oid sha256:7c989f87abcfc40379ef1e63e3262d4f447a472da5d0723137ba637aa778b6bc
size 3092
=======
-- Create custom types
CREATE TYPE document_status AS ENUM ('pending', 'processing', 'completed', 'failed');
CREATE TYPE document_classification AS ENUM ('public', 'internal', 'confidential', 'restricted');
CREATE TYPE security_level AS ENUM ('low', 'medium', 'high', 'critical');
CREATE TYPE workflow_status AS ENUM ('draft', 'active', 'archived', 'deleted');
CREATE TYPE step_type AS ENUM ('extract', 'transform', 'load', 'validate', 'notify');
CREATE TYPE step_status AS ENUM ('pending', 'in_progress', 'completed', 'failed', 'skipped');
-- Create documents table
CREATE TABLE documents (id UUID PRIMARY KEY, name VARCHAR(255) NOT NULL, content_type VARCHAR(255) NOT NULL, size BIGINT NOT NULL, storage_path VARCHAR(255) NOT NULL, status document_status NOT NULL, metadata JSONB NOT NULL, created_by VARCHAR(255) NOT NULL, created_at TIMESTAMP WITH TIME ZONE NOT NULL, updated_at TIMESTAMP WITH TIME ZONE NOT NULL);
-- Create workflows table
CREATE TABLE workflows (id UUID PRIMARY KEY, name VARCHAR(255) NOT NULL, description TEXT NOT NULL, status workflow_status NOT NULL, creator VARCHAR(255) NOT NULL, metadata JSONB NOT NULL, created_at TIMESTAMP WITH TIME ZONE NOT NULL, updated_at TIMESTAMP WITH TIME ZONE NOT NULL);
-- Create workflow steps table
CREATE TABLE workflow_steps (id UUID PRIMARY KEY, workflow_id UUID NOT NULL REFERENCES workflows(id) ON DELETE CASCADE, name VARCHAR(255) NOT NULL, step_type step_type NOT NULL, status step_status NOT NULL, "order" INTEGER NOT NULL, config JSONB NOT NULL, created_at TIMESTAMP WITH TIME ZONE NOT NULL, updated_at TIMESTAMP WITH TIME ZONE NOT NULL);
-- Create document workflow executions table
CREATE TABLE document_workflow_executions (id UUID PRIMARY KEY, document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE, workflow_id UUID NOT NULL REFERENCES workflows(id) ON DELETE CASCADE, status step_status NOT NULL, created_at TIMESTAMP WITH TIME ZONE NOT NULL, updated_at TIMESTAMP WITH TIME ZONE NOT NULL);
-- Create execution steps table
CREATE TABLE execution_steps (id UUID PRIMARY KEY, execution_id UUID NOT NULL REFERENCES document_workflow_executions(id) ON DELETE CASCADE, step_id UUID NOT NULL REFERENCES workflow_steps(id) ON DELETE CASCADE, status step_status NOT NULL, result JSONB, error TEXT, created_at TIMESTAMP WITH TIME ZONE NOT NULL, updated_at TIMESTAMP WITH TIME ZONE NOT NULL);
-- Create indexes
CREATE INDEX idx_documents_status ON documents(status); CREATE INDEX idx_documents_created_at ON documents(created_at); CREATE INDEX idx_workflows_status ON workflows(status); CREATE INDEX idx_workflows_created_at ON workflows(created_at); CREATE INDEX idx_workflow_steps_workflow_id ON workflow_steps(workflow_id); CREATE INDEX idx_workflow_steps_order ON workflow_steps("order"); CREATE INDEX idx_executions_document_id ON document_workflow_executions(document_id); CREATE INDEX idx_executions_workflow_id ON document_workflow_executions(workflow_id); CREATE INDEX idx_execution_steps_execution_id ON execution_steps(execution_id); CREATE INDEX idx_execution_steps_step_id ON execution_steps(step_id);
>>>>>>> 921251a (fetch)
