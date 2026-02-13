-- Migration: Create system_broadcast_events table
-- Date: 2026-02-13
-- Description: Stores system-wide events for broadcasting to clients (e.g., tier upgrades, maintenance)

CREATE TABLE IF NOT EXISTS system_broadcast_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type VARCHAR(100) NOT NULL, -- 'tier_updated', 'system_maintenance', 'announcement'
    payload JSONB NOT NULL DEFAULT '{}',
    target_user_id UUID REFERENCES users(id) ON DELETE SET NULL, -- NULL = broadcast to all
    is_read BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_system_broadcast_events_event_type ON system_broadcast_events(event_type);
CREATE INDEX IF NOT EXISTS idx_system_broadcast_events_target_user_id ON system_broadcast_events(target_user_id);
CREATE INDEX IF NOT EXISTS idx_system_broadcast_events_created_at ON system_broadcast_events(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_system_broadcast_events_is_read ON system_broadcast_events(is_read);

-- Rollback SQL:
-- DROP TABLE IF EXISTS system_broadcast_events;
