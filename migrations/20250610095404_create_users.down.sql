-- Add down migration script here
DROP INDEX IF EXISTS idx_users_id;

DROP TABLE IF EXISTS users;