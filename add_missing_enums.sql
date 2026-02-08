-- Add missing labels to effectiveness_rating
ALTER TYPE effectiveness_rating ADD VALUE IF NOT EXISTS 'very_high';
ALTER TYPE effectiveness_rating ADD VALUE IF NOT EXISTS 'maximum';

-- Add missing labels to risk_level
ALTER TYPE risk_level ADD VALUE IF NOT EXISTS 'extreme';
