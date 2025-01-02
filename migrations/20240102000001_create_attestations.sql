CREATE TABLE IF NOT EXISTS attestations (
    id UUID PRIMARY KEY,
    agent_id UUID NOT NULL,
    model_id TEXT NOT NULL,
    input_hash TEXT NOT NULL,
    output_hash TEXT NOT NULL,
    confidence_score DOUBLE PRECISION NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL,
    verified_at TIMESTAMPTZ
);