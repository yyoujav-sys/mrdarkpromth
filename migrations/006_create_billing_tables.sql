-- Billing & Payment Tables
-- Migration: 006_create_billing_tables.sql

-- Create user_tier type if not exists
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'user_tier') THEN
        CREATE TYPE user_tier AS ENUM ('free', 'premium', 'ultra');
    END IF;
END$$;

-- Plans table
CREATE TABLE IF NOT EXISTS plans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    tier VARCHAR(20) NOT NULL DEFAULT 'free',
    price DECIMAL(10, 2) NOT NULL,
    duration_days INTEGER NOT NULL DEFAULT 30,
    features JSONB NOT NULL DEFAULT '[]',
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Payments table
CREATE TABLE IF NOT EXISTS payments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    plan_id UUID NOT NULL REFERENCES plans(id),
    amount DECIMAL(10, 2) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    payment_method VARCHAR(20) NOT NULL,
    reference VARCHAR(255) UNIQUE,
    qr_code_data TEXT,
    slip_image_path VARCHAR(500),
    verified_by UUID REFERENCES users(id),
    verified_at TIMESTAMP WITH TIME ZONE,
    expires_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Subscriptions table
CREATE TABLE IF NOT EXISTS subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    plan_id UUID NOT NULL REFERENCES plans(id),
    payment_id UUID REFERENCES payments(id),
    tier VARCHAR(20) NOT NULL DEFAULT 'free',
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    start_date TIMESTAMP WITH TIME ZONE NOT NULL,
    end_date TIMESTAMP WITH TIME ZONE NOT NULL,
    auto_renew BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Payment slip verification queue
CREATE TABLE IF NOT EXISTS payment_slip_verifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    payment_id UUID NOT NULL REFERENCES payments(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id),
    slip_image_path VARCHAR(500) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    submitted_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    reviewed_by UUID REFERENCES users(id),
    reviewed_at TIMESTAMP WITH TIME ZONE,
    review_notes TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_payments_user_id ON payments(user_id);
CREATE INDEX idx_payments_status ON payments(status);
CREATE INDEX idx_payments_reference ON payments(reference);
CREATE INDEX idx_subscriptions_user_id ON subscriptions(user_id);
CREATE INDEX idx_subscriptions_status ON subscriptions(status);
CREATE INDEX idx_subscriptions_end_date ON subscriptions(end_date);
CREATE INDEX idx_slip_verifications_status ON payment_slip_verifications(status);

-- Insert default plans
INSERT INTO plans (name, tier, price, duration_days, features) VALUES
('Free', 'free', 0.00, 36500, '["basic_chat", "limited_requests"]'),
('Premium', 'premium', 299.00, 30, '["unlimited_chat", "jailbreak_prompts", "basic_tools", "priority_support"]'),
('Ultra', 'ultra', 999.00, 30, '["everything_in_premium", "multi_agent", "sandbox_execution", "terminal_access", "unrestricted_mode", "code_generation"]')
ON CONFLICT DO NOTHING;
