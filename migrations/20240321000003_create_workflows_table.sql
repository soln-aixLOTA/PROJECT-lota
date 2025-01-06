<<<<<<< HEAD
version https://git-lfs.github.com/spec/v1
oid sha256:fa8223dd86988018a441b4aa1b312beb0cb3c2ce7713666a2b621ebef7bca9ef
size 1275
=======
CREATE TYPE workflow_status AS ENUM ('pending', 'in_progress', 'completed', 'failed');
CREATE TYPE step_status AS ENUM (
    'pending',
    'in_progress',
    'completed',
    'failed',
    'skipped'
);
CREATE TYPE step_type AS ENUM (
    'approval',
    'ocr',
    'classification',
    'validation',
    'notification'
);
CREATE TABLE workflows (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    status workflow_status NOT NULL DEFAULT 'pending',
    creator VARCHAR(255) NOT NULL,
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE TABLE workflow_steps (
    id UUID PRIMARY KEY,
    workflow_id UUID NOT NULL REFERENCES workflows(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    step_type step_type NOT NULL,
    status step_status NOT NULL DEFAULT 'pending',
    config JSONB NOT NULL DEFAULT '{}',
    result JSONB,
    "order" INTEGER NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);
>>>>>>> 921251a (fetch)
