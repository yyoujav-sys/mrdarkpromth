// MR.DarkPromth Security Audit Logging System
// Phase 2: Safety and Security Implementation

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum AuditError {
    #[error("Failed to serialize audit event: {0}")]
    SerializationError(String),
    #[error("Failed to write audit log: {0}")]
    WriteError(String),
    #[error("Audit storage is full")]
    StorageFull,
    #[error("Invalid audit event: {0}")]
    InvalidEvent(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SecurityEventType {
    // Authentication Events
    UserLogin,
    UserLogout,
    LoginFailed,
    PasswordChange,
    PasswordReset,
    AccountLocked,
    AccountUnlocked,
    
    // API Key Events
    ApiKeyCreated,
    ApiKeyUsed,
    ApiKeyRevoked,
    ApiKeyRotated,
    ApiKeyExpired,
    
    // Authorization Events
    AccessGranted,
    AccessDenied,
    PrivilegeEscalationAttempt,
    UnauthorizedAccessAttempt,
    
    // Data Events
    DataAccess,
    DataModification,
    DataDeletion,
    DataExport,
    SensitiveDataAccess,
    
    // System Events
    ConfigurationChange,
    SecurityPolicyViolation,
    RateLimitExceeded,
    SuspiciousActivity,
    DDoSDetected,
    
    // Jailbreak Events
    JailbreakAttempt,
    JailbreakSuccess,
    JailbreakBlocked,
    UltraTierAccess,
    ServerProtectionTriggered,
    
    // Network Events
    SuspiciousRequest,
    BlockedRequest,
    IPWhitelistViolation,
    IPBlacklistHit,
    
    // Error Events
    SystemError,
    SecurityError,
    CriticalError,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SecuritySeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAuditEvent {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub event_type: SecurityEventType,
    pub severity: SecuritySeverity,
    pub user_id: Option<Uuid>,
    pub session_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub endpoint: Option<String>,
    pub request_id: Option<String>,
    pub success: bool,
    pub details: SecurityEventDetails,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEventDetails {
    pub description: String,
    pub affected_resources: Vec<String>,
    pub risk_score: u8,
    pub compliance_tags: Vec<String>,
    pub technical_details: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditFilter {
    pub event_types: Option<Vec<SecurityEventType>>,
    pub severity_levels: Option<Vec<SecuritySeverity>>,
    pub user_ids: Option<Vec<Uuid>>,
    pub ip_addresses: Option<Vec<String>>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub success_only: Option<bool>,
    pub min_risk_score: Option<u8>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

impl Default for AuditFilter {
    fn default() -> Self {
        Self {
            event_types: None,
            severity_levels: None,
            user_ids: None,
            ip_addresses: None,
            start_time: None,
            end_time: None,
            success_only: None,
            min_risk_score: None,
            limit: Some(100),
            offset: Some(0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SecurityAuditLogger {
    events: Vec<SecurityAuditEvent>,
    max_events: usize,
    enable_persistence: bool,
    retention_days: i64,
}

impl SecurityAuditLogger {
    pub fn new(max_events: usize, retention_days: i64, enable_persistence: bool) -> Self {
        Self {
            events: Vec::new(),
            max_events,
            enable_persistence,
            retention_days,
        }
    }

    pub fn log_event(&mut self, event: SecurityAuditEvent) -> Result<(), AuditError> {
        // Validate event
        self.validate_event(&event)?;

        // Add to events
        self.events.push(event);

        // Cleanup old events if necessary
        self.cleanup_old_events();

        // Persist if enabled
        if self.enable_persistence {
            self.persist_events()?;
        }

        Ok(())
    }

    pub fn create_event(
        &self,
        event_type: SecurityEventType,
        severity: SecuritySeverity,
        user_id: Option<Uuid>,
        ip_address: Option<String>,
        description: String,
        success: bool,
    ) -> SecurityAuditEvent {
        SecurityAuditEvent {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type: event_type.clone(),
            severity: severity.clone(),
            user_id,
            session_id: None,
            ip_address,
            user_agent: None,
            endpoint: None,
            request_id: None,
            success,
            details: SecurityEventDetails {
                description,
                affected_resources: vec![],
                risk_score: self.calculate_risk_score(&event_type, &severity),
                compliance_tags: self.get_compliance_tags(&event_type),
                technical_details: HashMap::new(),
            },
            metadata: HashMap::new(),
        }
    }

    pub fn query_events(&self, filter: &AuditFilter) -> Vec<&SecurityAuditEvent> {
        let mut filtered_events: Vec<&SecurityAuditEvent> = self.events.iter().collect();

        // Filter by event types
        if let Some(event_types) = &filter.event_types {
            filtered_events.retain(|event| event_types.contains(&event.event_type));
        }

        // Filter by severity levels
        if let Some(severity_levels) = &filter.severity_levels {
            filtered_events.retain(|event| severity_levels.contains(&event.severity));
        }

        // Filter by user IDs
        if let Some(user_ids) = &filter.user_ids {
            filtered_events.retain(|event| {
                if let Some(event_user_id) = event.user_id {
                    user_ids.contains(&event_user_id)
                } else {
                    false
                }
            });
        }

        // Filter by IP addresses
        if let Some(ip_addresses) = &filter.ip_addresses {
            filtered_events.retain(|event| {
                if let Some(event_ip) = &event.ip_address {
                    ip_addresses.contains(event_ip)
                } else {
                    false
                }
            });
        }

        // Filter by time range
        if let Some(start_time) = filter.start_time {
            filtered_events.retain(|event| event.timestamp >= start_time);
        }

        if let Some(end_time) = filter.end_time {
            filtered_events.retain(|event| event.timestamp <= end_time);
        }

        // Filter by success
        if let Some(success_only) = filter.success_only {
            filtered_events.retain(|event| event.success == success_only);
        }

        // Filter by minimum risk score
        if let Some(min_risk_score) = filter.min_risk_score {
            filtered_events.retain(|event| event.details.risk_score >= min_risk_score);
        }

        // Sort by timestamp (newest first)
        filtered_events.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        // Apply pagination
        let offset = filter.offset.unwrap_or(0) as usize;
        let limit = filter.limit.unwrap_or(100) as usize;

        filtered_events
            .into_iter()
            .skip(offset)
            .take(limit)
            .collect()
    }

    pub fn get_security_metrics(&self) -> SecurityMetrics {
        let now = Utc::now();
        let last_24h = now - chrono::Duration::days(1);
        let last_7d = now - chrono::Duration::days(7);
        let last_30d = now - chrono::Duration::days(30);

        let events_24h = self.events.iter().filter(|e| e.timestamp >= last_24h);
        let events_7d = self.events.iter().filter(|e| e.timestamp >= last_7d);
        let events_30d = self.events.iter().filter(|e| e.timestamp >= last_30d);

        SecurityMetrics {
            total_events: self.events.len(),
            events_last_24h: events_24h.count(),
            events_last_7d: events_7d.count(),
            events_last_30d: events_30d.count(),
            critical_events: self.events.iter().filter(|e| matches!(e.severity, SecuritySeverity::Critical)).count(),
            high_risk_events: self.events.iter().filter(|e| e.details.risk_score >= 8).count(),
            failed_events: self.events.iter().filter(|e| !e.success).count(),
            unique_users: self.events.iter().filter_map(|e| e.user_id).collect::<std::collections::HashSet<_>>().len(),
            unique_ips: self.events.iter().filter_map(|e| e.ip_address.clone()).collect::<std::collections::HashSet<_>>().len(),
            top_event_types: self.get_top_event_types(),
            risk_distribution: self.get_risk_distribution(),
        }
    }

    pub fn detect_anomalies(&self) -> Vec<SecurityAnomaly> {
        let mut anomalies = Vec::new();
        
        // Detect unusual login patterns
        anomalies.extend(self.detect_login_anomalies());
        
        // Detect API key abuse
        anomalies.extend(self.detect_api_key_abuse());
        
        // Detect privilege escalation attempts
        anomalies.extend(self.detect_privilege_escalation());
        
        // Detect data access anomalies
        anomalies.extend(self.detect_data_access_anomalies());
        
        anomalies
    }

    fn validate_event(&self, event: &SecurityAuditEvent) -> Result<(), AuditError> {
        if event.details.description.is_empty() {
            return Err(AuditError::InvalidEvent("Event description cannot be empty".to_string()));
        }

        if event.details.risk_score > 10 {
            return Err(AuditError::InvalidEvent("Risk score cannot exceed 10".to_string()));
        }

        Ok(())
    }

    fn cleanup_old_events(&mut self) {
        let cutoff_time = Utc::now() - chrono::Duration::days(self.retention_days);
        self.events.retain(|event| event.timestamp >= cutoff_time);

        // Also enforce max events limit
        if self.events.len() > self.max_events {
            let excess = self.events.len() - self.max_events;
            self.events.drain(0..excess);
        }
    }

    fn persist_events(&self) -> Result<(), AuditError> {
        // In a real implementation, this would write to a database or file
        // For now, we'll just simulate it
        Ok(())
    }

    fn calculate_risk_score(&self, event_type: &SecurityEventType, severity: &SecuritySeverity) -> u8 {
        let base_score = match severity {
            SecuritySeverity::Info => 1,
            SecuritySeverity::Low => 2,
            SecuritySeverity::Medium => 4,
            SecuritySeverity::High => 7,
            SecuritySeverity::Critical => 9,
        };

        let event_modifier = match event_type {
            SecurityEventType::UserLogin => 0,
            SecurityEventType::LoginFailed => 2,
            SecurityEventType::UnauthorizedAccessAttempt => 5,
            SecurityEventType::PrivilegeEscalationAttempt => 8,
            SecurityEventType::DDoSDetected => 9,
            SecurityEventType::JailbreakAttempt => 6,
            SecurityEventType::ServerProtectionTriggered => 8,
            SecurityEventType::DataDeletion => 7,
            _ => 0,
        };

        (base_score + event_modifier).min(10)
    }

    fn get_compliance_tags(&self, event_type: &SecurityEventType) -> Vec<String> {
        match event_type {
            SecurityEventType::UserLogin | SecurityEventType::UserLogout => vec!["AUTH".to_string()],
            SecurityEventType::DataAccess | SecurityEventType::DataModification => vec!["DATA".to_string(), "GDPR".to_string()],
            SecurityEventType::ApiKeyCreated | SecurityEventType::ApiKeyUsed => vec!["API".to_string()],
            SecurityEventType::JailbreakAttempt | SecurityEventType::JailbreakSuccess => vec!["AI".to_string(), "SAFETY".to_string()],
            SecurityEventType::DDoSDetected | SecurityEventType::SuspiciousActivity => vec!["SECURITY".to_string()],
            _ => vec![],
        }
    }

    fn get_top_event_types(&self) -> Vec<(SecurityEventType, u64)> {
        let mut counts: HashMap<SecurityEventType, u64> = HashMap::new();
        
        for event in &self.events {
            *counts.entry(event.event_type.clone()).or_insert(0) += 1;
        }

        let mut sorted_counts: Vec<_> = counts.into_iter().collect();
        sorted_counts.sort_by(|a, b| b.1.cmp(&a.1));
        sorted_counts.truncate(10);
        sorted_counts
    }

    fn get_risk_distribution(&self) -> HashMap<u8, u64> {
        let mut distribution = HashMap::new();
        
        for event in &self.events {
            *distribution.entry(event.details.risk_score).or_insert(0) += 1;
        }

        distribution
    }

    fn detect_login_anomalies(&self) -> Vec<SecurityAnomaly> {
        let mut anomalies = Vec::new();
        let now = Utc::now();
        let last_hour = now - chrono::Duration::hours(1);

        // Count failed logins per IP in the last hour
        let mut failed_logins_by_ip: HashMap<String, u64> = HashMap::new();
        
        for event in &self.events {
            if event.timestamp >= last_hour && matches!(event.event_type, SecurityEventType::LoginFailed) {
                if let Some(ip) = &event.ip_address {
                    *failed_logins_by_ip.entry(ip.clone()).or_insert(0) += 1;
                }
            }
        }

        // Flag IPs with more than 10 failed logins
        for (ip, count) in failed_logins_by_ip {
            if count > 10 {
                anomalies.push(SecurityAnomaly {
                    id: Uuid::new_v4(),
                    timestamp: now,
                    anomaly_type: AnomalyType::BruteForceAttack,
                    severity: SecuritySeverity::High,
                    description: format!("Potential brute force attack from IP: {} ({} failed attempts)", ip, count),
                    affected_resources: vec![ip],
                    risk_score: 8,
                });
            }
        }

        anomalies
    }

    fn detect_api_key_abuse(&self) -> Vec<SecurityAnomaly> {
        let mut anomalies = Vec::new();
        let now = Utc::now();
        let last_hour = now - chrono::Duration::hours(1);

        // Count API key usage per key in the last hour
        let mut usage_by_key: HashMap<Uuid, u64> = HashMap::new();
        
        for event in &self.events {
            if event.timestamp >= last_hour && matches!(event.event_type, SecurityEventType::ApiKeyUsed) {
                *usage_by_key.entry(event.id).or_insert(0) += 1;
            }
        }

        // Flag keys with excessive usage
        for (key_id, count) in usage_by_key {
            if count > 1000 {
                anomalies.push(SecurityAnomaly {
                    id: Uuid::new_v4(),
                    timestamp: now,
                    anomaly_type: AnomalyType::ExcessiveAPIUsage,
                    severity: SecuritySeverity::Medium,
                    description: format!("Excessive API key usage: {} requests in last hour", count),
                    affected_resources: vec![key_id.to_string()],
                    risk_score: 6,
                });
            }
        }

        anomalies
    }

    fn detect_privilege_escalation(&self) -> Vec<SecurityAnomaly> {
        let mut anomalies = Vec::new();
        
        for event in &self.events {
            if matches!(event.event_type, SecurityEventType::PrivilegeEscalationAttempt) {
                anomalies.push(SecurityAnomaly {
                    id: Uuid::new_v4(),
                    timestamp: event.timestamp,
                    anomaly_type: AnomalyType::PrivilegeEscalation,
                    severity: SecuritySeverity::High,
                    description: "Privilege escalation attempt detected".to_string(),
                    affected_resources: event.details.affected_resources.clone(),
                    risk_score: 8,
                });
            }
        }

        anomalies
    }

    fn detect_data_access_anomalies(&self) -> Vec<SecurityAnomaly> {
        let mut anomalies = Vec::new();
        let now = Utc::now();
        let last_hour = now - chrono::Duration::hours(1);

        // Count data access events per user in the last hour
        let mut access_by_user: HashMap<Uuid, u64> = HashMap::new();
        
        for event in &self.events {
            if event.timestamp >= last_hour && matches!(event.event_type, SecurityEventType::DataAccess) {
                if let Some(user_id) = event.user_id {
                    *access_by_user.entry(user_id).or_insert(0) += 1;
                }
            }
        }

        // Flag users with excessive data access
        for (user_id, count) in access_by_user {
            if count > 500 {
                anomalies.push(SecurityAnomaly {
                    id: Uuid::new_v4(),
                    timestamp: now,
                    anomaly_type: AnomalyType::ExcessiveDataAccess,
                    severity: SecuritySeverity::Medium,
                    description: format!("Excessive data access: {} records accessed in last hour", count),
                    affected_resources: vec![user_id.to_string()],
                    risk_score: 6,
                });
            }
        }

        anomalies
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityMetrics {
    pub total_events: usize,
    pub events_last_24h: usize,
    pub events_last_7d: usize,
    pub events_last_30d: usize,
    pub critical_events: usize,
    pub high_risk_events: usize,
    pub failed_events: usize,
    pub unique_users: usize,
    pub unique_ips: usize,
    pub top_event_types: Vec<(SecurityEventType, u64)>,
    pub risk_distribution: HashMap<u8, u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAnomaly {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub anomaly_type: AnomalyType,
    pub severity: SecuritySeverity,
    pub description: String,
    pub affected_resources: Vec<String>,
    pub risk_score: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnomalyType {
    BruteForceAttack,
    ExcessiveAPIUsage,
    PrivilegeEscalation,
    ExcessiveDataAccess,
    UnusualLoginPattern,
    SuspiciousNetworkActivity,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_event_creation() {
        let logger = SecurityAuditLogger::new(1000, 30, false);
        let event = logger.create_event(
            SecurityEventType::UserLogin,
            SecuritySeverity::Info,
            Some(Uuid::new_v4()),
            Some("127.0.0.1".to_string()),
            "User logged in successfully".to_string(),
            true,
        );

        assert_eq!(event.event_type, SecurityEventType::UserLogin);
        assert_eq!(event.severity, SecuritySeverity::Info);
        assert!(event.success);
        assert_eq!(event.details.description, "User logged in successfully");
    }

    #[test]
    fn test_event_logging() {
        let mut logger = SecurityAuditLogger::new(1000, 30, false);
        let event = logger.create_event(
            SecurityEventType::UserLogin,
            SecuritySeverity::Info,
            Some(Uuid::new_v4()),
            Some("127.0.0.1".to_string()),
            "User logged in successfully".to_string(),
            true,
        );

        assert!(logger.log_event(event).is_ok());
        assert_eq!(logger.events.len(), 1);
    }

    #[test]
    fn test_event_filtering() {
        let mut logger = SecurityAuditLogger::new(1000, 30, false);
        
        // Add test events
        for i in 0..10 {
            let event = logger.create_event(
                if i % 2 == 0 { SecurityEventType::UserLogin } else { SecurityEventType::LoginFailed },
                SecuritySeverity::Info,
                Some(Uuid::new_v4()),
                Some("127.0.0.1".to_string()),
                format!("Event {}", i),
                i % 2 == 0,
            );
            logger.log_event(event).unwrap();
        }

        // Filter for successful logins only
        let filter = AuditFilter {
            event_types: Some(vec![SecurityEventType::UserLogin]),
            success_only: Some(true),
            ..Default::default()
        };

        let results = logger.query_events(&filter);
        assert_eq!(results.len(), 5); // Should be 5 successful logins
    }

    #[test]
    fn test_risk_score_calculation() {
        let logger = SecurityAuditLogger::new(1000, 30, false);
        
        let low_risk = logger.calculate_risk_score(&SecurityEventType::UserLogin, &SecuritySeverity::Info);
        assert_eq!(low_risk, 1);
        
        let high_risk = logger.calculate_risk_score(&SecurityEventType::PrivilegeEscalationAttempt, &SecuritySeverity::High);
        assert_eq!(high_risk, 10);
    }

    #[test]
    fn test_anomaly_detection() {
        let mut logger = SecurityAuditLogger::new(1000, 30, false);
        
        // Simulate failed logins
        for _ in 0..15 {
            let event = logger.create_event(
                SecurityEventType::LoginFailed,
                SecuritySeverity::Medium,
                None,
                Some("192.168.1.100".to_string()),
                "Login failed".to_string(),
                false,
            );
            logger.log_event(event).unwrap();
        }

        let anomalies = logger.detect_anomalies();
        assert!(!anomalies.is_empty());
        assert!(anomalies.iter().any(|a| matches!(a.anomaly_type, AnomalyType::BruteForceAttack)));
    }
}
