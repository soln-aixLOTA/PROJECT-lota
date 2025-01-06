<<<<<<< HEAD
version https://git-lfs.github.com/spec/v1
oid sha256:28d21734e457bac16f3e7cdf2924c0bf078030d06cb0320a82e56d7f6a32332d
size 888
=======
CREATE TYPE document_status AS ENUM ('pending', 'processing', 'completed', 'failed');
CREATE TYPE security_level AS ENUM (
    'public',
    'internal',
    'confidential',
    'restricted'
);
CREATE TYPE document_classification AS ENUM ('contract', 'invoice', 'report', 'other');
CREATE TABLE documents (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    content_type VARCHAR(255) NOT NULL,
    size BIGINT NOT NULL,
    storage_path VARCHAR(255) NOT NULL,
    status document_status NOT NULL DEFAULT 'pending',
    author VARCHAR(255) NOT NULL,
    classification document_classification NOT NULL DEFAULT 'other',
    security_level security_level NOT NULL DEFAULT 'public',
    custom_fields JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);
>>>>>>> 921251a (fetch)
