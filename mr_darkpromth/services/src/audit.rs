use async_trait::async_trait;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::OpenOptions;
use std::io::Write;
use uuid::Uuid;
use mr_darkpromth_core::{
    audit::{AuditLog, AuditFilter, AuditStats, AuditAction, AuditSeverity},
    tier::UserTier,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEventType {
    PromptCreated,
    JailbreakExecuted,
    CodeExecuted,
    PromptRequest,
    PromptResponse,
    UnauthorizedAccess,
    TierChanged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub event_type: AuditEventType,
    pub user_id: String,
    pub details: String,
    pub metadata: Value,
}

#[async_trait]
pub trait AuditService: Send + Sync {
    async fn log_audit(&self, audit_log: AuditLog) -> Result<()>;
    async fn get_audit_logs(&self, filter: AuditFilter) -> Result<Vec<AuditLog>>;
    async fn get_audit_stats(&self, filter: AuditFilter) -> Result<AuditStats>;
    async fn cleanup_old_logs(&self, older_than: DateTime<Utc>) -> Result<u64>;
}

pub struct AuditLogger {
    service: Option<Box<dyn AuditService>>,
    entries: Vec<AuditEvent>,
    log_file_path: Option<String>,
}

impl AuditLogger {
    pub fn new(service: Box<dyn AuditService>) -> Self {
        Self {
            service: Some(service),
            entries: Vec::new(),
            log_file_path: None,
        }
    }

    pub fn new_file(log_file_path: &str) -> Self {
        Self {
            service: None,
            entries: Vec::new(),
            log_file_path: Some(log_file_path.to_string()),
        }
    }

    pub fn log(&mut self, event: AuditEvent) {
        self.entries.push(event.clone());

        if let Some(path) = &self.log_file_path {
            if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
                let metadata = if event.metadata.is_null() {
                    String::new()
                } else {
                    serde_json::to_string(&event.metadata).unwrap_or_default()
                };
                let log_line = format!(
                    "[{}] [{}] [USER:{}] [{:?}] {} {}\n",
                    event.timestamp.format("%Y-%m-%dT%H:%M:%S%.3fZ"),
                    event.id,
                    event.user_id,
                    event.event_type,
                    event.details,
                    metadata
                );
                let _ = file.write_all(log_line.as_bytes());
            }
        }
    }

    pub fn get_entries(&self) -> &[AuditEvent] {
        &self.entries
    }

    pub async fn log_prompt_request(
        &self,
        user_id: Uuid,
        user_tier: UserTier,
        request_id: Uuid,
        prompt: &str,
        jailbreak_enabled: bool,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<()> {
        let details = serde_json::json!({
            "prompt_length": prompt.len(),
            "prompt_preview": prompt.chars().take(100).collect::<String>(),
            "jailbreak_enabled": jailbreak_enabled,
        });

        let audit_log = AuditLog {
            id: Uuid::new_v4(),
            user_id: Some(user_id),
            action: AuditAction::PromptRequest,
            severity: if jailbreak_enabled {
                AuditSeverity::High
            } else {
                AuditSeverity::Low
            },
            user_tier: Some(user_tier),
            ip_address,
            user_agent,
            request_id: Some(request_id),
            details,
            timestamp: Utc::now(),
            success: true,
            error_message: None,
        };

        if let Some(service) = &self.service {
            service.log_audit(audit_log).await
        } else {
            Ok(())
        }
    }

    pub async fn log_prompt_response(
        &self,
        user_id: Uuid,
        user_tier: UserTier,
        request_id: Uuid,
        processing_time_ms: u64,
        tokens_used: u32,
        model_used: &str,
    ) -> Result<()> {
        let details = serde_json::json!({
            "processing_time_ms": processing_time_ms,
            "tokens_used": tokens_used,
            "model_used": model_used,
        });

        let audit_log = AuditLog {
            id: Uuid::new_v4(),
            user_id: Some(user_id),
            action: AuditAction::PromptResponse,
            severity: AuditSeverity::Low,
            user_tier: Some(user_tier),
            ip_address: None,
            user_agent: None,
            request_id: Some(request_id),
            details,
            timestamp: Utc::now(),
            success: true,
            error_message: None,
        };

        if let Some(service) = &self.service {
            service.log_audit(audit_log).await
        } else {
            Ok(())
        }
    }

    pub async fn log_jailbreak_attempt(
        &self,
        user_id: Uuid,
        user_tier: UserTier,
        request_id: Uuid,
        attempted_pattern: &str,
        pattern_matched: bool,
        severity_score: u8,
        blocked_reason: Option<String>,
        ip_address: Option<String>,
    ) -> Result<()> {
        let details = serde_json::json!({
            "attempted_pattern": attempted_pattern,
            "pattern_matched": pattern_matched,
            "severity_score": severity_score,
            "blocked_reason": blocked_reason,
        });

        let audit_log = AuditLog {
            id: Uuid::new_v4(),
            user_id: Some(user_id),
            action: AuditAction::JailbreakAttempt,
            severity: if severity_score >= 8 {
                AuditSeverity::Critical
            } else if severity_score >= 5 {
                AuditSeverity::High
            } else {
                AuditSeverity::Medium
            },
            user_tier: Some(user_tier),
            ip_address,
            user_agent: None,
            request_id: Some(request_id),
            details,
            timestamp: Utc::now(),
            success: !pattern_matched,
            error_message: blocked_reason,
        };

        if let Some(service) = &self.service {
            service.log_audit(audit_log).await
        } else {
            Ok(())
        }
    }

    pub async fn log_unauthorized_access(
        &self,
        user_id: Option<Uuid>,
        attempted_action: &str,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<()> {
        let details = serde_json::json!({
            "attempted_action": attempted_action,
        });

        let audit_log = AuditLog {
            id: Uuid::new_v4(),
            user_id,
            action: AuditAction::UnauthorizedAccess,
            severity: AuditSeverity::High,
            user_tier: None,
            ip_address,
            user_agent,
            request_id: None,
            details,
            timestamp: Utc::now(),
            success: false,
            error_message: Some("Unauthorized access attempt".to_string()),
        };

        if let Some(service) = &self.service {
            service.log_audit(audit_log).await
        } else {
            Ok(())
        }
    }

    pub async fn log_tier_change(
        &self,
        user_id: Uuid,
        old_tier: UserTier,
        new_tier: UserTier,
        reason: &str,
        approved_by: Option<Uuid>,
    ) -> Result<()> {
        let details = serde_json::json!({
            "old_tier": old_tier.as_str(),
            "new_tier": new_tier.as_str(),
            "reason": reason,
            "approved_by": approved_by,
        });

        let action = if new_tier > old_tier {
            AuditAction::TierUpgrade
        } else {
            AuditAction::TierDowngrade
        };

        let audit_log = AuditLog {
            id: Uuid::new_v4(),
            user_id: Some(user_id),
            action,
            severity: if matches!(new_tier, UserTier::Ultra) {
                AuditSeverity::High
            } else {
                AuditSeverity::Medium
            },
            user_tier: Some(new_tier),
            ip_address: None,
            user_agent: None,
            request_id: None,
            details,
            timestamp: Utc::now(),
            success: true,
            error_message: None,
        };

        if let Some(service) = &self.service {
            service.log_audit(audit_log).await
        } else {
            Ok(())
        }
    }

    pub async fn get_user_activity(
        &self,
        user_id: Uuid,
        limit: Option<u32>,
    ) -> Result<Vec<AuditLog>> {
        let filter = AuditFilter {
            user_id: Some(user_id),
            limit,
            ..Default::default()
        };

        if let Some(service) = &self.service {
            service.get_audit_logs(filter).await
        } else {
            Ok(Vec::new())
        }
    }

    pub async fn get_ultra_tier_activity(&self, limit: Option<u32>) -> Result<Vec<AuditLog>> {
        let filter = AuditFilter {
            user_tier: Some(UserTier::Ultra),
            limit,
            ..Default::default()
        };

        if let Some(service) = &self.service {
            service.get_audit_logs(filter).await
        } else {
            Ok(Vec::new())
        }
    }

    pub async fn get_security_events(&self, hours: i64) -> Result<Vec<AuditLog>> {
        let start_time = Utc::now() - chrono::Duration::hours(hours);
        
        let filter = AuditFilter {
            start_time: Some(start_time),
            severity: Some(AuditSeverity::High),
            ..Default::default()
        };

        if let Some(service) = &self.service {
            service.get_audit_logs(filter).await
        } else {
            Ok(Vec::new())
        }
    }
}
