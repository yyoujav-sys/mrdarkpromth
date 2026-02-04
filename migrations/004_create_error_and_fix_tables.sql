-- Create errors table
CREATE TABLE IF NOT EXISTS errors (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    source VARCHAR(50) NOT NULL,
    error_type VARCHAR(50) NOT NULL,
    severity VARCHAR(20) NOT NULL,
    message TEXT NOT NULL,
    file_path TEXT,
    line_number INTEGER,
    stack_trace TEXT,
    context JSONB,
    agent_id VARCHAR(255)
);

-- Create fixes table
CREATE TABLE IF NOT EXISTS fixes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    error_id UUID NOT NULL REFERENCES errors(id) ON DELETE CASCADE,
    description TEXT NOT NULL,
    code_changes JSONB NOT NULL,
    confidence_score DECIMAL(5,4) NOT NULL,
    reasoning TEXT NOT NULL,
    estimated_risk VARCHAR(20) NOT NULL,
    suggested_tests TEXT[],
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_errors_timestamp ON errors(timestamp);
CREATE INDEX IF NOT EXISTS idx_errors_error_type ON errors(error_type);
CREATE INDEX IF NOT EXISTS idx_errors_severity ON errors(severity);
CREATE INDEX IF NOT EXISTS idx_fixes_error_id ON fixes(error_id);
