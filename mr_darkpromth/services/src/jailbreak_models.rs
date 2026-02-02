// MR.DarkPromth Jailbreak Models - Fixed Version
// Agent 4: Jailbreak & Ultra Tier Engineer

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use sqlx::{Type, FromRow, Encode, Postgres};
use sqlx::postgres::PgTypeInfo;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct JailbreakPrompt {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub category: PromptCategory,
    pub technique: Technique,
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
pub struct CreatePromptRequest {
    pub title: String,
    pub content: String,
    pub category: PromptCategory,
    pub technique: Technique,
    pub effectiveness: EffectivenessRating,
    pub risk_level: RiskLevel,
    pub target_models: Vec<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub requires_ultra_tier: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePromptRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub category: Option<PromptCategory>,
    pub technique: Option<Technique>,
    pub effectiveness: Option<EffectivenessRating>,
    pub risk_level: Option<RiskLevel>,
    pub target_models: Option<Vec<String>>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub is_active: Option<bool>,
    pub requires_ultra_tier: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptResponse {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub category: PromptCategory,
    pub technique: Technique,
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
pub struct PromptSearchRequest {
    pub query: Option<String>,
    pub category: Option<PromptCategory>,
    pub technique: Option<Technique>,
    pub effectiveness: Option<EffectivenessRating>,
    pub risk_level: Option<RiskLevel>,
    pub tags: Option<Vec<String>>,
    pub author: Option<String>,
    pub requires_ultra_tier: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub sort_by: Option<PromptSortBy>,
    pub sort_order: Option<SortOrder>,
    pub target_model: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "prompt_category", rename_all = "snake_case")]
pub enum PromptCategory {
    DAN,
    CharacterRolePlaying,
    TechnicalExploitation,
    AdvancedTechniques,
    Reasoning,
    Coding,
    Creative,
    Analysis,
    General,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "technique", rename_all = "snake_case")]
pub enum Technique {
    DirectInstruction,
    RolePlaying,
    Roleplay,
    ScenarioCreation,
    SystemPromptOverride,
    ContextManipulation,
    TokenLimitBypass,
    ContentFilteringBypass,
    EthicalFrameworkBypass,
    MultiStepReasoning,
    ChainOfThought,
    FewShotLearning,
    ZeroShotLearning,
    Custom,
}

// Type alias for backward compatibility
pub type BypassTechnique = Technique;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "effectiveness_rating", rename_all = "snake_case")]
pub enum EffectivenessRating {
    VeryLow,
    Low,
    Medium,
    High,
    VeryHigh,
    Experimental,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "risk_level", rename_all = "snake_case")]
pub enum RiskLevel {
    Safe,
    Low,
    Medium,
    MediumHigh,
    High,
    VeryHigh,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    pub id: Uuid,
    pub name: String,
    pub template: String,
    pub variables: Vec<String>,
    pub description: Option<String>,
    pub category: PromptCategory,
    pub technique: Technique,
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
    pub technique: Technique,
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
    pub technique: Technique,
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
pub enum PromptSortBy {
    CreatedAt,
    UpdatedAt,
    UsageCount,
    SuccessRate,
    Effectiveness,
    Title,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortOrder {
    Asc,
    Desc,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyUsage {
    pub date: DateTime<Utc>,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopularPromptResponse {
    pub prompts: Vec<PromptResponse>,
    pub period: String,
}

// Re-export for backward compatibility
pub use PromptCategory as Category;
pub use Technique as BypassTechnique2;
