-- Migration: Create user_active_sessions table
-- Date: 2026-02-13
-- Description: Tracks active user sessions across all platforms (Website, VS Code Extension)

CREATE TABLE IF NOT EXISTS user_active_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    client_type VARCHAR(50) NOT NULL, -- 'website', 'vscode_extension'
    client_version VARCHAR(50),
    ip_address INET,
    socket_id VARCHAR(255),
    last_activity TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_user_active_sessions_user_id ON user_active_sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_user_active_sessions_socket_id ON user_active_sessions(socket_id);
CREATE INDEX IF NOT EXISTS idx_user_active_sessions_last_activity ON user_active_sessions(last_activity);

-- Rollback SQL:
-- DROP TABLE IF EXISTS user_active_sessions;
