-- Migration: Create correction_history table for Agent 8 Self-Correction Engine
-- This table stores the history of all corrections made by the self-correction engine

CREATE TABLE IF NOT EXISTS correction_history (
    id UUID PRIMARY KEY,
    error_id UUID NOT NULL,
    fix_id UUID NOT NULL,
    error_category VARCHAR(50) NOT NULL,
    error_message TEXT NOT NULL,
    fix_description TEXT NOT NULL,
    confidence_score DECIMAL(5,4) NOT NULL,
    risk_level VARCHAR(20) NOT NULL,
    validation_score DECIMAL(5,4) NOT NULL,
    outcome VARCHAR(20) NOT NULL,
    lessons_learned TEXT[] NOT NULL,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    
    -- Indexes for efficient queries
    CONSTRAINT fk_error FOREIGN KEY (error_id) REFERENCES errors(id) ON DELETE CASCADE,
    CONSTRAINT fk_fix FOREIGN KEY (fix_id) REFERENCES fixes(id) ON DELETE CASCADE
);

-- Create indexes for common query patterns
CREATE INDEX idx_correction_history_error_category ON correction_history(error_category);
CREATE INDEX idx_correction_history_outcome ON correction_history(outcome);
CREATE INDEX idx_correction_history_timestamp ON correction_history(timestamp DESC);
CREATE INDEX idx_correction_history_confidence ON correction_history(confidence_score);
CREATE INDEX idx_correction_history_validation ON correction_history(validation_score);

-- Create a composite index for error analysis
CREATE INDEX idx_correction_history_category_outcome ON correction_history(error_category, outcome);

-- Add comments
COMMENT ON TABLE correction_history IS 'Stores history of all corrections made by the self-correction engine';
COMMENT ON COLUMN correction_history.id IS 'Unique identifier for the correction record';
COMMENT ON COLUMN correction_history.error_id IS 'Reference to the error that was corrected';
COMMENT ON COLUMN correction_history.fix_id IS 'Reference to the fix that was applied';
COMMENT ON COLUMN correction_history.error_category IS 'Category of the error (e.g., SyntaxError, RuntimeError)';
COMMENT ON COLUMN correction_history.error_message IS 'Original error message';
COMMENT ON COLUMN correction_history.fix_description IS 'Description of the fix that was applied';
COMMENT ON COLUMN correction_history.confidence_score IS 'Confidence score of the fix (0.0 to 1.0)';
COMMENT ON COLUMN correction_history.risk_level IS 'Estimated risk level of the fix (Low, Medium, High, Critical)';
COMMENT ON COLUMN correction_history.validation_score IS 'Validation score from automated tests (0.0 to 1.0)';
COMMENT ON COLUMN correction_history.outcome IS 'Final outcome of the correction (Success, PartialSuccess, Failure, RolledBack)';
COMMENT ON COLUMN correction_history.lessons_learned IS 'Array of lessons learned from this correction';
COMMENT ON COLUMN correction_history.timestamp IS 'When the correction was attempted';

-- Create a view for correction statistics
CREATE OR REPLACE VIEW correction_statistics AS
SELECT 
    error_category,
    outcome,
    COUNT(*) as count,
    AVG(confidence_score) as avg_confidence,
    AVG(validation_score) as avg_validation,
    ROUND((COUNT(*) FILTER (WHERE outcome = 'Success') * 100.0 / COUNT(*)), 2) as success_rate
FROM correction_history
GROUP BY error_category, outcome
ORDER BY error_category, outcome;

COMMENT ON VIEW correction_statistics IS 'Aggregated statistics about corrections by category and outcome';
