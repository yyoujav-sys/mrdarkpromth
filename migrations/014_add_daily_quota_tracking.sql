-- Add daily message quota tracking table
CREATE TABLE IF NOT EXISTS user_daily_quota (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    date DATE NOT NULL DEFAULT CURRENT_DATE,
    messages_used INTEGER NOT NULL DEFAULT 0,
    tier VARCHAR(20) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(user_id, date)
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_user_daily_quota_user_id ON user_daily_quota(user_id);
CREATE INDEX IF NOT EXISTS idx_user_daily_quota_date ON user_daily_quota(date);
CREATE INDEX IF NOT EXISTS idx_user_daily_quota_user_date ON user_daily_quota(user_id, date);

-- Function to update daily quota
CREATE OR REPLACE FUNCTION update_daily_quota(
    p_user_id UUID,
    p_tier VARCHAR(20)
) RETURNS BOOLEAN AS $$
DECLARE
    current_count INTEGER;
    max_messages INTEGER;
BEGIN
    -- Get tier limits
    max_messages := CASE p_tier
        WHEN 'free' THEN 10
        WHEN 'premium' THEN 100
        WHEN 'ultra' THEN 1000
        WHEN 'admin' THEN 999999
        ELSE 10
    END;
    
    -- Insert or update daily quota record
    INSERT INTO user_daily_quota (user_id, date, messages_used, tier)
    VALUES (p_user_id, CURRENT_DATE, 1, p_tier)
    ON CONFLICT (user_id, date) 
    DO UPDATE SET 
        messages_used = user_daily_quota.messages_used + 1,
        updated_at = NOW(),
        tier = p_tier
    RETURNING messages_used INTO current_count;
    
    -- Return true if under quota, false if exceeded
    RETURN current_count <= max_messages;
END;
$$ LANGUAGE plpgsql;

-- Function to get current daily usage
CREATE OR REPLACE FUNCTION get_daily_usage(
    p_user_id UUID
) RETURNS INTEGER AS $$
BEGIN
    RETURN COALESCE(
        (SELECT messages_used FROM user_daily_quota 
         WHERE user_id = p_user_id AND date = CURRENT_DATE),
        0
    );
END;
$$ LANGUAGE plpgsql;

-- Function to check if user has quota available
CREATE OR REPLACE FUNCTION has_quota_available(
    p_user_id UUID,
    p_tier VARCHAR(20)
) RETURNS BOOLEAN AS $$
DECLARE
    current_count INTEGER;
    max_messages INTEGER;
BEGIN
    -- Get tier limits
    max_messages := CASE p_tier
        WHEN 'free' THEN 10
        WHEN 'premium' THEN 100
        WHEN 'ultra' THEN 1000
        WHEN 'admin' THEN 999999
        ELSE 10
    END;
    
    -- Get current usage
    current_count := COALESCE(
        (SELECT messages_used FROM user_daily_quota 
         WHERE user_id = p_user_id AND date = CURRENT_DATE),
        0
    );
    
    RETURN current_count < max_messages;
END;
$$ LANGUAGE plpgsql;

-- Trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_user_daily_quota_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER user_daily_quota_updated_at
    BEFORE UPDATE ON user_daily_quota
    FOR EACH ROW
    EXECUTE FUNCTION update_user_daily_quota_updated_at();

-- Add quota info to users table for quick access
ALTER TABLE users ADD COLUMN IF NOT EXISTS daily_quota_limit INTEGER DEFAULT 10;
ALTER TABLE users ADD COLUMN IF NOT EXISTS daily_messages_used INTEGER DEFAULT 0;
ALTER TABLE users ADD COLUMN IF NOT EXISTS last_quota_reset DATE DEFAULT CURRENT_DATE;

-- Update function to sync user table with daily quota
CREATE OR REPLACE FUNCTION sync_user_quota(p_user_id UUID) RETURNS VOID AS $$
BEGIN
    UPDATE users SET
        daily_messages_used = COALESCE(
            (SELECT messages_used FROM user_daily_quota 
             WHERE user_id = p_user_id AND date = CURRENT_DATE),
            0
        ),
        last_quota_reset = CURRENT_DATE
    WHERE id = p_user_id;
END;
$$ LANGUAGE plpgsql;
