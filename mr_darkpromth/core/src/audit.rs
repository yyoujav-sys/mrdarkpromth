use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::tier::UserTier;
use std::str::FromStr;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AuditAction {
    PromptRequest,
    PromptResponse,
    JailbreakAttempt,
    JailbreakApplied,
    SecurityViolation,
    TierUpgrade,
    TierDowngrade,
    UnauthorizedAccess,
    SystemConfigChange,
    DataExport,
    UserLogin,
    UserLogout,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AuditSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub action: AuditAction,
    pub severity: AuditSeverity,
    pub user_tier: Option<UserTier>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub request_id: Option<Uuid>,
    pub details: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub success: bool,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptAuditDetails {
    pub prompt_length: usize,
    pub prompt_preview: String, // First 100 chars
    pub jailbreak_enabled: bool,
    pub processing_time_ms: Option<u64>,
    pub tokens_used: Option<u32>,
    pub model_used: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JailbreakAuditDetails {
    pub attempted_pattern: String,
    pub pattern_matched: bool,
    pub severity_score: u8,
    pub bypass_attempted: bool,
    pub blocked_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierChangeAuditDetails {
    pub old_tier: UserTier,
    pub new_tier: UserTier,
    pub reason: String,
    pub approved_by: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditFilter {
    pub user_id: Option<Uuid>,
    pub action: Option<AuditAction>,
    pub severity: Option<AuditSeverity>,
    pub user_tier: Option<UserTier>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub success_only: Option<bool>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

impl Default for AuditFilter {
    fn default() -> Self {
        AuditFilter {
            user_id: None,
            action: None,
            severity: None,
            user_tier: None,
            start_time: None,
            end_time: None,
            success_only: None,
            limit: Some(100),
            offset: Some(0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditStats {
    pub total_logs: u64,
    pub successful_operations: u64,
    pub failed_operations: u64,
    pub jailbreak_attempts: u64,
    pub ultra_tier_operations: u64,
    pub average_processing_time: f64,
    pub most_active_users: Vec<(Uuid, u64)>,
    pub most_common_actions: Vec<(AuditAction, u64)>,
}

// Display implementations for AuditAction and AuditSeverity
impl fmt::Display for AuditAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuditAction::PromptRequest => write!(f, "PromptRequest"),
            AuditAction::PromptResponse => write!(f, "PromptResponse"),
            AuditAction::JailbreakAttempt => write!(f, "JailbreakAttempt"),
            AuditAction::JailbreakApplied => write!(f, "JailbreakApplied"),
            AuditAction::SecurityViolation => write!(f, "SecurityViolation"),
            AuditAction::TierUpgrade => write!(f, "TierUpgrade"),
            AuditAction::TierDowngrade => write!(f, "TierDowngrade"),
            AuditAction::UnauthorizedAccess => write!(f, "UnauthorizedAccess"),
            AuditAction::SystemConfigChange => write!(f, "SystemConfigChange"),
            AuditAction::DataExport => write!(f, "DataExport"),
            AuditAction::UserLogin => write!(f, "UserLogin"),
            AuditAction::UserLogout => write!(f, "UserLogout"),
        }
    }
}

impl fmt::Display for AuditSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuditSeverity::Low => write!(f, "Low"),
            AuditSeverity::Medium => write!(f, "Medium"),
            AuditSeverity::High => write!(f, "High"),
            AuditSeverity::Critical => write!(f, "Critical"),
        }
    }
}

// FromStr implementations for database conversions
impl FromStr for AuditAction {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "PromptRequest" => Ok(AuditAction::PromptRequest),
            "PromptResponse" => Ok(AuditAction::PromptResponse),
            "JailbreakAttempt" => Ok(AuditAction::JailbreakAttempt),
            "JailbreakApplied" => Ok(AuditAction::JailbreakApplied),
            "SecurityViolation" => Ok(AuditAction::SecurityViolation),
            "TierUpgrade" => Ok(AuditAction::TierUpgrade),
            "TierDowngrade" => Ok(AuditAction::TierDowngrade),
            "UnauthorizedAccess" => Ok(AuditAction::UnauthorizedAccess),
            "SystemConfigChange" => Ok(AuditAction::SystemConfigChange),
            "DataExport" => Ok(AuditAction::DataExport),
            "UserLogin" => Ok(AuditAction::UserLogin),
            "UserLogout" => Ok(AuditAction::UserLogout),
            _ => Err(format!("Invalid AuditAction: {}", s)),
        }
    }
}

impl FromStr for AuditSeverity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Low" => Ok(AuditSeverity::Low),
            "Medium" => Ok(AuditSeverity::Medium),
            "High" => Ok(AuditSeverity::High),
            "Critical" => Ok(AuditSeverity::Critical),
            _ => Err(format!("Invalid AuditSeverity: {}", s)),
        }
    }
}
