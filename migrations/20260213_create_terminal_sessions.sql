-- Migration: Create terminal_sessions table
-- Date: 2026-02-13
-- Description: Tracks terminal session lifecycle for Ultra Tier users

CREATE TABLE IF NOT EXISTS terminal_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    session_id VARCHAR(255) NOT NULL,
    client_type VARCHAR(50) NOT NULL DEFAULT 'website', -- 'website', 'vscode_extension'
    status VARCHAR(20) NOT NULL DEFAULT 'active', -- 'active', 'closed', 'timeout'
    command TEXT,
    exit_code INTEGER,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    closed_at TIMESTAMP WITH TIME ZONE
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_terminal_sessions_user_id ON terminal_sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_terminal_sessions_status ON terminal_sessions(status);
CREATE INDEX IF NOT EXISTS idx_terminal_sessions_session_id ON terminal_sessions(session_id);
CREATE INDEX IF NOT EXISTS idx_terminal_sessions_created_at ON terminal_sessions(created_at DESC);

-- Rollback SQL:
-- DROP TABLE IF EXISTS terminal_sessions;
