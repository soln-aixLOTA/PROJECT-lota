-- Create attestations table
CREATE TABLE IF NOT EXISTS attestations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    version_id UUID NOT NULL,
    agent_id UUID NOT NULL,
    attestation_type VARCHAR(50) NOT NULL,
    attestation_data JSONB NOT NULL,
    input_hash VARCHAR(64) NOT NULL,
    output_hash VARCHAR(64) NOT NULL,
    signature TEXT NOT NULL,
    public_key TEXT NOT NULL,
    confidence_score DECIMAL(5, 2),
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    verified_at TIMESTAMP WITH TIME ZONE
);
-- Create indexes
CREATE INDEX IF NOT EXISTS idx_attestations_version_id ON attestations(version_id);
CREATE INDEX IF NOT EXISTS idx_attestations_agent_id ON attestations(agent_id);
CREATE INDEX IF NOT EXISTS idx_attestations_status ON attestations(status);
CREATE INDEX IF NOT EXISTS idx_attestations_created_at ON attestations(created_at);
-- Create trigger for updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column() RETURNS TRIGGER AS $$ BEGIN NEW.updated_at = NOW();
RETURN NEW;
END;
$$ language 'plpgsql';
CREATE TRIGGER update_attestations_updated_at BEFORE
UPDATE ON attestations FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();