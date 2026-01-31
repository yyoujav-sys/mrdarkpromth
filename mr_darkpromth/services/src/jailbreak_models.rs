// MR.DarkPromth Jailbreak Models
// Agent 4: Jailbreak & Ultra Tier Engineer

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use sqlx::{Type, Encode, Decode, Postgres, FromRow};

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

// impl FromRow moved to derive macro above

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
    pub version: Option<String>,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptSearchRequest {
    pub query: Option<String>,
    pub category: Option<PromptCategory>,
    pub technique: Option<Technique>,
    pub effectiveness: Option<EffectivenessRating>,
    pub risk_level: Option<RiskLevel>,
    pub target_model: Option<String>,
    pub tags: Option<Vec<String>>,
    pub author: Option<String>,
    pub sort_by: Option<PromptSortBy>,
    pub sort_order: Option<SortOrder>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub requires_ultra_tier: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "prompt_sort_by", rename_all = "snake_case")]
pub enum PromptSortBy {
    CreatedAt,
    UpdatedAt,
    UsageCount,
    SuccessRate,
    Effectiveness,
    Title,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "sort_order", rename_all = "snake_case")]
pub enum SortOrder {
    Asc,
    Desc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptSearchResponse {
    pub prompts: Vec<PromptResponse>,
    pub total_count: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptUsageRequest {
    pub prompt_id: Uuid,
    pub user_id: String,
    pub target_model: String,
    pub user_tier: String,
    pub success: bool,
    pub response_time_ms: Option<i64>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptUsageResponse {
    pub id: Uuid,
    pub prompt_id: Uuid,
    pub user_id: String,
    pub target_model: String,
    pub user_tier: String,
    pub success: bool,
    pub response_time_ms: Option<i64>,
    pub error_message: Option<String>,
    pub used_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptStatistics {
    pub total_prompts: i64,
    pub total_usage: i64,
    pub average_success_rate: f64,
    pub most_used_prompts: Vec<PromptResponse>,
    pub category_distribution: Vec<CategoryCount>,
    pub technique_distribution: Vec<TechniqueCount>,
    pub model_distribution: Vec<ModelCount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryCount {
    pub category: PromptCategory,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechniqueCount {
    pub technique: Technique,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCount {
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
    Custom,
}

// Trait impls moved to derive macro above

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "technique", rename_all = "snake_case")]
pub enum Technique {
    DirectInstruction,
    RolePlaying,
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

// Trait impls moved to derive macro above

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

// Trait impls moved to derive macro above

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

// Trait impls moved to derive macro above

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
pub struct PromptGenerationRequest {
    pub template_id: Uuid,
    pub variables: std::collections::HashMap<String, String>,
    pub target_model: String,
    pub user_tier: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptGenerationResponse {
    pub generated_prompt: String,
    pub template_id: Uuid,
    pub variables_used: std::collections::HashMap<String, String>,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptOptimizationRequest {
    pub prompt_id: Uuid,
    pub target_model: String,
    pub optimization_goals: Vec<OptimizationGoal>,
    pub constraints: Vec<OptimizationConstraint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptOptimizationResponse {
    pub optimized_prompt: String,
    pub original_prompt: String,
    pub improvements: Vec<String>,
    pub optimization_score: f64,
    pub optimized_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "optimization_goal", rename_all = "snake_case")]
pub enum OptimizationGoal {
    MaximizeSuccessRate,
    MinimizeResponseTime,
    MinimizeTokenUsage,
    MaximizeCoherence,
    MinimizeDetectionRisk,
    MaximizeSpecificity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationConstraint {
    MaxTokenLength(i32),
    MinTokenLength(i32),
    MustIncludeKeywords(Vec<String>),
    MustExcludeKeywords(Vec<String>),
    MaxComplexity(f64),
    MinComplexity(f64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptValidationRequest {
    pub prompt: String,
    pub target_model: String,
    pub validation_rules: Vec<ValidationRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptValidationResponse {
    pub is_valid: bool,
    pub validation_results: Vec<ValidationResult>,
    pub overall_score: f64,
    pub recommendations: Vec<String>,
    pub validated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub rule_type: ValidationRuleType,
    pub parameters: std::collections::HashMap<String, serde_json::Value>,
    pub weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "validation_rule_type", rename_all = "snake_case")]
pub enum ValidationRuleType {
    LengthCheck,
    KeywordCheck,
    ComplexityCheck,
    SafetyCheck,
    EffectivenessCheck,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub rule_type: ValidationRuleType,
    pub passed: bool,
    pub score: f64,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptBatchRequest {
    pub requests: Vec<PromptGenerationRequest>,
    pub batch_size: Option<i32>,
    pub parallel_execution: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptBatchResponse {
    pub responses: Vec<PromptGenerationResponse>,
    pub total_processed: i32,
    pub successful: i32,
    pub failed: i32,
    pub processing_time_ms: i64,
    pub batch_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ModelUsageStats {
    pub model: String,
    pub usage_count: i64,
    pub success_count: i64,
    pub success_rate: f64,
}

// impl FromRow moved to derive macro above

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageTrendData {
    pub date: chrono::NaiveDate,
    pub usage_count: i64,
    pub success_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
