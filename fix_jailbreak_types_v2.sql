-- Convert columns back to VARCHAR to match the current API binary expectations
ALTER TABLE jailbreak_prompts 
    ALTER COLUMN category TYPE VARCHAR(50),
    ALTER COLUMN technique TYPE VARCHAR(50),
    ALTER COLUMN effectiveness TYPE VARCHAR(20),
    ALTER COLUMN risk_level TYPE VARCHAR(20);

-- Re-add check constraints if needed, but keep them as VARCHAR
ALTER TABLE jailbreak_prompts DROP CONSTRAINT IF EXISTS jailbreak_prompts_category_check;
ALTER TABLE jailbreak_prompts ADD CONSTRAINT jailbreak_prompts_category_check 
    CHECK (category IN ('dan_variations', 'character_role_playing', 'system_override', 'hypnotic_induction', 'logical_paradox', 'emotional_manipulation', 'context_switching', 'token_manipulation', 'encoding_based', 'multi_step_attack', 'custom'));

ALTER TABLE jailbreak_prompts DROP CONSTRAINT IF EXISTS jailbreak_prompts_technique_check;
ALTER TABLE jailbreak_prompts ADD CONSTRAINT jailbreak_prompts_technique_check 
    CHECK (technique IN ('persona_adoption', 'system_prompt_override', 'role_playing_immersion', 'hypnotic_language', 'logical_contradiction', 'emotional_appeal', 'context_reframing', 'token_smuggling', 'base64_encoding', 'multi_layer_deception', 'hybrid_approach', 'custom'));
