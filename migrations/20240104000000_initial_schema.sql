<<<<<<< HEAD
version https://git-lfs.github.com/spec/v1
oid sha256:de310b06d659e1c23f6afd3db90628dacfa5b9105e89eae339ddba0c3ac8a458
size 5220
=======
-- Create custom types
CREATE TYPE document_status AS ENUM (
    'pending',
    'processing',
    'completed',
    'failed',
    'archived'
);
CREATE TYPE document_classification AS ENUM ('legal', 'financial', 'medical', 'hr', 'general');
CREATE TYPE security_level AS ENUM (
    'public',
    'internal',
    'confidential',
    'restricted'
);
CREATE TYPE workflow_status AS ENUM (
    'draft',
    'active',
    'paused',
    'completed',
    'failed',
    'archived'
);
CREATE TYPE step_status AS ENUM (
    'pending',
    'in_progress',
    'completed',
    'failed',
    'skipped'
);
CREATE TYPE step_type AS ENUM (
    'ocr',
    'classification',
    'data_extraction',
    'validation',
    'approval',
    'notification',
    'storage',
    'custom'
);
-- Create documents table
CREATE TABLE documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    content_type VARCHAR(255) NOT NULL,
    size BIGINT NOT NULL,
    storage_path TEXT NOT NULL,
    status document_status NOT NULL DEFAULT 'pending',
    classification document_classification,
    security_level security_level NOT NULL DEFAULT 'internal',
    author VARCHAR(255) NOT NULL,
    tags TEXT [] DEFAULT ARRAY []::TEXT [],
    custom_fields JSONB DEFAULT '{}'::JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
-- Create workflows table
CREATE TABLE workflows (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    status workflow_status NOT NULL DEFAULT 'draft',
    creator VARCHAR(255) NOT NULL,
    tags TEXT [] DEFAULT ARRAY []::TEXT [],
    document_types TEXT [] DEFAULT ARRAY []::TEXT [],
    required_approvals TEXT [] DEFAULT ARRAY []::TEXT [],
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
-- Create workflow steps table
CREATE TABLE workflow_steps (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workflow_id UUID NOT NULL REFERENCES workflows(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    step_type step_type NOT NULL,
    config JSONB DEFAULT '{}'::JSONB,
    "order" INTEGER NOT NULL,
    status step_status NOT NULL DEFAULT 'pending',
    result JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
-- Create document workflow executions table
CREATE TABLE document_workflow_executions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    workflow_id UUID NOT NULL REFERENCES workflows(id),
    status workflow_status NOT NULL DEFAULT 'pending',
    started_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMPTZ,
    error_message TEXT,
    metadata JSONB DEFAULT '{}'::JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
-- Create execution steps table
CREATE TABLE execution_steps (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    execution_id UUID NOT NULL REFERENCES document_workflow_executions(id) ON DELETE CASCADE,
    workflow_step_id UUID NOT NULL REFERENCES workflow_steps(id),
    status step_status NOT NULL DEFAULT 'pending',
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    result JSONB,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
-- Create indexes
CREATE INDEX idx_documents_status ON documents(status);
CREATE INDEX idx_documents_classification ON documents(classification);
CREATE INDEX idx_documents_security_level ON documents(security_level);
CREATE INDEX idx_workflows_status ON workflows(status);
CREATE INDEX idx_workflow_steps_workflow_id ON workflow_steps(workflow_id);
CREATE INDEX idx_workflow_steps_order ON workflow_steps(workflow_id, "order");
CREATE INDEX idx_document_workflow_executions_document_id ON document_workflow_executions(document_id);
CREATE INDEX idx_document_workflow_executions_workflow_id ON document_workflow_executions(workflow_id);
CREATE INDEX idx_execution_steps_execution_id ON execution_steps(execution_id);
-- Create updated_at triggers
CREATE OR REPLACE FUNCTION update_updated_at_column() RETURNS TRIGGER AS $$ BEGIN NEW.updated_at = CURRENT_TIMESTAMP;
RETURN NEW;
END;
$$ language 'plpgsql';
CREATE TRIGGER update_documents_updated_at BEFORE
UPDATE ON documents FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_workflows_updated_at BEFORE
UPDATE ON workflows FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_workflow_steps_updated_at BEFORE
UPDATE ON workflow_steps FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_document_workflow_executions_updated_at BEFORE
UPDATE ON document_workflow_executions FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_execution_steps_updated_at BEFORE
UPDATE ON execution_steps FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
>>>>>>> 921251a (fetch)
