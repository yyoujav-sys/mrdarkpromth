-- Fixed Migration Order: Create users table first
-- This combines the essential parts of 001 and 002 in correct order

-- Create UserTier enum type first
CREATE TYPE user_tier AS ENUM ('free', 'premium', 'ultra');

-- Users Table - MUST be created before any table that references it
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    tier user_tier NOT NULL DEFAULT 'free',
    api_key VARCHAR(255) UNIQUE NOT NULL,
    api_key_expires_at TIMESTAMP WITH TIME ZONE,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for users table
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
CREATE INDEX IF NOT EXISTS idx_users_api_key ON users(api_key);
CREATE INDEX IF NOT EXISTS idx_users_tier ON users(tier);
CREATE INDEX IF NOT EXISTS idx_users_is_active ON users(is_active);
CREATE INDEX IF NOT EXISTS idx_users_created_at ON users(created_at);

-- Constraints for users table
ALTER TABLE users ADD CONSTRAINT users_username_length CHECK (LENGTH(username) >= 3);
ALTER TABLE users ADD CONSTRAINT users_email_format CHECK (email ~* '^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$');

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Trigger for users table
CREATE TRIGGER update_users_updated_at 
    BEFORE UPDATE ON users 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Insert default admin user
INSERT INTO users (username, email, password_hash, tier, api_key)
VALUES (
    'admin',
    'admin@mrdarkpromth.ai',
    '$argon2id$v=19$m=65536,t=3,p=4$example_hash_replace_in_app',
    'ultra',
    'mr_admin_' || gen_random_uuid()
) ON CONFLICT (username) DO NOTHING;

-- Now create Ultra Tier tables (moved from 001)
CREATE TABLE IF NOT EXISTS ultra_tier_consents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID UNIQUE NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    accepted_unrestricted BOOLEAN NOT NULL,
    accepted_responsibility BOOLEAN NOT NULL,
    accepted_legal_compliance BOOLEAN NOT NULL,
    accepted_risks BOOLEAN NOT NULL,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS ultra_tier_activations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID UNIQUE NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    activated BOOLEAN NOT NULL DEFAULT true,
    activation_time TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for ultra tier tables
CREATE INDEX IF NOT EXISTS idx_ultra_tier_consents_user_id ON ultra_tier_consents(user_id);
CREATE INDEX IF NOT EXISTS idx_ultra_tier_activations_user_id ON ultra_tier_activations(user_id);

-- Triggers for ultra tier tables
CREATE TRIGGER update_ultra_tier_consents_updated_at 
    BEFORE UPDATE ON ultra_tier_consents 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_ultra_tier_activations_updated_at 
    BEFORE UPDATE ON ultra_tier_activations 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
