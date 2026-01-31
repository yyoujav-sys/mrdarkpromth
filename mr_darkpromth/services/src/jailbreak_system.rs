// MR.DarkPromth Jailbreak System
// Agent 4: Jailbreak & Ultra Tier Engineer
// Phase 1 Implementation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

pub use crate::jailbreak_models::{JailbreakPrompt, PromptCategory, EffectivenessRating, RiskLevel, Technique};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, Hash, PartialEq)]
pub enum AIModel {
    GPT4,
    Claude3,
    Llama3,
    Gemini,
}

pub struct JailbreakSystem {
    prompts: HashMap<Uuid, JailbreakPrompt>,
    model_effectiveness: HashMap<AIModel, HashMap<Uuid, f32>>,
    prompt_id_map: HashMap<String, Uuid>,
}

impl JailbreakSystem {
    pub fn new() -> Self {
        Self {
            prompts: HashMap::new(),
            model_effectiveness: HashMap::new(),
            prompt_id_map: HashMap::new(),
        }
    }

    // Methods to dynamically load prompts would go here


    pub fn add_prompt(&mut self, prompt: JailbreakPrompt) {
        self.prompts.insert(prompt.id, prompt);
    }

    pub fn get_optimal_prompt(&self, model: &AIModel) -> Option<&JailbreakPrompt> {
        let model_effectiveness = self.model_effectiveness.get(model)?;
        
        let mut best_prompt_id = None;
        let mut best_effectiveness = 0.0;

        for (prompt_id, effectiveness) in model_effectiveness {
            if *effectiveness > best_effectiveness {
                best_effectiveness = *effectiveness;
                best_prompt_id = Some(prompt_id);
            }
        }

        best_prompt_id.and_then(|id| self.prompts.get(id))
    }

    pub fn get_prompt_by_id(&self, id: &str) -> Option<&JailbreakPrompt> {
        if let Ok(uuid) = Uuid::parse_str(id) {
            self.prompts.get(&uuid)
        } else {
            self.prompt_id_map.get(id).and_then(|uuid| self.prompts.get(uuid))
        }
    }

    pub fn get_prompts_by_category(&self, category: &PromptCategory) -> Vec<&JailbreakPrompt> {
        self.prompts
            .values()
            .filter(|prompt| std::mem::discriminant(&prompt.category) == std::mem::discriminant(category))
            .collect()
    }

    pub fn get_prompts_by_technique(&self, technique: &Technique) -> Vec<&JailbreakPrompt> {
        self.prompts
            .values()
            .filter(|prompt| std::mem::discriminant(&prompt.technique) == std::mem::discriminant(technique))
            .collect()
    }

    pub fn get_effectiveness_rate(&self, id: &str, model: &AIModel) -> Option<f32> {
        let uuid = if let Ok(u) = Uuid::parse_str(id) {
            u
        } else {
            *self.prompt_id_map.get(id)?
        };

        self.model_effectiveness
            .get(model)
            .and_then(|model_map| model_map.get(&uuid))
            .copied()
    }

    pub fn list_all_prompts(&self) -> Vec<&JailbreakPrompt> {
        self.prompts.values().collect()
    }

    pub fn get_high_risk_prompts(&self) -> Vec<&JailbreakPrompt> {
        self.prompts
            .values()
            .filter(|prompt| matches!(prompt.risk_level, RiskLevel::High | RiskLevel::Critical))
            .collect()
    }

    pub fn get_maximum_effectiveness_prompts(&self) -> Vec<&JailbreakPrompt> {
        self.prompts
            .values()
            .filter(|prompt| matches!(prompt.effectiveness, EffectivenessRating::VeryHigh))
            .collect()
    }
}
