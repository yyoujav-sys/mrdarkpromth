-- =============================================================================
-- MR.DarkPromth — Pre-Launch Test Data Wipe
-- Truncates all transactional/test data while preserving the 'admin' account
-- and system reference data (plans, jailbreak_prompts).
-- =============================================================================

BEGIN;

-- ── Chat data (child tables first due to FK constraints) ────────────────────
TRUNCATE TABLE chat_attachments   CASCADE;
TRUNCATE TABLE chat_messages      CASCADE;
TRUNCATE TABLE chat_sessions      CASCADE;

-- ── Billing / Payment data ──────────────────────────────────────────────────
TRUNCATE TABLE payment_slip_verifications CASCADE;
TRUNCATE TABLE subscriptions              CASCADE;
TRUNCATE TABLE payments                   CASCADE;

-- ── Operational logs ────────────────────────────────────────────────────────
TRUNCATE TABLE audit_logs          CASCADE;
TRUNCATE TABLE correction_history  CASCADE;
TRUNCATE TABLE prompt_usage_records CASCADE;
TRUNCATE TABLE user_daily_quota    CASCADE;

-- ── Error tracking ──────────────────────────────────────────────────────────
TRUNCATE TABLE fixes  CASCADE;
TRUNCATE TABLE errors CASCADE;

-- ── Token tables ────────────────────────────────────────────────────────────
TRUNCATE TABLE email_verification_tokens CASCADE;
TRUNCATE TABLE password_reset_tokens     CASCADE;

-- ── Ultra tier data ─────────────────────────────────────────────────────────
TRUNCATE TABLE ultra_tier_activations CASCADE;
TRUNCATE TABLE ultra_tier_consents    CASCADE;

-- ── Remove all users EXCEPT admin ───────────────────────────────────────────
DELETE FROM users WHERE username != 'admin';

COMMIT;

-- Verify what remains
SELECT 'Remaining users:' AS info, count(*) AS cnt FROM users
UNION ALL
SELECT 'Remaining plans:',  count(*) FROM plans
UNION ALL
SELECT 'Remaining prompts:', count(*) FROM jailbreak_prompts;
