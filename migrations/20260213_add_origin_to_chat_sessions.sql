-- Migration: Add origin column to chat_sessions table
-- Date: 2026-02-13
-- Description: Tracks which client platform originated each chat session

-- Forward Migration
ALTER TABLE chat_sessions ADD COLUMN IF NOT EXISTS origin VARCHAR(50) DEFAULT 'website';

-- Create index for origin queries
CREATE INDEX IF NOT EXISTS idx_chat_sessions_origin ON chat_sessions(origin);

-- Rollback SQL:
-- ALTER TABLE chat_sessions DROP COLUMN IF EXISTS origin;
-- DROP INDEX IF EXISTS idx_chat_sessions_origin;
