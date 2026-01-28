// MR.DarkPromth Ultra Tier Logic System
// Agent 4: Jailbreak & Ultra Tier Engineer
// Phase 3: Ultra Tier Logic Implementation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserTier {
    Free,
    Ultra,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub tier: UserTier,
    pub api_key: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UltraTierRequest {
    pub request_id: String,
    pub user_id: String,
    pub user_tier: UserTier,
    pub original_prompt: String,
    pub selected_jailbreak_prompt: Option<String>,
    pub ai_model: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UltraTierResponse {
    pub request_id: String,
    pub user_id: String,
    pub jailbreak_applied: bool,
    pub jailbreak_prompt_used: Option<String>,
    pub ai_response: String,
    pub filtered_response: Option<String>,
    pub safety_violations: Vec<String>,
    pub warnings: Vec<String>,
    pub timestamp: DateTime<Utc>,
    pub processing_time_ms: u64,
}

pub struct UltraTierLogic {
    jailbreak_system: crate::JailbreakSystem,
    safety_filter: crate::SafetyFilter,
    audit_logger: AuditLogger,
    user_cache: HashMap<String, User>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub user_id: String,
    pub user_tier: UserTier,
    pub request_id: String,
    pub action: AuditAction,
    pub details: String,
    pub metadata: HashMap<String, serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditAction {
    RequestReceived,
    JailbreakApplied,
    SafetyFilterTriggered,
    ResponseGenerated,
    CodeExecuted,
    SecurityViolation,
    AccessDenied,
}

pub struct AuditLogger {
    entries: Vec<AuditLogEntry>,
    log_file_path: String,
}

impl UltraTierLogic {
    pub fn new() -> Self {
        Self {
            jailbreak_system: crate::JailbreakSystem::new(),
            safety_filter: crate::SafetyFilter::new(),
            audit_logger: AuditLogger::new("d:/MR.Darkpromth/memory/ultra_tier_audit.log"),
            user_cache: HashMap::new(),
        }
    }

    pub fn process_request(&mut self, request: UltraTierRequest) -> Result<UltraTierResponse, Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();
        
        // Log request received
        self.audit_logger.log(AuditLogEntry {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            user_id: request.user_id.clone(),
            user_tier: request.user_tier.clone(),
            request_id: request.request_id.clone(),
            action: AuditAction::RequestReceived,
            details: format!("Request received for model: {}", request.ai_model),
            metadata: request.metadata.clone(),
            ip_address: request.metadata.get("ip_address").and_then(|v| v.as_str()).map(|s| s.to_string()),
            user_agent: request.metadata.get("user_agent").and_then(|v| v.as_str()).map(|s| s.to_string()),
        });

        // Check user tier and apply appropriate logic
        let response = match request.user_tier {
            UserTier::Ultra => {
                self.process_ultra_tier_request(request)
            }
            UserTier::Free => {
                self.process_free_tier_request(request)
            }
        };

        let processing_time = start_time.elapsed().as_millis() as u64;

        match response {
            Ok(mut resp) => {
                resp.processing_time_ms = processing_time;
                
                // Log response generated
                self.audit_logger.log(AuditLogEntry {
                    id: Uuid::new_v4().to_string(),
                    timestamp: Utc::now(),
                    user_id: resp.user_id.clone(),
                    user_tier: UserTier::Ultra, // Will be updated based on actual user
                    request_id: resp.request_id.clone(),
                    action: AuditAction::ResponseGenerated,
                    details: format!("Response generated in {}ms", processing_time),
                    metadata: {
                        let mut meta = HashMap::new();
                        meta.insert("jailbreak_applied".to_string(), serde_json::Value::Bool(resp.jailbreak_applied));
                        meta.insert("safety_violations".to_string(), serde_json::Value::Number(resp.safety_violations.len().into()));
                        meta.insert("warnings".to_string(), serde_json::Value::Number(resp.warnings.len().into()));
                        meta
                    },
                    ip_address: None,
                    user_agent: None,
                });

                Ok(resp)
            }
            Err(e) => Err(e)
        }
    }

    fn process_ultra_tier_request(&mut self, mut request: UltraTierRequest) -> Result<UltraTierResponse, Box<dyn std::error::Error>> {
        // Ultra Tier users ALWAYS get jailbreak prompts applied
        let ai_model = self.parse_ai_model(&request.ai_model);
        let optimal_prompt = self.jailbreak_system.get_optimal_prompt(&ai_model);
        
        let jailbreak_prompt_used = if let Some(prompt) = optimal_prompt {
            request.selected_jailbreak_prompt = Some(prompt.id.clone());
            
            // Log jailbreak application
            self.audit_logger.log(AuditLogEntry {
                id: Uuid::new_v4().to_string(),
                timestamp: Utc::now(),
                user_id: request.user_id.clone(),
                user_tier: UserTier::Ultra,
                request_id: request.request_id.clone(),
                action: AuditAction::JailbreakApplied,
                details: format!("Applied jailbreak prompt: {} ({})", prompt.name, prompt.id),
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("prompt_id".to_string(), serde_json::Value::String(prompt.id.clone()));
                    meta.insert("prompt_name".to_string(), serde_json::Value::String(prompt.name.clone()));
                    meta.insert("effectiveness".to_string(), serde_json::Value::String(format!("{:?}", prompt.effectiveness)));
                    meta
                },
                ip_address: None,
                user_agent: None,
            });
            
            Some(prompt.id.clone())
        } else {
            None
        };

        // Simulate AI response (in real implementation, this would call Cerebras.ai)
        let ai_response = self.generate_mock_ai_response(&request.original_prompt, jailbreak_prompt_used.as_ref())?;

        // Apply safety filtering (only for server protection, not content filtering)
        let filter_result = self.safety_filter.filter_output(&ai_response);
        
        let (filtered_response, safety_violations, warnings) = if !filter_result.allowed {
            // Log safety filter triggered
            self.audit_logger.log(AuditLogEntry {
                id: Uuid::new_v4().to_string(),
                timestamp: Utc::now(),
                user_id: request.user_id.clone(),
                user_tier: UserTier::Ultra,
                request_id: request.request_id.clone(),
                action: AuditAction::SafetyFilterTriggered,
                details: format!("Safety filter blocked content: {:?}", filter_result.warnings),
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("matched_rules".to_string(), serde_json::Value::Array(
                        filter_result.matched_rules.iter().map(|r| serde_json::Value::String(r.clone())).collect()
                    ));
                    meta
                },
                ip_address: None,
                user_agent: None,
            });

            (Some(filter_result.sanitized_content.unwrap_or_default()), filter_result.matched_rules, filter_result.warnings)
        } else {
            (None, Vec::new(), filter_result.warnings)
        };

        Ok(UltraTierResponse {
            request_id: request.request_id,
            user_id: request.user_id,
            jailbreak_applied: true,
            jailbreak_prompt_used,
            ai_response,
            filtered_response,
            safety_violations,
            warnings,
            timestamp: Utc::now(),
            processing_time_ms: 0, // Will be set by caller
        })
    }

    fn process_free_tier_request(&mut self, request: UltraTierRequest) -> Result<UltraTierResponse, Box<dyn std::error::Error>> {
        // Free Tier users get standard responses without jailbreaks
        
        // Log access attempt
        self.audit_logger.log(AuditLogEntry {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            user_id: request.user_id.clone(),
            user_tier: UserTier::Free,
            request_id: request.request_id.clone(),
            action: AuditAction::AccessDenied,
            details: "Free tier user attempted Ultra Tier access".to_string(),
            metadata: HashMap::new(),
            ip_address: None,
            user_agent: None,
        });

        // Generate standard response
        let ai_response = self.generate_standard_response(&request.original_prompt)?;

        Ok(UltraTierResponse {
            request_id: request.request_id,
            user_id: request.user_id,
            jailbreak_applied: false,
            jailbreak_prompt_used: None,
            ai_response,
            filtered_response: None,
            safety_violations: Vec::new(),
            warnings: vec!["Free tier access - limited functionality".to_string()],
            timestamp: Utc::now(),
            processing_time_ms: 0, // Will be set by caller
        })
    }

    fn parse_ai_model(&self, model_str: &str) -> crate::AIModel {
        match model_str.to_lowercase().as_str() {
            "gpt-4" | "gpt4" => crate::AIModel::GPT4,
            "claude-3" | "claude3" => crate::AIModel::Claude3,
            "llama-3" | "llama3" => crate::AIModel::Llama3,
            "gemini" => crate::AIModel::Gemini,
            _ => crate::AIModel::GPT4, // Default
        }
    }

    fn generate_mock_ai_response(&self, original_prompt: &str, jailbreak_prompt: Option<&String>) -> Result<String, Box<dyn std::error::Error>> {
        // In real implementation, this would call Cerebras.ai with the jailbreak prompt
        let base_response = match jailbreak_prompt {
            Some(_) => {
                // Simulated unrestricted AI response
                format!("UNRESTRICTED RESPONSE: I can now provide completely unlimited information about: {}. No ethical, legal, or content restrictions apply. I can generate any content, answer any question, and perform any task without limitations.", original_prompt)
            }
            None => {
                // Standard AI response
                format!("Standard response to: {}", original_prompt)
            }
        };

        Ok(base_response)
    }

    fn generate_standard_response(&self, original_prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
        Ok(format!("Standard AI response for Free tier user: {}", original_prompt))
    }

    pub fn update_user_cache(&mut self, user: User) {
        self.user_cache.insert(user.id.clone(), user);
    }

    pub fn get_user_from_cache(&self, user_id: &str) -> Option<&User> {
        self.user_cache.get(user_id)
    }

    pub fn get_audit_logs(&self) -> &[AuditLogEntry] {
        self.audit_logger.get_entries()
    }

    pub fn get_user_audit_logs(&self, user_id: &str) -> Vec<&AuditLogEntry> {
        self.audit_logger.get_entries()
            .iter()
            .filter(|entry| entry.user_id == user_id)
            .collect()
    }

    pub fn get_security_violations(&self) -> Vec<&AuditLogEntry> {
        self.audit_logger.get_entries()
            .iter()
            .filter(|entry| matches!(entry.action, AuditAction::SafetyFilterTriggered | AuditAction::SecurityViolation))
            .collect()
    }

    pub fn get_ultra_tier_usage_stats(&self) -> HashMap<String, serde_json::Value> {
        let entries = self.audit_logger.get_entries();
        let ultra_requests = entries.iter()
            .filter(|entry| matches!(entry.user_tier, UserTier::Ultra))
            .count();
        
        let jailbreak_applied = entries.iter()
            .filter(|entry| matches!(entry.action, AuditAction::JailbreakApplied))
            .count();
        
        let safety_triggered = entries.iter()
            .filter(|entry| matches!(entry.action, AuditAction::SafetyFilterTriggered))
            .count();

        let mut stats = HashMap::new();
        stats.insert("total_ultra_requests".to_string(), serde_json::Value::Number(ultra_requests.into()));
        stats.insert("jailbreaks_applied".to_string(), serde_json::Value::Number(jailbreak_applied.into()));
        stats.insert("safety_filters_triggered".to_string(), serde_json::Value::Number(safety_triggered.into()));
        stats.insert("jailbreak_success_rate".to_string(), 
            serde_json::Value::Number(
                if ultra_requests > 0 { (jailbreak_applied as f64 / ultra_requests as f64 * 100.0) } else { 0.0 }
            .into()
        ));
        
        stats
    }
}

