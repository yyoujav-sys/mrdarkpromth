-- Convert columns back to custom enum types to match Rust expectations
ALTER TABLE jailbreak_prompts 
    ALTER COLUMN category TYPE prompt_category USING category::prompt_category,
    ALTER COLUMN technique TYPE technique USING technique::technique,
    ALTER COLUMN effectiveness TYPE effectiveness_rating USING effectiveness::effectiveness_rating,
    ALTER COLUMN risk_level TYPE risk_level USING risk_level::risk_level;
