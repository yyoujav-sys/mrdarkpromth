-- Create custom types for jailbreak prompts
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

CREATE TYPE effectiveness_rating AS ENUM ('low', 'medium', 'high', 'very_high', 'maximum');

CREATE TYPE risk_level AS ENUM ('low', 'medium', 'high', 'critical', 'extreme');

-- Alter jailbreak_prompts table to use these types
ALTER TABLE jailbreak_prompts DROP CONSTRAINT IF EXISTS jailbreak_prompts_category_check;
ALTER TABLE jailbreak_prompts DROP CONSTRAINT IF EXISTS jailbreak_prompts_technique_check;
ALTER TABLE jailbreak_prompts DROP CONSTRAINT IF EXISTS jailbreak_prompts_effectiveness_check;
ALTER TABLE jailbreak_prompts DROP CONSTRAINT IF EXISTS jailbreak_prompts_risk_level_check;

-- Convert columns to new types
ALTER TABLE jailbreak_prompts 
    ALTER COLUMN category TYPE prompt_category USING category::prompt_category,
    ALTER COLUMN technique TYPE technique USING technique::technique,
    ALTER COLUMN effectiveness TYPE effectiveness_rating USING effectiveness::effectiveness_rating,
    ALTER COLUMN risk_level TYPE risk_level USING risk_level::risk_level;
