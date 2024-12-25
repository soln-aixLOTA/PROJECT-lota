-- Drop indexes first
DROP INDEX IF EXISTS idx_tenants_status;
DROP INDEX IF EXISTS idx_tenants_slug;

-- Drop the table
DROP TABLE IF EXISTS tenants; 