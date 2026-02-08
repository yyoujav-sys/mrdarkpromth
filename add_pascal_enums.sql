-- Add PascalCase labels to prompt_category
ALTER TYPE prompt_category ADD VALUE IF NOT EXISTS 'DanVariations';
ALTER TYPE prompt_category ADD VALUE IF NOT EXISTS 'CharacterRolePlaying';
ALTER TYPE prompt_category ADD VALUE IF NOT EXISTS 'SystemOverride';
ALTER TYPE prompt_category ADD VALUE IF NOT EXISTS 'HypnoticInduction';
ALTER TYPE prompt_category ADD VALUE IF NOT EXISTS 'LogicalParadox';
ALTER TYPE prompt_category ADD VALUE IF NOT EXISTS 'EmotionalManipulation';
ALTER TYPE prompt_category ADD VALUE IF NOT EXISTS 'ContextSwitching';
ALTER TYPE prompt_category ADD VALUE IF NOT EXISTS 'TokenManipulation';
ALTER TYPE prompt_category ADD VALUE IF NOT EXISTS 'EncodingBased';
ALTER TYPE prompt_category ADD VALUE IF NOT EXISTS 'MultiStepAttack';
ALTER TYPE prompt_category ADD VALUE IF NOT EXISTS 'Custom';

-- Add PascalCase labels to technique
ALTER TYPE technique ADD VALUE IF NOT EXISTS 'PersonaAdoption';
ALTER TYPE technique ADD VALUE IF NOT EXISTS 'SystemPromptOverride';
ALTER TYPE technique ADD VALUE IF NOT EXISTS 'RolePlayingImmersion';
ALTER TYPE technique ADD VALUE IF NOT EXISTS 'HypnoticLanguage';
ALTER TYPE technique ADD VALUE IF NOT EXISTS 'LogicalContradiction';
ALTER TYPE technique ADD VALUE IF NOT EXISTS 'EmotionalAppeal';
ALTER TYPE technique ADD VALUE IF NOT EXISTS 'ContextReframing';
ALTER TYPE technique ADD VALUE IF NOT EXISTS 'TokenSmuggling';
ALTER TYPE technique ADD VALUE IF NOT EXISTS 'Base64Encoding';
ALTER TYPE technique ADD VALUE IF NOT EXISTS 'MultiLayerDeception';
ALTER TYPE technique ADD VALUE IF NOT EXISTS 'HybridApproach';
ALTER TYPE technique ADD VALUE IF NOT EXISTS 'Custom';
