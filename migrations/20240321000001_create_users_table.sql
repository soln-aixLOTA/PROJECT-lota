<<<<<<< HEAD
version https://git-lfs.github.com/spec/v1
oid sha256:9ead5f7b3ed7e838a88b62dca2478576331e66e0e7b31a991c2c6723d5c502bc
size 980
=======
-- Create user_role enum type
CREATE TYPE user_role AS ENUM ('admin', 'user', 'service');
-- Create users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    role user_role NOT NULL DEFAULT 'user',
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_login TIMESTAMPTZ,
    is_active BOOLEAN NOT NULL DEFAULT true
);
-- Create index on email for faster lookups
CREATE INDEX idx_users_email ON users(email);
-- Create function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column() RETURNS TRIGGER AS $$ BEGIN NEW.updated_at = CURRENT_TIMESTAMP;
RETURN NEW;
END;
$$ language 'plpgsql';
-- Create trigger to automatically update updated_at
CREATE TRIGGER update_users_updated_at BEFORE
UPDATE ON users FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
>>>>>>> 921251a (fetch)
