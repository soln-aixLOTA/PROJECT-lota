<<<<<<< HEAD
version https://git-lfs.github.com/spec/v1
oid sha256:48853b4e321792825c3321867e63d6876f5b23580496a3b2201bdb88f9dfdd96
size 1174
=======
-- Create attestations table
CREATE TABLE IF NOT EXISTS attestations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    model_id UUID NOT NULL,
    version_id UUID NOT NULL,
    attestation_type VARCHAR(255) NOT NULL,
    attestation_data JSONB NOT NULL,
    signature TEXT NOT NULL,
    public_key TEXT NOT NULL,
    metadata JSONB,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    revoked_at TIMESTAMPTZ,
    revocation_reason TEXT
);
-- Create indexes
CREATE INDEX idx_attestations_model_id ON attestations(model_id);
CREATE INDEX idx_attestations_version_id ON attestations(version_id);
CREATE INDEX idx_attestations_status ON attestations(status);
CREATE INDEX idx_attestations_created_at ON attestations(created_at);
-- Add trigger for updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column() RETURNS TRIGGER AS $$ BEGIN NEW.updated_at = NOW();
RETURN NEW;
END;
$$ language 'plpgsql';
CREATE TRIGGER update_attestations_updated_at BEFORE
UPDATE ON attestations FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
>>>>>>> 921251a (fetch)
