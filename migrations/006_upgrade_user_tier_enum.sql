-- Upgrade user_tier enum values for legacy installations
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_type WHERE typname = 'user_tier') THEN
        -- Add premium if missing
        IF NOT EXISTS (
            SELECT 1 FROM pg_enum
            JOIN pg_type t ON pg_enum.enumtypid = t.oid
            WHERE t.typname = 'user_tier' AND enumlabel = 'premium'
        ) THEN
            ALTER TYPE user_tier ADD VALUE 'premium';
        END IF;

        -- Ensure free exists (legacy installs may only have basic)
        IF NOT EXISTS (
            SELECT 1 FROM pg_enum
            JOIN pg_type t ON pg_enum.enumtypid = t.oid
            WHERE t.typname = 'user_tier' AND enumlabel = 'free'
        ) THEN
            ALTER TYPE user_tier ADD VALUE 'free';
        END IF;

        -- Map legacy basic tier to free when present
        IF EXISTS (
            SELECT 1 FROM pg_enum
            JOIN pg_type t ON pg_enum.enumtypid = t.oid
            WHERE t.typname = 'user_tier' AND enumlabel = 'basic'
        ) THEN
            IF EXISTS (
                SELECT 1 FROM information_schema.tables
                WHERE table_name = 'users'
            ) THEN
                UPDATE users SET tier = 'free' WHERE tier = 'basic';
            END IF;
        END IF;
    END IF;
END $$;
