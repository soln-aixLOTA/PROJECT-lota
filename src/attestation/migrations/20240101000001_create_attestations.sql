-- Create attestations table
CREATE TABLE IF NOT EXISTS attestations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id VARCHAR(255) NOT NULL,
    attestation_type VARCHAR(50) NOT NULL,
    attestation_data JSONB NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP WITH TIME ZONE,
    status VARCHAR(20) DEFAULT 'pending'
);

-- Create indexes
CREATE INDEX idx_attestations_client_id ON attestations(client_id);
CREATE INDEX idx_attestations_type ON attestations(attestation_type);
CREATE INDEX idx_attestations_status ON attestations(status);
CREATE INDEX idx_attestations_expires_at ON attestations(expires_at);

-- Add trigger for updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_attestations_updated_at
    BEFORE UPDATE ON attestations
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
