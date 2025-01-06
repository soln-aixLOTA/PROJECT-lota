<<<<<<< HEAD
version https://git-lfs.github.com/spec/v1
oid sha256:40c914d281ac25b45991df12a7196489c844ac095ff23a2497e585a7e42c6704
size 288
=======
-- Drop trigger first
DROP TRIGGER IF EXISTS update_users_updated_at ON users;
-- Drop function
DROP FUNCTION IF EXISTS update_updated_at_column;
-- Drop index
DROP INDEX IF EXISTS idx_users_email;
-- Drop table
DROP TABLE IF EXISTS users;
-- Drop enum type
DROP TYPE IF EXISTS user_role;
>>>>>>> 921251a (fetch)
