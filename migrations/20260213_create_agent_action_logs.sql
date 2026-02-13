-- Migration: Create agent_action_logs table
-- Date: 2026-02-13
-- Description: Logs all Agent actions (thoughts, tool usage, file operations, terminal commands) for monitoring and auditing

CREATE TABLE IF NOT EXISTS agent_action_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    session_id UUID REFERENCES chat_sessions(id) ON DELETE SET NULL,
    action_type VARCHAR(50) NOT NULL, -- 'thought', 'plan', 'tool_use', 'file_op', 'terminal_cmd'
    content TEXT,
    metadata JSONB DEFAULT '{}',
    client_origin VARCHAR(50), -- 'website', 'vscode_extension'
    duration_ms INTEGER,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_agent_action_logs_user_id ON agent_action_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_agent_action_logs_session_id ON agent_action_logs(session_id);
CREATE INDEX IF NOT EXISTS idx_agent_action_logs_action_type ON agent_action_logs(action_type);
CREATE INDEX IF NOT EXISTS idx_agent_action_logs_created_at ON agent_action_logs(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_agent_action_logs_user_action ON agent_action_logs(user_id, action_type);

-- Rollback SQL:
-- DROP TABLE IF EXISTS agent_action_logs;
