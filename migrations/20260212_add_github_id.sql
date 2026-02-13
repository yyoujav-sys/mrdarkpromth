-- Add github_id column to users table
ALTER TABLE users ADD COLUMN github_id VARCHAR(255);
CREATE UNIQUE INDEX idx_users_github_id ON users(github_id);
