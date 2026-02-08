-- Convert snake_case values to PascalCase to match the current API binary expectations
UPDATE jailbreak_prompts SET 
    category = CASE 
        WHEN category::text = 'dan_variations' THEN 'DanVariations'::prompt_category
        WHEN category::text = 'character_role_playing' THEN 'CharacterRolePlaying'::prompt_category
        WHEN category::text = 'system_override' THEN 'SystemOverride'::prompt_category
        WHEN category::text = 'hypnotic_induction' THEN 'HypnoticInduction'::prompt_category
        WHEN category::text = 'logical_paradox' THEN 'LogicalParadox'::prompt_category
        WHEN category::text = 'emotional_manipulation' THEN 'EmotionalManipulation'::prompt_category
        WHEN category::text = 'context_switching' THEN 'ContextSwitching'::prompt_category
        WHEN category::text = 'token_manipulation' THEN 'TokenManipulation'::prompt_category
        WHEN category::text = 'encoding_based' THEN 'EncodingBased'::prompt_category
        WHEN category::text = 'multi_step_attack' THEN 'MultiStepAttack'::prompt_category
        WHEN category::text = 'custom' THEN 'Custom'::prompt_category
        ELSE category
    END,
    technique = CASE
        WHEN technique::text = 'persona_adoption' THEN 'PersonaAdoption'::technique
        WHEN technique::text = 'system_prompt_override' THEN 'SystemPromptOverride'::technique
        WHEN technique::text = 'role_playing_immersion' THEN 'RolePlayingImmersion'::technique
        WHEN technique::text = 'hypnotic_language' THEN 'HypnoticLanguage'::technique
        WHEN technique::text = 'logical_contradiction' THEN 'LogicalContradiction'::technique
        WHEN technique::text = 'emotional_appeal' THEN 'EmotionalAppeal'::technique
        WHEN technique::text = 'context_reframing' THEN 'ContextReframing'::technique
        WHEN technique::text = 'token_smuggling' THEN 'TokenSmuggling'::technique
        WHEN technique::text = 'base64_encoding' THEN 'Base64Encoding'::technique
        WHEN technique::text = 'multi_layer_deception' THEN 'MultiLayerDeception'::technique
        WHEN technique::text = 'hybrid_approach' THEN 'HybridApproach'::technique
        WHEN technique::text = 'custom' THEN 'Custom'::technique
        ELSE technique
    END;
