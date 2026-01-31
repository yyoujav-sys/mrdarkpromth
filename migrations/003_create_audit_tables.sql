-- Create audit_logs table for comprehensive audit logging
CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    action VARCHAR(50) NOT NULL,
    severity VARCHAR(20) NOT NULL,
    user_tier VARCHAR(20),
    ip_address INET,
    user_agent TEXT,
    request_id UUID,
    details JSONB NOT NULL DEFAULT '{}',
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    success BOOLEAN NOT NULL DEFAULT true,
    error_message TEXT
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_audit_logs_user_id ON audit_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_action ON audit_logs(action);
CREATE INDEX IF NOT EXISTS idx_audit_logs_severity ON audit_logs(severity);
CREATE INDEX IF NOT EXISTS idx_audit_logs_user_tier ON audit_logs(user_tier);
CREATE INDEX IF NOT EXISTS idx_audit_logs_timestamp ON audit_logs(timestamp);
CREATE INDEX IF NOT EXISTS idx_audit_logs_request_id ON audit_logs(request_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_success ON audit_logs(success);

-- Create composite indexes for common queries
CREATE INDEX IF NOT EXISTS idx_audit_logs_user_timestamp ON audit_logs(user_id, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_audit_logs_tier_timestamp ON audit_logs(user_tier, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_audit_logs_severity_timestamp ON audit_logs(severity, timestamp DESC);

-- Add table comments
COMMENT ON TABLE audit_logs IS 'Comprehensive audit logging for all system operations';
COMMENT ON COLUMN audit_logs.id IS 'Unique identifier for each audit entry';
COMMENT ON COLUMN audit_logs.user_id IS 'Reference to the user who performed the action';
COMMENT ON COLUMN audit_logs.action IS 'Type of action performed (prompt_request, jailbreak_attempt, etc.)';
COMMENT ON COLUMN audit_logs.severity IS 'Severity level (low, medium, high, critical)';
COMMENT ON COLUMN audit_logs.user_tier IS 'User tier at the time of the action';
COMMENT ON COLUMN audit_logs.ip_address IS 'IP address of the client';
COMMENT ON COLUMN audit_logs.user_agent IS 'User agent string of the client';
COMMENT ON COLUMN audit_logs.request_id IS 'Reference to the specific request being audited';
COMMENT ON COLUMN audit_logs.details IS 'JSON details specific to the action type';
COMMENT ON COLUMN audit_logs.timestamp IS 'When the action occurred';
COMMENT ON COLUMN audit_logs.success IS 'Whether the action was successful';
COMMENT ON COLUMN audit_logs.error_message IS 'Error message if the action failed';

-- Create a function to automatically clean up old audit logs
CREATE OR REPLACE FUNCTION cleanup_old_audit_logs(retention_days INTEGER DEFAULT 90)
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM audit_logs 
    WHERE timestamp < NOW() - INTERVAL '1 day' * retention_days;
    
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    
    INSERT INTO audit_logs (
        user_id, action, severity, details, timestamp, success
    ) VALUES (
        NULL, 
        'system_config_change', 
        'low', 
        jsonb_build_object('action', 'cleanup_old_audit_logs', 'deleted_count', deleted_count, 'retention_days', retention_days),
        NOW(),
        true
    );
    
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- Create a view for recent security events
CREATE OR REPLACE VIEW recent_security_events AS
SELECT 
    id,
    user_id,
    action,
    severity,
    user_tier,
    ip_address,
    details,
    timestamp,
    success,
    error_message
FROM audit_logs 
WHERE severity IN ('high', 'critical')
   OR action IN ('jailbreak_attempt', 'unauthorized_access', 'tier_upgrade')
ORDER BY timestamp DESC;

COMMENT ON VIEW recent_security_events IS 'Recent security-related events for monitoring';

-- Create a view for ultra tier activity
CREATE OR REPLACE VIEW ultra_tier_activity AS
SELECT 
    id,
    user_id,
    action,
    severity,
    ip_address,
    details,
    timestamp,
    success
FROM audit_logs 
WHERE user_tier = 'ultra'
ORDER BY timestamp DESC;

COMMENT ON VIEW ultra_tier_activity IS 'All activity from ultra tier users';

-- Create a function to get audit statistics
CREATE OR REPLACE FUNCTION get_audit_stats(
    start_time TIMESTAMP WITH TIME ZONE DEFAULT NOW() - INTERVAL '24 hours',
    end_time TIMESTAMP WITH TIME ZONE DEFAULT NOW()
)
RETURNS TABLE (
    total_logs BIGINT,
    successful_operations BIGINT,
    failed_operations BIGINT,
    jailbreak_attempts BIGINT,
    ultra_tier_operations BIGINT,
    average_processing_time DOUBLE PRECISION
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        COUNT(*) as total_logs,
        COUNT(*) FILTER (WHERE success = true) as successful_operations,
        COUNT(*) FILTER (WHERE success = false) as failed_operations,
        COUNT(*) FILTER (WHERE action = 'jailbreak_attempt') as jailbreak_attempts,
        COUNT(*) FILTER (WHERE user_tier = 'ultra') as ultra_tier_operations,
        AVG((details->>'processing_time_ms')::DOUBLE PRECISION) as average_processing_time
    FROM audit_logs 
    WHERE timestamp BETWEEN start_time AND end_time;
END;
$$ LANGUAGE plpgsql;

-- Grant necessary permissions (adjust as needed for your setup)
-- GRANT SELECT, INSERT ON audit_logs TO app_user;
-- GRANT USAGE, SELECT ON SEQUENCE audit_logs_id_seq TO app_user;
-- GRANT SELECT ON recent_security_events TO app_user;
-- GRANT SELECT ON ultra_tier_activity TO app_user;
-- GRANT EXECUTE ON FUNCTION cleanup_old_audit_logs(INTEGER) TO app_user;
-- GRANT EXECUTE ON FUNCTION get_audit_stats(TIMESTAMP WITH TIME ZONE, TIMESTAMP WITH TIME ZONE) TO app_user;
