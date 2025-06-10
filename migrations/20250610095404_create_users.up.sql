-- Add up migration script here
CREATE TABLE
    IF NOT EXISTS users (
        id VARCHAR(255) PRIMARY KEY,
        username VARCHAR(255) NOT NULL UNIQUE,
        user_password VARCHAR(255) NOT NULL,
        created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
        archived_at TIMESTAMP NULL,
    );

CREATE INDEX IF NOT EXISTS idx_users_id ON users (id);

CREATE INDEX IF NOT EXISTS idx_users_username ON users (username);