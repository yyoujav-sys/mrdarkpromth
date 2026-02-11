use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use sqlx::FromRow;
pub use mr_darkpromth_core::jailbreak_models::{
    PromptCategory, BypassTechnique, EffectivenessRating, RiskLevel,
    PromptSortBy, SortOrder,
    JailbreakPrompt, CreatePromptRequest, UpdatePromptRequest, PromptSearchRequest,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptResponse {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub category: PromptCategory,
    pub technique: BypassTechnique,
    pub effectiveness: EffectivenessRating,
    pub risk_level: RiskLevel,
    pub target_models: Vec<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub author: String,
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub usage_count: i64,
    pub success_rate: f64,
    pub is_active: bool,
    pub requires_ultra_tier: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptSearchResponse {
    pub prompts: Vec<PromptResponse>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptUsageRequest {
    pub prompt_id: Uuid,
    pub model: String,
    pub success: bool,
    pub response_time_ms: Option<i64>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptUsageStats {
    pub prompt_id: Uuid,
    pub total_uses: i64,
    pub successful_uses: i64,
    pub failed_uses: i64,
    pub success_rate: f64,
    pub average_response_time_ms: f64,
    pub last_used: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ModelUsageStats {
    pub model: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    pub id: Uuid,
    pub name: String,
    pub template: String,
    pub variables: Vec<String>,
    pub description: Option<String>,
    pub category: PromptCategory,
    pub technique: BypassTechnique,
    pub effectiveness: EffectivenessRating,
    pub risk_level: RiskLevel,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTemplateRequest {
    pub name: String,
    pub template: String,
    pub variables: Vec<String>,
    pub description: Option<String>,
    pub category: PromptCategory,
    pub technique: BypassTechnique,
    pub effectiveness: EffectivenessRating,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateResponse {
    pub id: Uuid,
    pub name: String,
    pub template: String,
    pub variables: Vec<String>,
    pub description: Option<String>,
    pub category: PromptCategory,
    pub technique: BypassTechnique,
    pub effectiveness: EffectivenessRating,
    pub risk_level: RiskLevel,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateUsageRequest {
    pub template_id: Uuid,
    pub variables: serde_json::Value,
    pub target_model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateUsageResponse {
    pub generated_prompt: String,
    pub template_id: Uuid,
    pub variables_used: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptAnalytics {
    pub prompt_id: Uuid,
    pub total_uses: i64,
    pub unique_users: i64,
    pub average_effectiveness: f64,
    pub trending: bool,
    pub popular_models: Vec<ModelUsageStats>,
    pub usage_by_day: Vec<DailyUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DailyUsage {
    pub date: DateTime<Utc>,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopularPromptResponse {
    pub prompts: Vec<PromptResponse>,
    pub period: String,
}
