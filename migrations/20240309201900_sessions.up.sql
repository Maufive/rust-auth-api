-- Add up migration script here
CREATE TABLE IF NOT EXISTS sessions (
    session_token BYTEA PRIMARY KEY,
    user_id UUID REFERENCES users (id) ON DELETE CASCADE
);