-- Migration: Create tables for Learning System
-- This migration sets up the required tables for the agent learning and self-correction system.

CREATE TABLE IF NOT EXISTS correction_history (
    id UUID PRIMARY KEY,
    error_id UUID NOT NULL,
    fix_id UUID NOT NULL,
    error_category VARCHAR(100) NOT NULL,
    error_message TEXT NOT NULL,
    fix_description TEXT NOT NULL,
    confidence_score FLOAT NOT NULL,
    risk_level VARCHAR(50) NOT NULL,
    validation_score FLOAT NOT NULL,
    outcome VARCHAR(50) NOT NULL,
    lessons_learned TEXT[],
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS error_patterns (
    id UUID PRIMARY KEY,
    pattern VARCHAR(500) NOT NULL,
    category VARCHAR(100) NOT NULL,
    frequency INTEGER NOT NULL DEFAULT 1,
    last_updated TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    success_rate FLOAT NOT NULL DEFAULT 0.0,
    recommended_approach TEXT
);

CREATE INDEX IF NOT EXISTS idx_correction_history_error_id ON correction_history(error_id);
CREATE INDEX IF NOT EXISTS idx_correction_history_timestamp ON correction_history(timestamp);
CREATE INDEX IF NOT EXISTS idx_error_patterns_category ON error_patterns(category);
CREATE INDEX IF NOT EXISTS idx_error_patterns_frequency ON error_patterns(frequency);
