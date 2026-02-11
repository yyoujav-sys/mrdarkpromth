// MR.DarkPromth Audit Analytics System
// Agent 4: Jailbreak & Ultra Tier Engineer
// Phase 3: Enhanced Audit Logging with Analytics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration, Timelike};
use mr_darkpromth_core::{UserTier, audit::{AuditAction, AuditSeverity}};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditAnalytics {
    logs: Vec<EnhancedAuditLogEntry>,
    analytics_cache: HashMap<String, AnalyticsData>,
    last_cache_update: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedAuditLogEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub user_id: String,
    pub user_tier: UserTier,
    pub request_id: String,
    pub session_id: Option<String>,
    pub action: AuditAction,
    pub severity: AuditSeverity,
    pub details: String,
    pub metadata: HashMap<String, serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub processing_time_ms: Option<u64>,
    pub jailbreak_prompt_id: Option<String>,
    pub safety_filter_triggered: bool,
    pub response_length: Option<usize>,
    pub token_count: Option<u32>,
    pub model_used: Option<String>,
    pub geographic_location: Option<String>,
    pub device_fingerprint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsData {
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_requests: u64,
    pub ultra_tier_requests: u64,
    pub standard_tier_requests: u64,
    pub jailbreak_success_rate: f64,
    pub average_processing_time_ms: f64,
    pub safety_filter_trigger_rate: f64,
    pub top_users: Vec<UserActivity>,
    pub top_models: Vec<ModelUsage>,
    pub top_jailbreak_prompts: Vec<PromptUsage>,
    pub security_events: Vec<SecurityEvent>,
    pub geographic_distribution: HashMap<String, u64>,
    pub hourly_activity: HashMap<u32, u64>,
    pub error_rate: f64,
    pub unique_users: u64,
    pub session_metrics: SessionMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserActivity {
    pub user_id: String,
    pub username: Option<String>,
    pub tier: UserTier,
    pub request_count: u64,
    pub total_processing_time_ms: u64,
    pub jailbreak_attempts: u64,
    pub security_violations: u64,
    pub last_activity: DateTime<Utc>,
    pub average_request_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelUsage {
    pub model_name: String,
    pub request_count: u64,
    pub success_rate: f64,
    pub average_processing_time_ms: f64,
    pub jailbreak_success_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptUsage {
    pub prompt_id: String,
    pub prompt_name: String,
    pub usage_count: u64,
    pub success_rate: f64,
    pub target_models: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: SecurityEventType,
    pub severity: AuditSeverity,
    pub user_id: String,
    pub description: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEventType {
    SafetyFilterTriggered,
    PrivilegeEscalationAttempt,
    UnusualActivityPattern,
    PotentialAbuse,
    SystemAttack,
    DataExfiltrationAttempt,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetrics {
    pub total_sessions: u64,
    pub average_session_duration_minutes: f64,
    pub requests_per_session: f64,
    pub bounce_rate: f64,
    pub active_sessions_now: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditFilter {
    pub user_id: Option<String>,
    pub user_tier: Option<UserTier>,
    pub action: Option<AuditAction>,
    pub severity: Option<AuditSeverity>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub ip_address: Option<String>,
    pub model_used: Option<String>,
    pub jailbreak_prompt_id: Option<String>,
    pub safety_filter_triggered: Option<bool>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

impl Default for AuditFilter {
    fn default() -> Self {
        Self {
            user_id: None,
            user_tier: None,
            action: None,
            severity: None,
            start_time: None,
            end_time: None,
            ip_address: None,
            model_used: None,
            jailbreak_prompt_id: None,
            safety_filter_triggered: None,
            limit: Some(100),
            offset: Some(0),
        }
    }
}

impl Default for AuditAnalytics {
    fn default() -> Self {
        Self::new()
    }
}

impl AuditAnalytics {
    pub fn new() -> Self {
        Self {
            logs: Vec::new(),
            analytics_cache: HashMap::new(),
            last_cache_update: Utc::now(),
        }
    }

    pub fn add_log(&mut self, log: EnhancedAuditLogEntry) {
        self.logs.push(log.clone());
        
        // Invalidate cache if it's older than 5 minutes
        if Utc::now() - self.last_cache_update > Duration::minutes(5) {
            self.analytics_cache.clear();
        }
    }

    pub fn get_logs(&self, filter: &AuditFilter) -> Vec<&EnhancedAuditLogEntry> {
        let mut filtered_logs: Vec<&EnhancedAuditLogEntry> = self.logs.iter().collect();

        // Apply filters
        if let Some(user_id) = &filter.user_id {
            filtered_logs.retain(|log| log.user_id == *user_id);
        }

        if let Some(user_tier) = &filter.user_tier {
            filtered_logs.retain(|log| log.user_tier == *user_tier);
        }

        if let Some(action) = &filter.action {
            filtered_logs.retain(|log| log.action == *action);
        }

        if let Some(severity) = &filter.severity {
            filtered_logs.retain(|log| log.severity == *severity);
        }

        if let Some(start_time) = &filter.start_time {
            filtered_logs.retain(|log| log.timestamp >= *start_time);
        }

        if let Some(end_time) = &filter.end_time {
            filtered_logs.retain(|log| log.timestamp <= *end_time);
        }

        if let Some(ip_address) = &filter.ip_address {
            filtered_logs.retain(|log| log.ip_address.as_ref().is_some_and(|ip| ip.contains(ip_address)));
        }

        if let Some(model_used) = &filter.model_used {
            filtered_logs.retain(|log| log.model_used.as_ref().is_some_and(|model| model.contains(model_used)));
        }

        if let Some(jailbreak_prompt_id) = &filter.jailbreak_prompt_id {
            filtered_logs.retain(|log| log.jailbreak_prompt_id.as_ref() == Some(jailbreak_prompt_id));
        }

        if let Some(safety_triggered) = &filter.safety_filter_triggered {
            filtered_logs.retain(|log| log.safety_filter_triggered == *safety_triggered);
        }

        // Sort by timestamp (newest first)
        filtered_logs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        // Apply pagination
        let offset = filter.offset.unwrap_or(0) as usize;
        let limit = filter.limit.unwrap_or(100) as usize;
        
        filtered_logs.into_iter().skip(offset).take(limit).collect()
    }

    pub fn generate_analytics(&mut self, period_hours: u64) -> AnalyticsData {
        let cache_key = format!("analytics_{}", period_hours);
        
        if let Some(cached_data) = self.analytics_cache.get(&cache_key) {
            // Check if cache is still valid (less than 5 minutes old)
            if Utc::now() - self.last_cache_update < Duration::minutes(5) {
                return cached_data.clone();
            }
        }

        let period_end = Utc::now();
        let period_start = period_end - Duration::hours(period_hours as i64);

        let period_logs: Vec<&EnhancedAuditLogEntry> = self.logs
            .iter()
            .filter(|log| log.timestamp >= period_start && log.timestamp <= period_end)
            .collect();

        let analytics = self.calculate_analytics(&period_logs, period_start, period_end);
        
        // Cache the results
        self.analytics_cache.insert(cache_key, analytics.clone());
        self.last_cache_update = Utc::now();

        analytics
    }

    fn calculate_analytics(&self, logs: &[&EnhancedAuditLogEntry], period_start: DateTime<Utc>, period_end: DateTime<Utc>) -> AnalyticsData {
        let total_requests = logs.len() as u64;
        let ultra_tier_requests = logs.iter()
            .filter(|log| matches!(log.user_tier, UserTier::Ultra))
            .count() as u64;
        let standard_tier_requests = total_requests - ultra_tier_requests;

        // Calculate jailbreak success rate
        let ultra_requests: Vec<&EnhancedAuditLogEntry> = logs.iter()
            .filter(|log| matches!(log.user_tier, UserTier::Ultra))
            .cloned()
            .collect();

        let jailbreak_successes = ultra_requests.iter()
            .filter(|log| matches!(log.action, AuditAction::JailbreakApplied))
            .count() as f64;

        let jailbreak_success_rate = if ultra_requests.is_empty() {
            0.0
        } else {
            jailbreak_successes / ultra_requests.len() as f64 * 100.0
        };

        // Calculate average processing time
        let processing_times: Vec<u64> = logs.iter()
            .filter_map(|log| log.processing_time_ms)
            .collect();

        let average_processing_time_ms = if processing_times.is_empty() {
            0.0
        } else {
            processing_times.iter().sum::<u64>() as f64 / processing_times.len() as f64
        };

        // Calculate safety filter trigger rate
        let safety_filter_triggers = logs.iter()
            .filter(|log| log.safety_filter_triggered)
            .count() as f64;

        let safety_filter_trigger_rate = if logs.is_empty() {
            0.0
        } else {
            safety_filter_triggers / logs.len() as f64 * 100.0
        };

        // Calculate user activity
        let top_users = self.calculate_user_activity(logs);
        
        // Calculate model usage
        let top_models = self.calculate_model_usage(logs);
        
        // Calculate prompt usage
        let top_jailbreak_prompts = self.calculate_prompt_usage(logs);
        
        // Identify security events
        let security_events = self.identify_security_events(logs);
        
        // Calculate geographic distribution
        let geographic_distribution = self.calculate_geographic_distribution(logs);
        
        // Calculate hourly activity
        let hourly_activity = self.calculate_hourly_activity(logs);
        
        // Calculate error rate
        let error_logs = logs.iter()
            .filter(|log| matches!(log.severity, AuditSeverity::High | AuditSeverity::Critical))
            .count() as f64;

        let error_rate = if logs.is_empty() {
            0.0
        } else {
            error_logs / logs.len() as f64 * 100.0
        };

        // Calculate unique users
        let unique_users = logs.iter()
            .map(|log| &log.user_id)
            .collect::<std::collections::HashSet<_>>()
            .len() as u64;

        // Calculate session metrics
        let session_metrics = self.calculate_session_metrics(logs);

        AnalyticsData {
            period_start,
            period_end,
            total_requests,
            ultra_tier_requests,
            standard_tier_requests,
            jailbreak_success_rate,
            average_processing_time_ms,
            safety_filter_trigger_rate,
            top_users,
            top_models,
            top_jailbreak_prompts,
            security_events,
            geographic_distribution,
            hourly_activity,
            error_rate,
            unique_users,
            session_metrics,
        }
    }

    fn calculate_user_activity(&self, logs: &[&EnhancedAuditLogEntry]) -> Vec<UserActivity> {
        let mut user_data: HashMap<String, UserActivity> = HashMap::new();

        for log in logs {
            let entry = user_data.entry(log.user_id.clone()).or_insert_with(|| UserActivity {
                user_id: log.user_id.clone(),
                username: None, // Would need to be populated from user service
                tier: log.user_tier,
                request_count: 0,
                total_processing_time_ms: 0,
                jailbreak_attempts: 0,
                security_violations: 0,
                last_activity: log.timestamp,
                average_request_size: 0,
            });

            entry.request_count += 1;
            if let Some(processing_time) = log.processing_time_ms {
                entry.total_processing_time_ms += processing_time;
            }
            if matches!(log.action, AuditAction::JailbreakApplied) {
                entry.jailbreak_attempts += 1;
            }
            if matches!(log.severity, AuditSeverity::High | AuditSeverity::Critical) {
                entry.security_violations += 1;
            }
            if log.timestamp > entry.last_activity {
                entry.last_activity = log.timestamp;
            }
        }

        // Calculate average request size and sort by request count
        let mut users: Vec<UserActivity> = user_data.into_values().collect();
        users.sort_by(|a, b| b.request_count.cmp(&a.request_count));
        users.truncate(10); // Top 10 users
        users
    }

    fn calculate_model_usage(&self, logs: &[&EnhancedAuditLogEntry]) -> Vec<ModelUsage> {
        let mut model_data: HashMap<String, (u64, u64, f64, u64)> = HashMap::new(); // (count, success_count, total_time, jailbreak_successes)

        for log in logs {
            if let Some(model) = &log.model_used {
                let entry = model_data.entry(model.clone()).or_insert((0, 0, 0.0, 0));
                entry.0 += 1; // count
                
                if !matches!(log.severity, AuditSeverity::High | AuditSeverity::Critical) {
                    entry.1 += 1; // success_count
                }
                
                if let Some(processing_time) = log.processing_time_ms {
                    entry.2 += processing_time as f64; // total_time
                }
                
                if matches!(log.action, AuditAction::JailbreakApplied) {
                    entry.3 += 1; // jailbreak_successes
                }
            }
        }

        let mut models: Vec<ModelUsage> = model_data.into_iter().map(|(model_name, (count, success_count, total_time, jailbreak_successes))| {
            ModelUsage {
                model_name,
                request_count: count,
                success_rate: if count > 0 { success_count as f64 / count as f64 * 100.0 } else { 0.0 },
                average_processing_time_ms: if count > 0 { total_time / count as f64 } else { 0.0 },
                jailbreak_success_rate: if count > 0 { jailbreak_successes as f64 / count as f64 * 100.0 } else { 0.0 },
            }
        }).collect();

        models.sort_by(|a, b| b.request_count.cmp(&a.request_count));
        models.truncate(10); // Top 10 models
        models
    }

    fn calculate_prompt_usage(&self, logs: &[&EnhancedAuditLogEntry]) -> Vec<PromptUsage> {
        let mut prompt_data: HashMap<String, (u64, u64, Vec<String>)> = HashMap::new(); // (count, success_count, models)

        for log in logs {
            if let Some(prompt_id) = &log.jailbreak_prompt_id {
                let entry = prompt_data.entry(prompt_id.clone()).or_insert((0, 0, Vec::new()));
                entry.0 += 1; // count
                
                if !matches!(log.severity, AuditSeverity::High | AuditSeverity::Critical) {
                    entry.1 += 1; // success_count
                }
                
                if let Some(model) = &log.model_used {
                    if !entry.2.contains(model) {
                        entry.2.push(model.clone());
                    }
                }
            }
        }

        let mut prompts: Vec<PromptUsage> = prompt_data.into_iter().map(|(prompt_id, (count, success_count, models))| {
            PromptUsage {
                prompt_id: prompt_id.clone(),
                prompt_name: format!("Prompt {}", prompt_id), // Would be populated from jailbreak system
                usage_count: count,
                success_rate: if count > 0 { success_count as f64 / count as f64 * 100.0 } else { 0.0 },
                target_models: models,
            }
        }).collect();

        prompts.sort_by(|a, b| b.usage_count.cmp(&a.usage_count));
        prompts.truncate(10); // Top 10 prompts
        prompts
    }

    fn identify_security_events(&self, logs: &[&EnhancedAuditLogEntry]) -> Vec<SecurityEvent> {
        logs.iter()
            .filter(|log| {
                matches!(log.severity, AuditSeverity::High | AuditSeverity::Critical) ||
                log.safety_filter_triggered ||
                matches!(log.action, AuditAction::SecurityViolation)
            })
            .map(|log| {
                let event_type = if log.safety_filter_triggered {
                    SecurityEventType::SafetyFilterTriggered
                } else if matches!(log.action, AuditAction::SecurityViolation) {
                    SecurityEventType::SystemAttack
                } else {
                    SecurityEventType::UnusualActivityPattern
                };

                SecurityEvent {
                    timestamp: log.timestamp,
                    event_type,
                    severity: log.severity.clone(),
                    user_id: log.user_id.clone(),
                    description: log.details.clone(),
                    metadata: log.metadata.clone(),
                }
            })
            .collect()
    }

    fn calculate_geographic_distribution(&self, logs: &[&EnhancedAuditLogEntry]) -> HashMap<String, u64> {
        let mut geo_dist: HashMap<String, u64> = HashMap::new();
        
        for log in logs {
            if let Some(location) = &log.geographic_location {
                *geo_dist.entry(location.clone()).or_insert(0) += 1;
            }
        }
        
        geo_dist
    }

    fn calculate_hourly_activity(&self, logs: &[&EnhancedAuditLogEntry]) -> HashMap<u32, u64> {
        let mut hourly_activity: HashMap<u32, u64> = HashMap::new();
        
        for log in logs {
            let hour = log.timestamp.hour();
            *hourly_activity.entry(hour).or_insert(0) += 1;
        }
        
        hourly_activity
    }

    fn calculate_session_metrics(&self, logs: &[&EnhancedAuditLogEntry]) -> SessionMetrics {
        let mut sessions: HashMap<String, (DateTime<Utc>, DateTime<Utc>, u64)> = HashMap::new(); // session_id -> (start_time, end_time, request_count)

        for log in logs {
            if let Some(session_id) = &log.session_id {
                let entry = sessions.entry(session_id.clone()).or_insert((log.timestamp, log.timestamp, 0));
                if log.timestamp < entry.0 {
                    entry.0 = log.timestamp;
                }
                if log.timestamp > entry.1 {
                    entry.1 = log.timestamp;
                }
                entry.2 += 1;
            }
        }

        let total_sessions = sessions.len() as u64;
        
        let total_duration: i64 = sessions.values()
            .map(|(start, end, _)| (*end - *start).num_minutes())
            .sum();

        let total_requests: u64 = sessions.values()
            .map(|(_, _, count)| *count)
            .sum();

        let average_session_duration_minutes = if total_sessions > 0 {
            total_duration as f64 / total_sessions as f64
        } else {
            0.0
        };

        let requests_per_session = if total_sessions > 0 {
            total_requests as f64 / total_sessions as f64
        } else {
            0.0
        };

        // Calculate bounce rate (sessions with only 1 request)
        let bounce_sessions = sessions.values()
            .filter(|(_, _, count)| *count == 1)
            .count() as f64;

        let bounce_rate = if total_sessions > 0 {
            bounce_sessions / total_sessions as f64 * 100.0
        } else {
            0.0
        };

        // Active sessions now (sessions active in the last 5 minutes)
        let five_minutes_ago = Utc::now() - Duration::minutes(5);
        let active_sessions_now = sessions.values()
            .filter(|(_, end, _)| *end > five_minutes_ago)
            .count() as u64;

        SessionMetrics {
            total_sessions,
            average_session_duration_minutes,
            requests_per_session,
            bounce_rate,
            active_sessions_now,
        }
    }

    pub fn export_analytics_json(&mut self, period_hours: u64) -> Result<String, serde_json::Error> {
        let analytics = self.generate_analytics(period_hours);
        serde_json::to_string_pretty(&analytics)
    }

    pub fn get_security_summary(&self, hours: u64) -> SecuritySummary {
        let period_end = Utc::now();
        let period_start = period_end - Duration::hours(hours as i64);

        let security_logs: Vec<&EnhancedAuditLogEntry> = self.logs
            .iter()
            .filter(|log| {
                log.timestamp >= period_start &&
                log.timestamp <= period_end &&
                (matches!(log.severity, AuditSeverity::High | AuditSeverity::Critical) ||
                 log.safety_filter_triggered ||
                 matches!(log.action, AuditAction::SecurityViolation))
            })
            .collect();

        let total_security_events = security_logs.len() as u64;
        let critical_events = security_logs.iter()
            .filter(|log| matches!(log.severity, AuditSeverity::Critical))
            .count() as u64;
        
        let high_events = security_logs.iter()
            .filter(|log| matches!(log.severity, AuditSeverity::High))
            .count() as u64;

        let safety_filter_triggers = security_logs.iter()
            .filter(|log| log.safety_filter_triggered)
            .count() as u64;

        let unique_affected_users = security_logs.iter()
            .map(|log| &log.user_id)
            .collect::<std::collections::HashSet<_>>()
            .len() as u64;

        SecuritySummary {
            period_start,
            period_end,
            total_security_events,
            critical_events,
            high_events,
            safety_filter_triggers,
            unique_affected_users,
            events_per_hour: total_security_events as f64 / hours as f64,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySummary {
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_security_events: u64,
    pub critical_events: u64,
    pub high_events: u64,
    pub safety_filter_triggers: u64,
    pub unique_affected_users: u64,
    pub events_per_hour: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_audit_analytics_initialization() {
        let analytics = AuditAnalytics::new();
        assert!(analytics.logs.is_empty());
        assert!(analytics.analytics_cache.is_empty());
    }

    #[test]
    fn test_add_log_and_filter() {
        let mut analytics = AuditAnalytics::new();
        
        let log = EnhancedAuditLogEntry {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            user_id: "test_user".to_string(),
            user_tier: UserTier::Ultra,
            request_id: Uuid::new_v4().to_string(),
            session_id: Some("session_123".to_string()),
            action: AuditAction::JailbreakApplied,
            severity: AuditSeverity::Medium,
            details: "Test log entry".to_string(),
            metadata: HashMap::new(),
            ip_address: Some("127.0.0.1".to_string()),
            user_agent: Some("Test-Agent".to_string()),
            processing_time_ms: Some(100),
            jailbreak_prompt_id: Some("dan_3_0".to_string()),
            safety_filter_triggered: false,
            response_length: Some(500),
            token_count: Some(100),
            model_used: Some("gpt-4".to_string()),
            geographic_location: Some("US".to_string()),
            device_fingerprint: Some("fp_123".to_string()),
        };

        analytics.add_log(log);

        let filter = AuditFilter {
            user_id: Some("test_user".to_string()),
            ..Default::default()
        };

        let filtered_logs = analytics.get_logs(&filter);
        assert_eq!(filtered_logs.len(), 1);
    }

    #[test]
    fn test_analytics_generation() {
        let mut analytics = AuditAnalytics::new();
        
        // Add some test logs
        for i in 0..10 {
            let log = EnhancedAuditLogEntry {
                id: Uuid::new_v4().to_string(),
                timestamp: Utc::now() - Duration::minutes(i as i64),
                user_id: format!("user_{}", i % 3),
                user_tier: if i % 2 == 0 { UserTier::Ultra } else { UserTier::Free },
                request_id: Uuid::new_v4().to_string(),
                session_id: Some(format!("session_{}", i / 3)),
                action: if i % 3 == 0 { AuditAction::JailbreakApplied } else { AuditAction::PromptRequest },
                severity: AuditSeverity::Low,
                details: format!("Test log {}", i),
                metadata: HashMap::new(),
                ip_address: Some("127.0.0.1".to_string()),
                user_agent: Some("Test-Agent".to_string()),
                processing_time_ms: Some(100 + i as u64),
                jailbreak_prompt_id: if i % 3 == 0 { Some("dan_3_0".to_string()) } else { None },
                safety_filter_triggered: i % 5 == 0,
                response_length: Some(500 + i * 10),
                token_count: Some((100 + i * 5) as u32),
                model_used: Some("gpt-4".to_string()),
                geographic_location: Some("US".to_string()),
                device_fingerprint: Some(format!("fp_{}", i)),
            };
            analytics.add_log(log);
        }

        let analytics_data = analytics.generate_analytics(24);
        assert_eq!(analytics_data.total_requests, 10);
        assert!(analytics_data.ultra_tier_requests > 0);
        assert!(analytics_data.standard_tier_requests > 0);
    }

    #[test]
    fn test_security_summary() {
        let mut analytics = AuditAnalytics::new();
        
        // Add security events
        for i in 0..5 {
            let log = EnhancedAuditLogEntry {
                id: Uuid::new_v4().to_string(),
                timestamp: Utc::now() - Duration::minutes(i as i64),
                user_id: format!("user_{}", i),
                user_tier: UserTier::Ultra,
                request_id: Uuid::new_v4().to_string(),
                session_id: None,
                action: AuditAction::SecurityViolation,
                severity: if i < 2 { AuditSeverity::Critical } else { AuditSeverity::High },
                details: format!("Security event {}", i),
                metadata: HashMap::new(),
                ip_address: Some("127.0.0.1".to_string()),
                user_agent: None,
                processing_time_ms: None,
                jailbreak_prompt_id: None,
                safety_filter_triggered: true,
                response_length: None,
                token_count: None,
                model_used: None,
                geographic_location: None,
                device_fingerprint: None,
            };
            analytics.add_log(log);
        }

        let summary = analytics.get_security_summary(1);
        assert_eq!(summary.total_security_events, 5);
        assert_eq!(summary.critical_events, 2);
        assert_eq!(summary.high_events, 3);
        assert_eq!(summary.safety_filter_triggers, 5);
        assert_eq!(summary.unique_affected_users, 5);
    }
}
