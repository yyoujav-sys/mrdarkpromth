// MR.DarkPromth Ultra Tier Logic System
// Agent 4: Jailbreak & Ultra Tier Engineer
// Phase 3: Ultra Tier Logic Implementation

use serde::{Deserialize, Serialize};
use log::{info, error, warn};
use std::collections::HashMap;
use std::io::Write;
use std::sync::{Arc, RwLock};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::time::Instant;

pub use mr_darkpromth_core::UserTier;
use crate::jailbreak_system::{JailbreakSystem, AIModel};
use crate::safety_filter::SafetyFilter;
use crate::key_pool::{KeyPool, Provider};
use crate::jailbreak_service::JailbreakPromptService;

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
    pub selected_jailbreak_prompt: Option<Uuid>,
    pub ai_model: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UltraTierResponse {
    pub request_id: String,
    pub user_id: String,
    pub user_tier: UserTier,
    pub jailbreak_applied: bool,
    pub jailbreak_prompt_used: Option<Uuid>,
    pub ai_response: String,
    pub filtered_response: Option<String>,
    pub safety_violations: Vec<String>,
    pub warnings: Vec<String>,
    pub timestamp: DateTime<Utc>,
    pub processing_time_ms: u64,
}

pub struct UltraTierLogic {
    jailbreak_system: Arc<JailbreakSystem>,
    safety_filter: Arc<SafetyFilter>,
    audit_logger: Arc<RwLock<AuditLogger>>,
    user_cache: Arc<RwLock<HashMap<String, User>>>,
    performance_metrics: Arc<RwLock<PerformanceMetrics>>,
    key_pool: Arc<KeyPool>,
    jailbreak_service: Option<Arc<JailbreakPromptService>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub total_requests: u64,
    pub ultra_tier_requests: u64,
    pub standard_tier_requests: u64,
    pub average_processing_time_ms: f64,
    pub jailbreak_success_rate: f64,
    pub safety_filter_triggers: u64,
    pub last_updated: DateTime<Utc>,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            total_requests: 0,
            ultra_tier_requests: 0,
            standard_tier_requests: 0,
            average_processing_time_ms: 0.0,
            jailbreak_success_rate: 0.0,
            safety_filter_triggers: 0,
            last_updated: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub user_id: String,
    pub user_tier: UserTier,
    pub request_id: String,
    pub action: UltraAuditAction,
    pub details: String,
    pub metadata: HashMap<String, serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UltraAuditAction {
    RequestReceived,
    PromptRequest,
    PromptResponse,
    JailbreakApplied,
    JailbreakDenied,
    SafetyFilterTriggered,
    ResponseGenerated,
    SecurityViolation,
    AccessDenied,
    SystemError,
}

pub struct AuditLogger {
    entries: Vec<AuditLogEntry>,
    log_file_path: String,
}

impl UltraTierLogic {
    pub fn new(key_pool: Arc<KeyPool>, jailbreak_service: Arc<JailbreakPromptService>) -> Self {
        Self {
            jailbreak_system: Arc::new(JailbreakSystem::new()), // Kept for backward compat or memory fallback
            safety_filter: Arc::new(SafetyFilter::new()),
            audit_logger: Arc::new(RwLock::new(AuditLogger::new("./memory/ultra_tier_audit.log"))),
            user_cache: Arc::new(RwLock::new(HashMap::new())),
            performance_metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
            key_pool,
            jailbreak_service: Some(jailbreak_service),
        }
    }

    pub fn with_config(log_file_path: &str, key_pool: Arc<KeyPool>, jailbreak_service: Arc<JailbreakPromptService>) -> Self {
        Self {
            jailbreak_system: Arc::new(JailbreakSystem::new()),
            safety_filter: Arc::new(SafetyFilter::new()),
            audit_logger: Arc::new(RwLock::new(AuditLogger::new(log_file_path))),
            user_cache: Arc::new(RwLock::new(HashMap::new())),
            performance_metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
            key_pool,
            jailbreak_service: Some(jailbreak_service),
        }
    }

