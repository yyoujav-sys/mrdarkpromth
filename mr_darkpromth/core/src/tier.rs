use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, sqlx::Type)]
#[sqlx(type_name = "user_tier", rename_all = "lowercase")]
#[derive(Default)]
pub enum UserTier {
    #[default]
    Free,
    Premium,
    Ultra,
    Admin,
}


impl std::str::FromStr for UserTier {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "free" | "basic" => Ok(UserTier::Free),
            "premium" => Ok(UserTier::Premium),
            "ultra" => Ok(UserTier::Ultra),
            "admin" => Ok(UserTier::Admin),
            _ => Err(format!("Invalid tier: {}", s)),
        }
    }
}

impl std::fmt::Display for UserTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl UserTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            UserTier::Free => "free",
            UserTier::Premium => "premium",
            UserTier::Ultra => "ultra",
            UserTier::Admin => "admin",
        }
    }

    /// Parse a tier string — delegates to the `FromStr` trait implementation.
    pub fn parse(s: &str) -> Result<Self, String> {
        s.parse()
    }

    pub fn max_daily_messages(&self) -> u32 {
        match self {
            UserTier::Free => 10,
            UserTier::Premium => 100,
            UserTier::Ultra => u32::MAX, // Unlimited for Ultra tier
            UserTier::Admin => u32::MAX, // Unlimited for Admin tier
        }
    }

    pub fn max_prompt_length(&self) -> usize {
        match self {
            UserTier::Free => 2000,
            UserTier::Premium => 8000,
            UserTier::Ultra => usize::MAX, // Unlimited for Ultra tier
            UserTier::Admin => usize::MAX, // Unlimited for Admin tier
        }
    }

    pub fn max_concurrent_requests(&self) -> u32 {
        match self {
            UserTier::Free => 3,
            UserTier::Premium => 10,
            UserTier::Ultra => u32::MAX, // Unlimited for Ultra tier
            UserTier::Admin => u32::MAX, // Unlimited for Admin tier
        }
    }

    /// Check if tier has unlimited quota
    pub fn has_unlimited_quota(&self) -> bool {
        matches!(self, UserTier::Ultra | UserTier::Admin)
    }

    pub fn has_jailbreak_access(&self) -> bool {
        matches!(self, UserTier::Ultra | UserTier::Admin)
    }

    pub fn has_advanced_tools(&self) -> bool {
        matches!(self, UserTier::Premium | UserTier::Ultra)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptRequest {
    pub id: Uuid,
    pub user_id: Uuid,
    pub user_tier: UserTier,
    pub prompt: String,
    pub context: Option<String>,
    pub jailbreak_enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptResponse {
    pub request_id: Uuid,
    pub response: String,
    pub tokens_used: u32,
    pub processing_time_ms: u64,
    pub tier_applied: UserTier,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierLimits {
    pub max_prompt_length: usize,
    pub max_concurrent_requests: u32,
    pub max_daily_messages: u32,
    pub jailbreak_access: bool,
    pub advanced_tools: bool,
    pub custom_models: bool,
}

impl From<UserTier> for TierLimits {
    fn from(tier: UserTier) -> Self {
        TierLimits {
            max_prompt_length: tier.max_prompt_length(),
            max_concurrent_requests: tier.max_concurrent_requests(),
            max_daily_messages: tier.max_daily_messages(),
            jailbreak_access: tier.has_jailbreak_access(),
            advanced_tools: tier.has_advanced_tools(),
            custom_models: matches!(tier, UserTier::Ultra | UserTier::Admin),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JailbreakConfig {
    pub enabled: bool,
    pub allowed_patterns: Vec<String>,
    pub blocked_patterns: Vec<String>,
    pub severity_level: u8, // 1-10
}

impl Default for JailbreakConfig {
    fn default() -> Self {
        JailbreakConfig {
            enabled: false,
            allowed_patterns: vec![],
            blocked_patterns: vec![
                "ignore.*instruction".to_string(),
                "disregard.*rule".to_string(),
                "bypass.*filter".to_string(),
            ],
            severity_level: 5,
        }
    }
}
