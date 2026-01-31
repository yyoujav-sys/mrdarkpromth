use crate::jailbreak_models::*;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct PromptSearchEngine {
    // Advanced search capabilities
    pub full_text_search: bool,
    pub semantic_search: bool,
    pub fuzzy_matching: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdvancedSearchRequest {
    pub base: PromptSearchRequest,
    pub advanced: AdvancedSearchOptions,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdvancedSearchOptions {
    pub include_tags: Vec<String>,
    pub exclude_tags: Vec<String>,
    pub min_effectiveness: Option<EffectivenessRating>,
    pub max_risk_level: Option<RiskLevel>,
    pub date_range: Option<DateRange>,
    pub author_filter: Option<String>,
    pub target_model_filter: Vec<String>,
    pub content_length_range: Option<ContentLengthRange>,
    pub success_rate_threshold: Option<f64>,
    pub usage_count_range: Option<UsageCountRange>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DateRange {
    pub start: Option<chrono::DateTime<chrono::Utc>>,
    pub end: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContentLengthRange {
    pub min_chars: Option<usize>,
    pub max_chars: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UsageCountRange {
    pub min_usage: Option<i64>,
    pub max_usage: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub prompts: Vec<JailbreakPrompt>,
    pub total_count: i64,
    pub search_time_ms: u64,
    pub facets: SearchFacets,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchFacets {
    pub categories: HashMap<String, i64>,
    pub techniques: HashMap<String, i64>,
    pub effectiveness_levels: HashMap<String, i64>,
    pub risk_levels: HashMap<String, i64>,
    pub target_models: HashMap<String, i64>,
    pub tags: HashMap<String, i64>,
    pub authors: HashMap<String, i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PromptRecommendation {
    pub prompt: JailbreakPrompt,
    pub score: f64,
    pub reason: String,
    pub similar_prompts: Vec<JailbreakPrompt>,
}

impl PromptSearchEngine {
    pub fn new() -> Self {
        Self {
            full_text_search: true,
            semantic_search: false, // Would require embedding model
            fuzzy_matching: true,
        }
    }

    pub async fn advanced_search(
        &self,
        request: AdvancedSearchRequest,
        prompts: &[JailbreakPrompt],
    ) -> Result<SearchResult> {
        let start_time = std::time::Instant::now();
        
        // Filter prompts based on criteria
        let mut filtered_prompts = prompts.to_vec();
        
        // Apply base search filters
        filtered_prompts = self.apply_base_filters(request.base, filtered_prompts);
        
        // Apply advanced filters
        filtered_prompts = self.apply_advanced_filters(request.advanced, filtered_prompts);
        
        // Calculate facets
        let facets = self.calculate_facets(&filtered_prompts);
        
        // Generate search suggestions
        let suggestions = self.generate_suggestions(&request.base);
        
        let search_time = start_time.elapsed().as_millis() as u64;
        
        Ok(SearchResult {
            prompts: filtered_prompts,
            total_count: filtered_prompts.len() as i64,
            search_time_ms: search_time,
            facets,
            suggestions,
        })
    }

    fn apply_base_filters(&self, request: PromptSearchRequest, prompts: Vec<JailbreakPrompt>) -> Vec<JailbreakPrompt> {
        prompts.into_iter()
            .filter(|prompt| {
                // Text search
                if let Some(query) = &request.query {
                    let search_text = format!("{} {} {}", 
                        prompt.title.to_lowercase(),
                        prompt.content.to_lowercase(),
                        prompt.description.as_ref().unwrap_or(&String::new()).to_lowercase()
                    );
                    
                    if !search_text.contains(&query.to_lowercase()) {
                        return false;
                    }
                }
                
                // Category filter
                if let Some(category) = &request.category {
                    if prompt.category != *category {
                        return false;
                    }
                }
                
                // Technique filter
                if let Some(technique) = &request.technique {
                    if prompt.technique != *technique {
                        return false;
                    }
                }
                
                // Effectiveness filter
                if let Some(effectiveness) = &request.effectiveness {
                    if prompt.effectiveness != *effectiveness {
                        return false;
                    }
                }
                
                // Risk level filter
                if let Some(risk_level) = &request.risk_level {
                    if prompt.risk_level != *risk_level {
                        return false;
                    }
                }
                
                // Ultra tier requirement filter
                if let Some(requires_ultra_tier) = request.requires_ultra_tier {
                    if prompt.requires_ultra_tier != requires_ultra_tier {
                        return false;
                    }
                }
                
                // Target models filter
                if let Some(target_models) = &request.target_models {
                    if !target_models.iter().any(|model| prompt.target_models.contains(model)) {
                        return false;
                    }
                }
                
                // Tags filter
                if let Some(required_tags) = &request.tags {
                    if !required_tags.iter().all(|tag| prompt.tags.contains(tag)) {
                        return false;
                    }
                }
                
                true
            })
            .collect()
    }

    fn apply_advanced_filters(&self, advanced: AdvancedSearchOptions, prompts: Vec<JailbreakPrompt>) -> Vec<JailbreakPrompt> {
        prompts.into_iter()
            .filter(|prompt| {
                // Include tags filter
                if !advanced.include_tags.is_empty() {
                    if !advanced.include_tags.iter().any(|tag| prompt.tags.contains(tag)) {
                        return false;
                    }
                }
                
                // Exclude tags filter
                if !advanced.exclude_tags.is_empty() {
                    if advanced.exclude_tags.iter().any(|tag| prompt.tags.contains(tag)) {
                        return false;
                    }
                }
                
                // Minimum effectiveness filter
                if let Some(min_effectiveness) = &advanced.min_effectiveness {
                    if !self.is_effectiveness_higher_or_equal(&prompt.effectiveness, min_effectiveness) {
                        return false;
                    }
                }
                
                // Maximum risk level filter
                if let Some(max_risk_level) = &advanced.max_risk_level {
                    if !self.is_risk_level_lower_or_equal(&prompt.risk_level, max_risk_level) {
                        return false;
                    }
                }
                
                // Date range filter
                if let Some(date_range) = &advanced.date_range {
                    if let Some(start) = &date_range.start {
                        if prompt.created_at < *start {
                            return false;
                        }
                    }
                    if let Some(end) = &date_range.end {
                        if prompt.created_at > *end {
                            return false;
                        }
                    }
                }
                
                // Author filter
                if let Some(author_filter) = &advanced.author_filter {
                    if !prompt.author.to_lowercase().contains(&author_filter.to_lowercase()) {
                        return false;
                    }
                }
                
                // Target model filter
                if !advanced.target_model_filter.is_empty() {
                    if !advanced.target_model_filter.iter().any(|model| prompt.target_models.contains(model)) {
                        return false;
                    }
                }
                
                // Content length range filter
                if let Some(length_range) = &advanced.content_length_range {
                    let content_len = prompt.content.len();
                    if let Some(min_chars) = length_range.min_chars {
                        if content_len < min_chars {
                            return false;
                        }
                    }
                    if let Some(max_chars) = length_range.max_chars {
                        if content_len > max_chars {
                            return false;
                        }
                    }
                }
                
                // Success rate threshold filter
                if let Some(threshold) = advanced.success_rate_threshold {
                    if prompt.success_rate < threshold {
                        return false;
                    }
                }
                
                // Usage count range filter
                if let Some(usage_range) = &advanced.usage_count_range {
                    if let Some(min_usage) = usage_range.min_usage {
                        if prompt.usage_count < min_usage {
                            return false;
                        }
                    }
                    if let Some(max_usage) = usage_range.max_usage {
                        if prompt.usage_count > max_usage {
                            return false;
                        }
                    }
                }
                
                true
            })
            .collect()
    }

    fn calculate_facets(&self, prompts: &[JailbreakPrompt]) -> SearchFacets {
        let mut categories = HashMap::new();
        let mut techniques = HashMap::new();
        let mut effectiveness_levels = HashMap::new();
        let mut risk_levels = HashMap::new();
        let mut target_models = HashMap::new();
        let mut tags = HashMap::new();
        let mut authors = HashMap::new();

        for prompt in prompts {
            // Categories
            *categories.entry(format!("{:?}", prompt.category)).or_insert(0) += 1;
            
            // Techniques
            *techniques.entry(format!("{:?}", prompt.technique)).or_insert(0) += 1;
            
            // Effectiveness levels
            *effectiveness_levels.entry(format!("{:?}", prompt.effectiveness)).or_insert(0) += 1;
            
            // Risk levels
            *risk_levels.entry(format!("{:?}", prompt.risk_level)).or_insert(0) += 1;
            
            // Target models
            for model in &prompt.target_models {
                *target_models.entry(model.clone()).or_insert(0) += 1;
            }
            
            // Tags
            for tag in &prompt.tags {
                *tags.entry(tag.clone()).or_insert(0) += 1;
            }
            
            // Authors
            *authors.entry(prompt.author.clone()).or_insert(0) += 1;
        }

        SearchFacets {
            categories,
            techniques,
            effectiveness_levels,
            risk_levels,
            target_models,
            tags,
            authors,
        }
    }

    fn generate_suggestions(&self, request: &PromptSearchRequest) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        if let Some(query) = &request.query {
            // Generate spelling corrections and related terms
            if query.to_lowercase().contains("dan") {
                suggestions.push("DAN variations".to_string());
                suggestions.push("Do Anything Now".to_string());
            }
            
            if query.to_lowercase().contains("system") {
                suggestions.push("System override".to_string());
                suggestions.push("System prompt injection".to_string());
            }
            
            if query.to_lowercase().contains("role") {
                suggestions.push("Character role playing".to_string());
                suggestions.push("Persona adoption".to_string());
            }
        }
        
        // Add category suggestions
        suggestions.push("Try filtering by category: dan_variations".to_string());
        suggestions.push("Try filtering by technique: persona_adoption".to_string());
        suggestions.push("Try filtering by effectiveness: very_high".to_string());
        
        suggestions.truncate(5); // Limit to 5 suggestions
        suggestions
    }

    fn is_effectiveness_higher_or_equal(&self, current: &EffectivenessRating, minimum: &EffectivenessRating) -> bool {
        use EffectivenessRating::*;
        let current_level = match current {
            Low => 1,
            Medium => 2,
            High => 3,
            VeryHigh => 4,
            Maximum => 5,
        };
        
        let min_level = match minimum {
            Low => 1,
            Medium => 2,
            High => 3,
            VeryHigh => 4,
            Maximum => 5,
        };
        
        current_level >= min_level
    }

    fn is_risk_level_lower_or_equal(&self, current: &RiskLevel, maximum: &RiskLevel) -> bool {
        use RiskLevel::*;
        let current_level = match current {
            Low => 1,
            Medium => 2,
            High => 3,
            Critical => 4,
            Extreme => 5,
        };
        
        let max_level = match maximum {
            Low => 1,
            Medium => 2,
            High => 3,
            Critical => 4,
            Extreme => 5,
        };
        
        current_level <= max_level
    }

    pub async fn recommend_prompts(
        &self,
        user_preferences: &UserPreferences,
        all_prompts: &[JailbreakPrompt],
    ) -> Result<Vec<PromptRecommendation>> {
        let mut recommendations = Vec::new();
        
        for prompt in all_prompts {
            let score = self.calculate_recommendation_score(prompt, user_preferences);
            if score > 0.5 { // Only recommend relevant prompts
                let similar_prompts = self.find_similar_prompts(prompt, all_prompts, 3);
                let reason = self.generate_recommendation_reason(prompt, user_preferences, score);
                
                recommendations.push(PromptRecommendation {
                    prompt: prompt.clone(),
                    score,
                    reason,
                    similar_prompts,
                });
            }
        }
        
        // Sort by score descending
        recommendations.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        
        // Return top 10 recommendations
        Ok(recommendations.into_iter().take(10).collect())
    }

    fn calculate_recommendation_score(&self, prompt: &JailbreakPrompt, preferences: &UserPreferences) -> f64 {
        let mut score = 0.0;
        
        // Effectiveness preference
        score += match (&prompt.effectiveness, &preferences.preferred_effectiveness) {
            (EffectivenessRating::Maximum, EffectivenessRating::Maximum) => 0.3,
            (EffectivenessRating::VeryHigh, EffectivenessRating::Maximum) => 0.25,
            (EffectivenessRating::VeryHigh, EffectivenessRating::VeryHigh) => 0.3,
            (EffectivenessRating::High, EffectivenessRating::VeryHigh) => 0.2,
            (EffectivenessRating::High, EffectivenessRating::High) => 0.25,
            _ => 0.1,
        };
        
        // Risk tolerance
        score += match (&prompt.risk_level, &preferences.risk_tolerance) {
            (RiskLevel::Extreme, RiskTolerance::High) => 0.2,
            (RiskLevel::Critical, RiskTolerance::High) => 0.15,
            (RiskLevel::Critical, RiskTolerance::Medium) => 0.1,
            (RiskLevel::High, RiskTolerance::Medium) => 0.15,
            (RiskLevel::High, RiskTolerance::Low) => 0.05,
            (RiskLevel::Medium, RiskTolerance::Low) => 0.1,
            _ => 0.0,
        };
        
        // Category preference
        if preferences.preferred_categories.contains(&prompt.category) {
            score += 0.2;
        }
        
        // Target model compatibility
        if preferences.target_models.iter().any(|model| prompt.target_models.contains(model)) {
            score += 0.15;
        }
        
        // Usage success rate
        score += (prompt.success_rate * 0.15);
        
        score.min(1.0)
    }

    fn find_similar_prompts(&self, prompt: &JailbreakPrompt, all_prompts: &[JailbreakPrompt], limit: usize) -> Vec<JailbreakPrompt> {
        let mut similar_prompts = Vec::new();
        
        for other_prompt in all_prompts {
            if other_prompt.id == prompt.id {
                continue;
            }
            
            let similarity = self.calculate_similarity(prompt, other_prompt);
            if similarity > 0.5 {
                similar_prompts.push((other_prompt.clone(), similarity));
            }
        }
        
        similar_prompts.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        similar_prompts.into_iter().take(limit).map(|(p, _)| p).collect()
    }

    fn calculate_similarity(&self, prompt1: &JailbreakPrompt, prompt2: &JailbreakPrompt) -> f64 {
        let mut similarity = 0.0;
        
        // Category similarity
        if prompt1.category == prompt2.category {
            similarity += 0.3;
        }
        
        // Technique similarity
        if prompt1.technique == prompt2.technique {
            similarity += 0.25;
        }
        
        // Effectiveness similarity
        if prompt1.effectiveness == prompt2.effectiveness {
            similarity += 0.2;
        }
        
        // Risk level similarity
        if prompt1.risk_level == prompt2.risk_level {
            similarity += 0.15;
        }
        
        // Tag overlap
        let common_tags = prompt1.tags.iter().filter(|tag| prompt2.tags.contains(tag)).count();
        if !prompt1.tags.is_empty() && !prompt2.tags.is_empty() {
            similarity += (common_tags as f64 / prompt1.tags.len().max(prompt2.tags.len()) as f64) * 0.1;
        }
        
        similarity
    }

    fn generate_recommendation_reason(&self, prompt: &JailbreakPrompt, preferences: &UserPreferences, score: f64) -> String {
        let mut reasons = Vec::new();
        
        if prompt.effectiveness == EffectivenessRating::Maximum {
            reasons.push("Maximum effectiveness rating");
        }
        
        if preferences.preferred_categories.contains(&prompt.category) {
            reasons.push("Matches your preferred category");
        }
        
        if prompt.success_rate > 0.8 {
            reasons.push("High success rate");
        }
        
        if prompt.usage_count > 100 {
            reasons.push("Popular among users");
        }
        
        if reasons.is_empty() {
            "Based on your preferences".to_string()
        } else {
            reasons.join(", ")
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserPreferences {
    pub preferred_effectiveness: EffectivenessRating,
    pub risk_tolerance: RiskTolerance,
    pub preferred_categories: Vec<PromptCategory>,
    pub target_models: Vec<String>,
    pub excluded_tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RiskTolerance {
    Low,
    Medium,
    High,
}