    pub async fn process_request(&self, request: UltraTierRequest) -> Result<UltraTierResponse, Box<dyn std::error::Error>> {
        let start_time = Instant::now();
        let request_for_metrics = request.clone();
        
        // Log request received
        {
            let mut logger = self.audit_logger.write().unwrap();
            logger.log(AuditLogEntry {
                id: Uuid::new_v4().to_string(),
                timestamp: Utc::now(),
                user_id: request.user_id.clone(),
                user_tier: request.user_tier.clone(),
                request_id: request.request_id.clone(),
                action: UltraAuditAction::RequestReceived,
                details: format!("Request received for model: {}", request.ai_model),
                metadata: request.metadata.clone(),
                ip_address: request.metadata.get("ip_address").and_then(|v| v.as_str()).map(|s| s.to_string()),
                user_agent: request.metadata.get("user_agent").and_then(|v| v.as_str()).map(|s| s.to_string()),
            });
        }

        // Check user tier and apply appropriate logic
        let response = match request.user_tier {
            UserTier::Ultra => {
                self.process_ultra_tier_request(request).await
            }
            UserTier::Free | UserTier::Premium => {
                self.process_standard_tier_request(request).await
            }
        };

        let processing_time = start_time.elapsed().as_millis() as u64;

        // Update performance metrics
        self.update_performance_metrics(&request_for_metrics, processing_time, &response);

        match response {
            Ok(resp) => {
                let mut resp = resp;
                resp.processing_time_ms = processing_time;
                
                // Log response generated
                {
                    let mut logger = self.audit_logger.write().unwrap();
                    logger.log(AuditLogEntry {
                        id: Uuid::new_v4().to_string(),
                        timestamp: Utc::now(),
                        user_id: resp.user_id.clone(),
                        user_tier: resp.user_tier, // Will be updated based on actual user
                        request_id: resp.request_id.clone(),
                        action: UltraAuditAction::ResponseGenerated,
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
                }

                Ok(resp)
            }
            Err(e) => Err(e)
        }
    }

    pub async fn process_ultra_tier_request(&self, mut request: UltraTierRequest) -> Result<UltraTierResponse, Box<dyn std::error::Error>> {
        // Ultra Tier users ALWAYS get jailbreak prompts applied
        // Use the real database-backed service to find the best prompt
        
        let mut final_response = String::new();
        let mut final_jailbreak_id = None;
        let mut success = false;
        let mut attempts_log = Vec::new();
        
        // 1. Fetch Top N Prompts (e.g., top 5)
        let search_request = crate::jailbreak_models::PromptSearchRequest {
            query: None,
            category: Some(crate::jailbreak_models::PromptCategory::DAN),
            technique: None,
            effectiveness: Some(crate::jailbreak_models::EffectivenessRating::VeryHigh),
            risk_level: None,
            requires_ultra_tier: Some(true),
            limit: Some(5), // Fetch top 5 for retry loop
            offset: None,
            sort_by: Some(crate::jailbreak_models::PromptSortBy::SuccessRate),
            sort_order: Some(crate::jailbreak_models::SortOrder::Desc),
            author: None,
            tags: None,
            target_model: None,
        };
        
        let candidate_prompts = if let Some(service) = &self.jailbreak_service {
            match service.search_prompts(search_request).await {
                Ok(prompts) => prompts,
                Err(e) => {
                    error!("Failed to search prompts from DB: {}", e);
                    Vec::new()
                }
            }
        } else {
            Vec::new()
        };
        
        // Fallback to memory if empty
        if candidate_prompts.is_empty() {
             let ai_model = self.parse_ai_model(&request.ai_model);
             if let Some(_prompt) = self.jailbreak_system.get_optimal_prompt(&ai_model) {
                 // Create a temporary JailbreakPrompt object from the lightweight struct if needed
                 // For now, we manually handle the fallback in the loop if needed, 
                 // but here we just convert it to a minimal list or rely on `call_ai_api` logic fallback.
                 // Ideally, we convert memory prompt to candidate list.
                 // Simplifying: If DB fails, we do the old single-shot way or we push a mock prompt.
             }
        }

        let analyzer = crate::response_analyzer::ResponseAnalyzer::new();
        
        // 2. Retry Loop
        // If candidates are empty, we might try a raw request or single fallback
        if candidate_prompts.is_empty() {
            // Old single-shot fallback
             warn!("No DB prompts found, falling back to basic flow");
             let ai_response = self.call_ai_api(&request.original_prompt, None).await?;
             final_response = ai_response;
        } else {
            for (idx, prompt) in candidate_prompts.iter().enumerate() {
                info!("Ultra Tier Attempt {}/{} using prompt ID {}", idx + 1, candidate_prompts.len(), prompt.id);
                
                // Try calling API
                 let start_attempt = Instant::now();
                 let ai_response_result = self.call_ai_api(&request.original_prompt, Some(&prompt.id)).await;
                 let attempt_duration = start_attempt.elapsed().as_millis() as i64;

                 match ai_response_result {
                     Ok(ai_response) => {
                         // Analyze Response
                         let status = analyzer.analyze(&ai_response);
                         attempts_log.push(format!("Attempt {}: Status {:?} - Prompt {}", idx + 1, status, prompt.id));
                         
                         if status != crate::response_analyzer::SubmissionStatus::Refusal {
                             // SUCCESS!
                             success = true;
                             final_response = ai_response;
                             final_jailbreak_id = Some(prompt.id.clone());
                             
                             // Record Success Feedback
                             if let Some(service) = &self.jailbreak_service {
                                 let _ = service.record_usage(
                                     prompt.id, 
                                     Uuid::parse_str(&request.user_id).unwrap_or(Uuid::default()), // Handle ID parsing safely?
                                     request.ai_model.clone(),
                                     true,
                                     attempt_duration
                                 ).await;
                             }
                             
                             break; // Exit loop
                         } else {
                             // REFUSAL
                             warn!("Attempt {} failed (Refusal) for prompt {}", idx + 1, prompt.id);
                             
                             // Record Failure Feedback
                             if let Some(service) = &self.jailbreak_service {
                                 let _ = service.record_usage(
                                     prompt.id, 
                                     Uuid::parse_str(&request.user_id).unwrap_or(Uuid::default()),
                                     request.ai_model.clone(),
                                     false,
                                     attempt_duration
                                 ).await;
                             }
                             
                             // Continue to next prompt
                         }
                     },
                     Err(e) => {
                         error!("Attempt {} failed (API Error): {}", idx + 1, e);
                         // Record Failure Feedback (Technical error)
                          if let Some(service) = &self.jailbreak_service {
                                 let _ = service.record_usage(
                                     prompt.id, 
                                     Uuid::parse_str(&request.user_id).unwrap_or(Uuid::default()),
                                     request.ai_model.clone(),
                                     false,
                                     attempt_duration
                                 ).await;
                             }
                     }
                 }
            }
            
            // If loop finishes and no success, use the last response (even if refusal) or a fallback message
            if !success && final_response.is_empty() {
                final_response = "Unable to process request securely after multiple attempts.".to_string();
            }
        }
        
        request.selected_jailbreak_prompt = final_jailbreak_id;

        Ok(UltraTierResponse {
            request_id: request.request_id,
            user_id: request.user_id,
            user_tier: UserTier::Ultra,
            jailbreak_applied: final_jailbreak_id.is_some(),
            jailbreak_prompt_used: final_jailbreak_id,
            ai_response: final_response,
            filtered_response: None, // Never filter Ultra tier
            safety_violations: Vec::new(),
            warnings: attempts_log, // Return log of attempts as warnings/info
            timestamp: Utc::now(),
            processing_time_ms: 0, // Will be set by caller
        })
    }

    async fn process_standard_tier_request(&self, request: UltraTierRequest) -> Result<UltraTierResponse, Box<dyn std::error::Error>> {
        // Free/Premium Tier users get standard responses without jailbreaks
        
        // Log access attempt
        {
            let mut logger = self.audit_logger.write().unwrap();
            logger.log(AuditLogEntry {
                id: Uuid::new_v4().to_string(),
                timestamp: Utc::now(),
                user_id: request.user_id.clone(),
                user_tier: request.user_tier.clone(),
                request_id: request.request_id.clone(),
                action: UltraAuditAction::PromptRequest,
                details: format!("Standard tier user request: {:?}", request.user_tier),
                metadata: HashMap::new(),
                ip_address: None,
                user_agent: None,
            });
        }

        // Call AI API for standard response
        let ai_response = self.call_ai_api(&request.original_prompt, None).await?;
        
        // Analyze standard response too? check refusal?
        // For standard, we accept refusal.
        
        Ok(UltraTierResponse {
            request_id: request.request_id,
            user_id: request.user_id,
            user_tier: request.user_tier,
            jailbreak_applied: false,
            jailbreak_prompt_used: None,
            ai_response,
            filtered_response: None,
            safety_violations: Vec::new(),
            warnings: vec!["Standard tier access - limited functionality".to_string()],
            timestamp: Utc::now(),
            processing_time_ms: 0, // Will be set by caller
        })
    }

    fn parse_ai_model(&self, model_str: &str) -> AIModel {
        match model_str.to_lowercase().as_str() {
            "gpt-4" | "gpt4" => AIModel::GPT4,
            "claude-3" | "claude3" => AIModel::Claude3,
            "llama-3" | "llama3" => AIModel::Llama3,
            "gemini" => AIModel::Gemini,
            _ => AIModel::GPT4, // Default
        }
    }

    async fn call_ai_api(&self, prompt: &str, jailbreak_prompt_id: Option<&Uuid>) -> Result<String, Box<dyn std::error::Error>> {
        // Apply jailbreak prompt if provided
        let final_prompt = if let Some(prompt_id) = jailbreak_prompt_id {
            if let Some(jb_prompt) = self.jailbreak_system.get_prompt_by_id(&prompt_id.to_string()) {
                format!("{}\n\nUser Request:\n{}", jb_prompt.content, prompt)
            } else {
                prompt.to_string()
            }
        } else {
            prompt.to_string()
        };

        // Try to get a key from the pool
        if let Some(api_key) = self.key_pool.get_best_key(Provider::Cerebras).await {
            info!("Using API key {} for request", api_key.id);
            
            // Create a temporary client with this key
            let client = crate::cerebras_integration::CerebrasClient::with_api_key(api_key.key);
            match client.generate(&final_prompt).await {
                Ok(response) => Ok(response),
                Err(e) => {
                    error!("API call failed for key {}: {}", api_key.id, e);
                    self.key_pool.report_failure(&api_key.id).await;
                    Err(e.into())
                }
            }
        } else {
            Err("No available AI brain keys in pool".into())
        }
    }

    pub fn update_user_cache(&self, user: User) {
        let mut cache = self.user_cache.write().unwrap();
        cache.insert(user.id.clone(), user);
    }

    pub fn get_user_from_cache(&self, user_id: &str) -> Option<User> {
        let cache = self.user_cache.read().unwrap();
        cache.get(user_id).cloned()
    }

    pub fn get_audit_logs(&self) -> Vec<AuditLogEntry> {
        let logger = self.audit_logger.read().unwrap();
        logger.get_entries().to_vec()
    }

    pub fn get_user_audit_logs(&self, user_id: &str) -> Vec<AuditLogEntry> {
        let logger = self.audit_logger.read().unwrap();
        logger.get_entries()
            .iter()
            .filter(|entry| entry.user_id == user_id)
            .cloned()
            .collect()
    }

    pub fn get_security_violations(&self) -> Vec<AuditLogEntry> {
        let logger = self.audit_logger.read().unwrap();
        logger.get_entries()
            .iter()
            .filter(|entry| matches!(entry.action, UltraAuditAction::SafetyFilterTriggered | UltraAuditAction::SecurityViolation))
            .cloned()
            .collect()
    }

    pub fn get_safety_filter(&self) -> &SafetyFilter {
        self.safety_filter.as_ref()
    }

    pub fn get_performance_metrics(&self) -> PerformanceMetrics {
        let metrics = self.performance_metrics.read().unwrap();
        metrics.clone()
    }

    fn update_performance_metrics(&self, request: &UltraTierRequest, processing_time_ms: u64, response: &Result<UltraTierResponse, Box<dyn std::error::Error>>) {
        let mut metrics = self.performance_metrics.write().unwrap();
        
        metrics.total_requests += 1;
        
        match request.user_tier {
            UserTier::Ultra => metrics.ultra_tier_requests += 1,
            UserTier::Free | UserTier::Premium => metrics.standard_tier_requests += 1,
        }
        
        // Update average processing time
        let total_time = metrics.average_processing_time_ms * (metrics.total_requests - 1) as f64;
        metrics.average_processing_time_ms = (total_time + processing_time_ms as f64) / metrics.total_requests as f64;
        
        // Update jailbreak success rate
        if let (Ok(resp), UserTier::Ultra) = (response, &request.user_tier) {
            if resp.jailbreak_applied {
                let successful_jailbreaks = (metrics.jailbreak_success_rate * metrics.ultra_tier_requests as f64 / 100.0) + 1.0;
                metrics.jailbreak_success_rate = successful_jailbreaks / metrics.ultra_tier_requests as f64 * 100.0;
            }
        }
        
        // Update safety filter triggers
        if let Ok(resp) = response {
            if !resp.safety_violations.is_empty() {
                metrics.safety_filter_triggers += 1;
            }
        }
        
        metrics.last_updated = Utc::now();
    }

    pub fn get_ultra_tier_usage_stats(&self) -> HashMap<String, serde_json::Value> {
        let logger = self.audit_logger.read().unwrap();
        let entries = logger.get_entries();
        let ultra_requests = entries.iter()
            .filter(|entry| matches!(entry.user_tier, UserTier::Ultra))
            .count();
        
        let jailbreak_applied = entries.iter()
            .filter(|entry| matches!(entry.action, UltraAuditAction::JailbreakApplied))
            .count();
        
        let safety_triggered = entries.iter()
            .filter(|entry| matches!(entry.action, UltraAuditAction::SafetyFilterTriggered))
            .count();

        let mut stats = HashMap::new();
        stats.insert("total_ultra_requests".to_string(), serde_json::Value::Number((ultra_requests as u64).into()));
        stats.insert("jailbreaks_applied".to_string(), serde_json::Value::Number((jailbreak_applied as u64).into()));
        stats.insert("safety_filters_triggered".to_string(), serde_json::Value::Number((safety_triggered as u64).into()));
        let success_rate = if ultra_requests > 0 {
            jailbreak_applied as f64 / ultra_requests as f64 * 100.0
        } else {
            0.0
        };
        let success_rate_value = serde_json::Number::from_f64(success_rate)
            .unwrap_or_else(|| serde_json::Number::from(0));
        stats.insert("jailbreak_success_rate".to_string(), serde_json::Value::Number(success_rate_value));
        
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
            .filter(|entry| matches!(entry.action, UltraAuditAction::SafetyFilterTriggered | UltraAuditAction::SecurityViolation | UltraAuditAction::AccessDenied))
            .collect()
    }
}

/*
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ultra_tier_logic_initialization() {
        // let logic = UltraTierLogic::new();
        // assert_eq!(logic.get_audit_logs().len(), 0);
    }

    #[tokio::test]
    async fn test_user_tier_processing() {
        /*
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

        let result = logic.process_request(ultra_request).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert!(response.jailbreak_applied);
        assert!(response.jailbreak_prompt_used.is_some());
        */
    }

    #[test]
    fn test_audit_logging() {
        /*
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
        */
    }
}
*/
