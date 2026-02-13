-- Migration: Add client tracking columns to users table
-- Date: 2026-02-13
-- Description: Adds last_client_type and is_online columns for cross-platform session tracking

-- Forward Migration
ALTER TABLE users ADD COLUMN IF NOT EXISTS last_client_type VARCHAR(50);
ALTER TABLE users ADD COLUMN IF NOT EXISTS is_online BOOLEAN NOT NULL DEFAULT false;

-- Create index for online status queries
CREATE INDEX IF NOT EXISTS idx_users_is_online ON users(is_online);

-- Rollback SQL:
-- ALTER TABLE users DROP COLUMN IF EXISTS last_client_type;
-- ALTER TABLE users DROP COLUMN IF EXISTS is_online;
-- DROP INDEX IF EXISTS idx_users_is_online;
