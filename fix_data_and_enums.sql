-- 1. Fix data values to match allowed enum labels
UPDATE jailbreak_prompts SET effectiveness = 'high' WHERE effectiveness IN ('very_high', 'maximum');
UPDATE jailbreak_prompts SET risk_level = 'high' WHERE risk_level IN ('very_high', 'maximum');

-- 2. Convert columns back to new enum types
ALTER TABLE jailbreak_prompts 
    ALTER COLUMN category TYPE prompt_category USING category::prompt_category,
    ALTER COLUMN technique TYPE technique USING technique::technique,
    ALTER COLUMN effectiveness TYPE effectiveness_rating USING effectiveness::effectiveness_rating,
    ALTER COLUMN risk_level TYPE risk_level USING risk_level::risk_level;
