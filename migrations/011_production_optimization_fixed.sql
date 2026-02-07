-- Fixed Production Database Optimization Script
-- Creates optimized indexes for high-performance production workloads

-- Users table optimization
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_users_tier_is_active ON users(tier, is_active);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_users_created_at_tier ON users(created_at, tier);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_users_email_active ON users(email, is_active) WHERE is_active = true;

-- Audit logs optimization for security monitoring
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_audit_logs_user_timestamp ON audit_logs(user_id, "timestamp" DESC);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_audit_logs_action_timestamp ON audit_logs(action, "timestamp" DESC);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_audit_logs_severity_timestamp ON audit_logs(severity, "timestamp" DESC);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_audit_logs_success_timestamp ON audit_logs(success, "timestamp" DESC) WHERE success = false;

-- Chat sessions and messages optimization
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_chat_sessions_user_created ON chat_sessions(user_id, created_at DESC);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_chat_messages_session_created ON chat_messages(session_id, created_at DESC);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_chat_messages_user_created ON chat_messages(user_id, created_at DESC);

-- Payment and billing optimization
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_payments_user_status_created ON payments(user_id, status, created_at DESC);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_payments_reference ON payments(reference) WHERE reference IS NOT NULL;
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_subscriptions_user_status_end ON subscriptions(user_id, status, end_date DESC);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_subscriptions_status_end_date ON subscriptions(status, end_date DESC) WHERE status = 'active';

-- Email tokens optimization for cleanup
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_email_tokens_user_expires ON email_verification_tokens(user_id, expires_at);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_password_tokens_user_expires ON password_reset_tokens(user_id, expires_at);

-- Jailbreak prompts optimization (fixed column names)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_jailbreak_prompts_category_effectiveness ON jailbreak_prompts(category, effectiveness);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_jailbreak_prompts_risk_level ON jailbreak_prompts(risk_level) WHERE risk_level IN ('high', 'critical');

-- Ultra tier optimization
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_ultra_activations_user_time ON ultra_tier_activations(user_id, activation_time DESC);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_ultra_consents_user_time ON ultra_tier_consents(user_id, "timestamp" DESC);

-- Error tracking optimization
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_errors_source_timestamp ON errors(source, "timestamp" DESC);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_errors_severity_timestamp ON errors(severity, "timestamp" DESC);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_fixes_error_id ON fixes(error_id);

-- Prompt usage optimization (fixed column names)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_prompt_usage_user_timestamp ON prompt_usage_records(user_id, used_at DESC);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_prompt_usage_model_timestamp ON prompt_usage_records(target_model, used_at DESC);

-- Update table statistics for query planner
ANALYZE users;
ANALYZE audit_logs;
ANALYZE chat_sessions;
ANALYZE chat_messages;
ANALYZE payments;
ANALYZE subscriptions;
ANALYZE email_verification_tokens;
ANALYZE password_reset_tokens;
ANALYZE jailbreak_prompts;
ANALYZE ultra_tier_activations;
ANALYZE ultra_tier_consents;
ANALYZE errors;
ANALYZE fixes;
ANALYZE prompt_usage_records;

-- Create materialized views for common queries
CREATE MATERIALIZED VIEW IF NOT EXISTS user_activity_summary AS
SELECT 
    u.id,
    u.username,
    u.email,
    u.tier,
    u.created_at as user_created_at,
    COUNT(DISTINCT cs.id) as session_count,
    COUNT(DISTINCT cm.id) as message_count,
    MAX(cm.created_at) as last_activity,
    COUNT(DISTINCT al.id) as audit_event_count
FROM users u
LEFT JOIN chat_sessions cs ON u.id = cs.user_id
LEFT JOIN chat_messages cm ON cs.id = cm.session_id
LEFT JOIN audit_logs al ON u.id = al.user_id
GROUP BY u.id, u.username, u.email, u.tier, u.created_at;

CREATE UNIQUE INDEX IF NOT EXISTS idx_user_activity_summary_id ON user_activity_summary(id);

-- Create function to refresh materialized views
CREATE OR REPLACE FUNCTION refresh_user_activity_summary()
RETURNS void AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY user_activity_summary;
END;
$$ LANGUAGE plpgsql;

-- Set up automatic cleanup for expired tokens
CREATE OR REPLACE FUNCTION cleanup_expired_tokens()
RETURNS void AS $$
BEGIN
    DELETE FROM email_verification_tokens WHERE expires_at < NOW();
    DELETE FROM password_reset_tokens WHERE expires_at < NOW();
END;
$$ LANGUAGE plpgsql;

-- Create indexes for better performance on common queries
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_users_composite_tier_active_created 
ON users(tier, is_active, created_at DESC);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_audit_logs_composite_severity_time 
ON audit_logs(severity, "timestamp" DESC) WHERE severity IN ('high', 'critical');

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_payments_composite_status_time 
ON payments(status, created_at DESC);

-- Performance monitoring query
SELECT 
    schemaname,
    tablename,
    attname,
    n_distinct,
    correlation
FROM pg_stats 
WHERE schemaname = 'public' 
    AND tablename IN ('users', 'audit_logs', 'chat_sessions', 'payments')
ORDER BY tablename, attname;
