CREATE TABLE IF NOT EXISTS inference_records (
    id UUID PRIMARY KEY,
    agent_id UUID NOT NULL,
    model_id TEXT NOT NULL,
    input TEXT NOT NULL,
    output TEXT NOT NULL,
    confidence_score DOUBLE PRECISION NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL
);