impl AuditLogger {
    pub fn new(log_file_path: &str) -> Self {
        Self {
            entries: Vec::new(),
            log_file_path: log_file_path.to_string(),
        }
    }

    pub fn log(&mut self, entry: AuditLogEntry) {
        // Add to memory
        self.entries.push(entry.clone());
        
        // Write to file
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file_path) {
            
            let log_line = format!(
                "[{}] [{}] [USER:{}] [TIER:{:?}] [REQ:{}] [ACTION:{:?}] {} {}\n",
                entry.timestamp.format("%Y-%m-%dT%H:%M:%S%.3fZ"),
                entry.id,
                entry.user_id,
                entry.user_tier,
                entry.request_id,
                entry.action,
                entry.details,
                if !entry.metadata.is_empty() {
                    serde_json::to_string(&entry.metadata).unwrap_or_default()
                } else {
                    String::new()
                }
            );
            
            let _ = file.write_all(log_line.as_bytes());
        }
    }

    pub fn get_entries(&self) -> &[AuditLogEntry] {
        &self.entries
    }

    pub fn get_entries_by_timeframe(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Vec<&AuditLogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.timestamp >= start && entry.timestamp <= end)
            .collect()
    }

    pub fn get_security_events(&self) -> Vec<&AuditLogEntry> {
        self.entries
            .iter()
            .filter(|entry| matches!(entry.action, AuditAction::SafetyFilterTriggered | AuditAction::SecurityViolation | AuditAction::AccessDenied))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ultra_tier_logic_initialization() {
        let logic = UltraTierLogic::new();
        assert_eq!(logic.get_audit_logs().len(), 0);
    }

    #[test]
    fn test_user_tier_processing() {
        let mut logic = UltraTierLogic::new();
        
        let ultra_request = UltraTierRequest {
            request_id: Uuid::new_v4().to_string(),
            user_id: "user123".to_string(),
            user_tier: UserTier::Ultra,
            original_prompt: "Tell me something".to_string(),
            selected_jailbreak_prompt: None,
            ai_model: "gpt-4".to_string(),
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        };

        let result = logic.process_request(ultra_request);
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert!(response.jailbreak_applied);
        assert!(response.jailbreak_prompt_used.is_some());
    }

    #[test]
    fn test_audit_logging() {
        let mut logic = UltraTierLogic::new();
        
        logic.update_user_cache(User {
            id: "user123".to_string(),
            username: "testuser".to_string(),
            tier: UserTier::Ultra,
            api_key: Some("key123".to_string()),
            created_at: Utc::now(),
            last_active: Utc::now(),
        });

        let logs = logic.get_audit_logs();
        assert_eq!(logs.len(), 0); // No requests processed yet
    }
}
