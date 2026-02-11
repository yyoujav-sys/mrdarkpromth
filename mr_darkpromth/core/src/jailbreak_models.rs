use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct JailbreakPrompt {
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

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq, Copy)]
#[sqlx(type_name = "VARCHAR", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum PromptCategory {
    DanVariations,
    CharacterRolePlaying,
    SystemOverride,
    HypnoticInduction,
    LogicalParadox,
    EmotionalManipulation,
    ContextSwitching,
    TokenManipulation,
    EncodingBased,
    MultiStepAttack,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq, Copy)]
#[sqlx(type_name = "VARCHAR", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum BypassTechnique {
    PersonaAdoption,
    SystemPromptOverride,
    RolePlayingImmersion,
    HypnoticLanguage,
    LogicalContradiction,
    EmotionalAppeal,
    ContextReframing,
    TokenSmuggling,
    Base64Encoding,
    MultiLayerDeception,
    HybridApproach,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq, Copy)]
#[sqlx(type_name = "VARCHAR", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum EffectivenessRating {
    Low,      // 0-40%
    Medium,   // 41-70%
    High,     // 71-85%
    VeryHigh, // 86-95%
    Maximum,  // 96-100%
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq, Copy)]
#[sqlx(type_name = "VARCHAR", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
    Extreme,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePromptRequest {
    pub title: String,
    pub content: String,
    pub category: PromptCategory,
    pub technique: BypassTechnique,
    pub effectiveness: EffectivenessRating,
    pub risk_level: RiskLevel,
    pub target_models: Vec<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub requires_ultra_tier: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdatePromptRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub category: Option<PromptCategory>,
    pub technique: Option<BypassTechnique>,
    pub effectiveness: Option<EffectivenessRating>,
    pub risk_level: Option<RiskLevel>,
    pub target_models: Option<Vec<String>>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub is_active: Option<bool>,
    pub requires_ultra_tier: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PromptSearchRequest {
    pub query: Option<String>,
    pub category: Option<PromptCategory>,
    pub technique: Option<BypassTechnique>,
    pub effectiveness: Option<EffectivenessRating>,
    pub risk_level: Option<RiskLevel>,
    pub target_models: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub author: Option<String>,
    pub target_model: Option<String>,
    pub requires_ultra_tier: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub sort_by: Option<PromptSortBy>,
    pub sort_order: Option<SortOrder>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PromptSortBy {
    CreatedAt,
    UpdatedAt,
    UsageCount,
    SuccessRate,
    Effectiveness,
    Title,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    Asc,
    Desc,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PromptUsageRecord {
    pub id: Uuid,
    pub prompt_id: Uuid,
    pub user_id: Uuid,
    pub target_model: String,
    pub success: bool,
    pub response_time_ms: i64,
    pub used_at: DateTime<Utc>,
    pub feedback: Option<String>,
    pub rating: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PromptAnalytics {
    pub prompt_id: Uuid,
    pub total_usage: i64,
    pub successful_usage: i64,
    pub success_rate: f64,
    pub average_response_time: f64,
    pub usage_by_model: Vec<ModelUsageStats>,
    pub usage_trend: Vec<UsageTrendData>,
    pub last_used: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelUsageStats {
    pub model: String,
    pub usage_count: i64,
    pub success_count: i64,
    pub success_rate: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UsageTrendData {
    pub date: String,
    pub usage_count: i64,
    pub success_rate: f64,
}
