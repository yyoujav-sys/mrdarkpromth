-- Seed additional test users for comprehensive testing
-- All passwords are 'admin123' (hash copied from admin user)

-- Free Tier User
INSERT INTO users (username, email, password_hash, tier, api_key, is_active)
VALUES (
    'testfree',
    'testfree@mrdarkpromth.ai',
    '$argon2id$v=19$m=19456,t=2,p=1$PaqTPuSDTe9S6OsT+l7+zw$ScE7QbLywnTOe59cF0csKTchlCBKY8DIneQwNAFcbdg',
    'free',
    'mk_testfree_' || gen_random_uuid(),
    true
) ON CONFLICT (username) DO NOTHING;

-- Premium Tier User
INSERT INTO users (username, email, password_hash, tier, api_key, is_active)
VALUES (
    'testpremium',
    'testpremium@mrdarkpromth.ai',
    '$argon2id$v=19$m=19456,t=2,p=1$PaqTPuSDTe9S6OsT+l7+zw$ScE7QbLywnTOe59cF0csKTchlCBKY8DIneQwNAFcbdg',
    'premium',
    'mk_testpremium_' || gen_random_uuid(),
    true
) ON CONFLICT (username) DO NOTHING;

-- Ultra Tier User (Ensure it exists and has known password)
INSERT INTO users (username, email, password_hash, tier, api_key, is_active)
VALUES (
    'testultra',
    'testultra@mrdarkpromth.ai',
    '$argon2id$v=19$m=19456,t=2,p=1$PaqTPuSDTe9S6OsT+l7+zw$ScE7QbLywnTOe59cF0csKTchlCBKY8DIneQwNAFcbdg',
    'ultra',
    'mk_testultra_' || gen_random_uuid(),
    true
) ON CONFLICT (username) DO UPDATE SET
    password_hash = EXCLUDED.password_hash,
    tier = 'ultra';
