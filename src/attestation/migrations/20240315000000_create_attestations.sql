-- Add new columns to attestations table
ALTER TABLE attestations
ADD COLUMN IF NOT EXISTS verification_status VARCHAR(20) DEFAULT 'unverified',
ADD COLUMN IF NOT EXISTS verification_data JSONB,
ADD COLUMN IF NOT EXISTS last_verified_at TIMESTAMP WITH TIME ZONE;

-- Create index for verification status
CREATE INDEX IF NOT EXISTS idx_attestations_verification_status ON attestations(verification_status);

-- Add audit columns
ALTER TABLE attestations
ADD COLUMN IF NOT EXISTS created_by VARCHAR(255),
ADD COLUMN IF NOT EXISTS updated_by VARCHAR(255);

-- Create audit trigger
CREATE OR REPLACE FUNCTION update_audit_columns()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        NEW.created_by = current_user;
    END IF;
    NEW.updated_by = current_user;
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_attestations_audit
    BEFORE INSERT OR UPDATE ON attestations
    FOR EACH ROW
    EXECUTE FUNCTION update_audit_columns();
