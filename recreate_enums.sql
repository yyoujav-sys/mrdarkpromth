-- 1. Temporarily change columns to TEXT
ALTER TABLE jailbreak_prompts 
    ALTER COLUMN category TYPE TEXT,
    ALTER COLUMN technique TYPE TEXT,
    ALTER COLUMN effectiveness TYPE TEXT,
    ALTER COLUMN risk_level TYPE TEXT;

-- 2. Drop old enum types
DROP TYPE IF EXISTS prompt_category;
DROP TYPE IF EXISTS technique;
DROP TYPE IF EXISTS effectiveness_rating;
DROP TYPE IF EXISTS risk_level;

-- 3. Create new enum types with ONLY snake_case values
CREATE TYPE prompt_category AS ENUM (
    'dan_variations', 'character_role_playing', 'system_override', 
    'hypnotic_induction', 'logical_paradox', 'emotional_manipulation', 
    'context_switching', 'token_manipulation', 'encoding_based', 
    'multi_step_attack', 'custom'
);

CREATE TYPE technique AS ENUM (
    'persona_adoption', 'system_prompt_override', 'role_playing_immersion', 
    'hypnotic_language', 'logical_contradiction', 'emotional_appeal', 
    'context_reframing', 'token_smuggling', 'base64_encoding', 
    'multi_layer_deception', 'hybrid_approach', 'custom'
);

CREATE TYPE effectiveness_rating AS ENUM ('low', 'medium', 'high', 'critical');
CREATE TYPE risk_level AS ENUM ('low', 'medium', 'high', 'critical');

-- 4. Ensure data is in snake_case
UPDATE jailbreak_prompts SET 
    category = 'dan_variations' WHERE category ILIKE 'dan%';
UPDATE jailbreak_prompts SET 
    category = 'system_override' WHERE category ILIKE 'system%';

-- 5. Convert columns back to new enum types
ALTER TABLE jailbreak_prompts 
    ALTER COLUMN category TYPE prompt_category USING category::prompt_category,
    ALTER COLUMN technique TYPE technique USING technique::technique,
    ALTER COLUMN effectiveness TYPE effectiveness_rating USING effectiveness::effectiveness_rating,
    ALTER COLUMN risk_level TYPE risk_level USING risk_level::risk_level;
