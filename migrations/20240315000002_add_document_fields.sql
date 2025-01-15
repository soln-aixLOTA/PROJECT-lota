-- Add missing document fields
ALTER TABLE documents 
    ADD COLUMN IF NOT EXISTS content_type VARCHAR(255),
    ADD COLUMN IF NOT EXISTS size BIGINT,
    ADD COLUMN IF NOT EXISTS metadata JSONB; 