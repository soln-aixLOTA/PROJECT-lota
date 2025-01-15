-- Add file_path column to documents table
ALTER TABLE documents ADD COLUMN IF NOT EXISTS file_path VARCHAR(255);

-- Update existing documents to have a default file_path
UPDATE documents SET file_path = '' WHERE file_path IS NULL; 