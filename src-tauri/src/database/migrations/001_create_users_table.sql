-- Create users table
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    email TEXT UNIQUE,
    password_hash TEXT NOT NULL,
    created_at TEXT NOT NULL, -- ISO 8601 datetime string
    last_login TEXT -- ISO 8601 datetime string, nullable
);

-- Create index for username lookup
CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);