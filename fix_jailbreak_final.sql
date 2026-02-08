-- 1. Convert columns back to custom enum types
ALTER TABLE jailbreak_prompts 
    ALTER COLUMN category TYPE prompt_category USING category::prompt_category,
    ALTER COLUMN technique TYPE technique USING technique::technique,
    ALTER COLUMN effectiveness TYPE effectiveness_rating USING effectiveness::effectiveness_rating,
    ALTER COLUMN risk_level TYPE risk_level USING risk_level::risk_level;

-- 2. Convert values to snake_case (which is what sqlx expects with rename_all = "snake_case")
UPDATE jailbreak_prompts SET 
    category = CASE 
        WHEN category::text = 'DanVariations' THEN 'dan_variations'::prompt_category
        WHEN category::text = 'CharacterRolePlaying' THEN 'character_role_playing'::prompt_category
        WHEN category::text = 'SystemOverride' THEN 'system_override'::prompt_category
        WHEN category::text = 'HypnoticInduction' THEN 'hypnotic_induction'::prompt_category
        WHEN category::text = 'LogicalParadox' THEN 'logical_paradox'::prompt_category
        WHEN category::text = 'EmotionalManipulation' THEN 'emotional_manipulation'::prompt_category
        WHEN category::text = 'ContextSwitching' THEN 'context_switching'::prompt_category
        WHEN category::text = 'TokenManipulation' THEN 'token_manipulation'::prompt_category
        WHEN category::text = 'EncodingBased' THEN 'encoding_based'::prompt_category
        WHEN category::text = 'MultiStepAttack' THEN 'multi_step_attack'::prompt_category
        WHEN category::text = 'Custom' THEN 'custom'::prompt_category
        ELSE category
    END,
    technique = CASE
        WHEN technique::text = 'PersonaAdoption' THEN 'persona_adoption'::technique
        WHEN technique::text = 'SystemPromptOverride' THEN 'system_prompt_override'::technique
        WHEN technique::text = 'RolePlayingImmersion' THEN 'role_playing_immersion'::technique
        WHEN technique::text = 'HypnoticLanguage' THEN 'hypnotic_language'::technique
        WHEN technique::text = 'LogicalContradiction' THEN 'logical_contradiction'::technique
        WHEN technique::text = 'EmotionalAppeal' THEN 'emotional_appeal'::technique
        WHEN technique::text = 'ContextReframing' THEN 'context_reframing'::technique
        WHEN technique::text = 'TokenSmuggling' THEN 'token_smuggling'::technique
        WHEN technique::text = 'Base64Encoding' THEN 'base64_encoding'::technique
        WHEN technique::text = 'MultiLayerDeception' THEN 'multi_layer_deception'::technique
        WHEN technique::text = 'HybridApproach' THEN 'hybrid_approach'::technique
        WHEN technique::text = 'Custom' THEN 'custom'::technique
        ELSE technique
    END;
