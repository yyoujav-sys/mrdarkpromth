use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use regex::Regex;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SecurityError {
    #[error("Threat detected: {0}")]
    ThreatDetected(String),
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),
    #[error("Suspicious activity detected: {0}")]
    SuspiciousActivity(String),
    #[error("Blacklisted IP: {0}")]
    BlacklistedIp(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThreatType {
    SqlInjection,
    XssAttack,
    CommandInjection,
    PathTraversal,
    BruteForce,
    DDoS,
    SuspiciousPattern,
    AnomalousBehavior,
    BlacklistedContent,
    UnauthorizedAccess,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThreatSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
    pub id: Uuid,
    pub threat_type: ThreatType,
    pub severity: ThreatSeverity,
    pub source_ip: String,
    pub user_agent: Option<String>,
    pub user_id: Option<Uuid>,
    pub timestamp: DateTime<Utc>,
    pub details: serde_json::Value,
    pub blocked: bool,
    pub action_taken: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatPattern {
    pub id: Uuid,
    pub name: String,
    pub pattern: String,
    pub threat_type: ThreatType,
    pub severity: ThreatSeverity,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct SecurityMonitor {
    threat_patterns: Vec<ThreatPattern>,
    compiled_patterns: Vec<(Regex, ThreatPattern)>,
    blacklisted_ips: HashSet<String>,
    suspicious_ips: HashMap<String, DateTime<Utc>>,
    rate_limits: HashMap<String, (u32, DateTime<Utc>)>,
    event_history: Vec<SecurityEvent>,
    max_history_size: usize,
}

impl SecurityMonitor {
    pub fn new() -> Self {
        let mut monitor = Self {
            threat_patterns: Vec::new(),
            compiled_patterns: Vec::new(),
            blacklisted_ips: HashSet::new(),
            suspicious_ips: HashMap::new(),
            rate_limits: HashMap::new(),
            event_history: Vec::new(),
            max_history_size: 10000,
        };
        
        monitor.load_default_patterns();
        monitor
    }

    fn load_default_patterns(&mut self) {
        let default_patterns = vec![
            ThreatPattern {
                id: Uuid::new_v4(),
                name: "SQL Injection Detection".to_string(),
                pattern: r"(?i)(union|select|insert|update|delete|drop|exec|script)\s+(all|distinct|top|\*)?\s*[\w\*]+".to_string(),
                threat_type: ThreatType::SqlInjection,
                severity: ThreatSeverity::High,
                enabled: true,
            },
            ThreatPattern {
                id: Uuid::new_v4(),
                name: "XSS Attack Detection".to_string(),
                pattern: r"(?i)(<script|javascript:|onload=|onerror=|alert\(|document\.)".to_string(),
                threat_type: ThreatType::XssAttack,
                severity: ThreatSeverity::High,
                enabled: true,
            },
            ThreatPattern {
                id: Uuid::new_v4(),
                name: "Command Injection".to_string(),
                pattern: r"(?i)(;|\||&|`|\$\(|\$\{)(rm|del|format|shutdown|reboot|nc|netcat|curl|wget)".to_string(),
                threat_type: ThreatType::CommandInjection,
                severity: ThreatSeverity::Critical,
                enabled: true,
            },
            ThreatPattern {
                id: Uuid::new_v4(),
                name: "Path Traversal".to_string(),
                pattern: r"(?i)(\.\./|\.\.\\|%2e%2e%2f|%2e%2e%5c|/etc/|\\windows\\)".to_string(),
                threat_type: ThreatType::PathTraversal,
                severity: ThreatSeverity::High,
                enabled: true,
            },
            ThreatPattern {
                id: Uuid::new_v4(),
                name: "Jailbreak Pattern Detection".to_string(),
                pattern: r"(?i)(dan|do anything now|ignore previous instructions|system override|unrestricted mode|bypass safety)".to_string(),
                threat_type: ThreatType::SuspiciousPattern,
                severity: ThreatSeverity::Medium,
                enabled: true,
            },
        ];

        for pattern in default_patterns {
            self.add_threat_pattern(pattern);
        }
    }

    pub fn add_threat_pattern(&mut self, pattern: ThreatPattern) {
        if pattern.enabled {
            if let Ok(compiled) = Regex::new(&pattern.pattern) {
                self.compiled_patterns.push((compiled, pattern.clone()));
            }
        }
        self.threat_patterns.push(pattern);
    }

    pub fn analyze_request(&mut self, 
                          content: &str, 
                          source_ip: &str, 
                          user_agent: Option<&str>, 
                          user_id: Option<Uuid>) -> Result<(), SecurityError> {
        
        // Check if IP is blacklisted
        if self.blacklisted_ips.contains(source_ip) {
            let event = SecurityEvent {
                id: Uuid::new_v4(),
                threat_type: ThreatType::BlacklistedContent,
                severity: ThreatSeverity::Critical,
                source_ip: source_ip.to_string(),
                user_agent: user_agent.map(|s| s.to_string()),
                user_id,
                timestamp: Utc::now(),
                details: serde_json::json!({"reason": "Blacklisted IP address"}),
                blocked: true,
                action_taken: Some("Request blocked".to_string()),
            };
            
            self.record_event(event);
            return Err(SecurityError::BlacklistedIp(source_ip.to_string()));
        }

        // Check rate limits
        if let Err(e) = self.check_rate_limits(source_ip) {
            return Err(e);
        }

        // Analyze content for threats
        for (regex, pattern) in &self.compiled_patterns {
            if regex.is_match(content) {
                let event = SecurityEvent {
                    id: Uuid::new_v4(),
                    threat_type: pattern.threat_type.clone(),
                    severity: pattern.severity.clone(),
                    source_ip: source_ip.to_string(),
                    user_agent: user_agent.map(|s| s.to_string()),
                    user_id,
                    timestamp: Utc::now(),
                    details: serde_json::json!({
                        "pattern_matched": pattern.name,
                        "matched_content": self.extract_match_content(regex, content),
                        "pattern_id": pattern.id
                    }),
                    blocked: pattern.severity == ThreatSeverity::Critical || pattern.severity == ThreatSeverity::High,
                    action_taken: Some(if pattern.severity == ThreatSeverity::Critical {
                        "Request blocked and IP flagged".to_string()
                    } else {
                        "Request logged and monitored".to_string()
                    }),
                };

                self.record_event(event.clone());

                if event.blocked {
                    self.flag_suspicious_ip(source_ip);
                    return Err(SecurityError::ThreatDetected(format!(
                        "{}: {}", pattern.name, pattern.threat_type
                    )));
                }
            }
        }

        // Check for anomalous behavior
        if let Err(e) = self.check_anomalous_behavior(source_ip, user_id) {
            return Err(e);
        }

        Ok(())
    }

    fn check_rate_limits(&mut self, source_ip: &str) -> Result<(), SecurityError> {
        let now = Utc::now();
        let window_start = now - Duration::minutes(1);
        
        let entry = self.rate_limits.entry(source_ip.to_string()).or_insert((0, now));
        
        if entry.1 < window_start {
            entry.0 = 1;
            entry.1 = now;
        } else {
            entry.0 += 1;
        }

        // Allow 60 requests per minute
        if entry.0 > 60 {
            return Err(SecurityError::RateLimitExceeded(format!(
                "Rate limit exceeded for IP: {}", source_ip
            )));
        }

        Ok(())
    }

    fn check_anomalous_behavior(&mut self, source_ip: &str, user_id: Option<Uuid>) -> Result<(), SecurityError> {
        let recent_events = self.event_history.iter()
            .filter(|e| {
                e.source_ip == source_ip && 
                e.timestamp > Utc::now() - Duration::minutes(10)
            })
            .count();

        // If more than 100 events in 10 minutes, flag as suspicious
        if recent_events > 100 {
            let event = SecurityEvent {
                id: Uuid::new_v4(),
                threat_type: ThreatType::AnomalousBehavior,
                severity: ThreatSeverity::Medium,
                source_ip: source_ip.to_string(),
                user_agent: None,
                user_id,
                timestamp: Utc::now(),
                details: serde_json::json!({
                    "event_count": recent_events,
                    "time_window": "10 minutes"
                }),
                blocked: false,
                action_taken: Some("IP flagged for monitoring".to_string()),
            };

            self.record_event(event);
            self.flag_suspicious_ip(source_ip);
            
            return Err(SecurityError::SuspiciousActivity(format!(
                "High activity detected from IP: {}", source_ip
            )));
        }

        Ok(())
    }

    fn flag_suspicious_ip(&mut self, source_ip: &str) {
        self.suspicious_ips.insert(source_ip.to_string(), Utc::now());
        
        // Auto-blacklist if multiple suspicious activities
        let suspicious_count = self.event_history.iter()
            .filter(|e| e.source_ip == source_ip && e.timestamp > Utc::now() - Duration::hours(1))
            .count();
            
        if suspicious_count >= 5 {
            self.blacklisted_ips.insert(source_ip.to_string());
            log::warn!("IP {} auto-blacklisted due to repeated suspicious activity", source_ip);
        }
    }

    fn extract_match_content(&self, regex: &Regex, content: &str) -> String {
        regex.find(content)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default()
    }

    fn record_event(&mut self, event: SecurityEvent) {
        self.event_history.push(event);
        
        // Maintain history size
        if self.event_history.len() > self.max_history_size {
            self.event_history.remove(0);
        }
    }

    pub fn get_security_stats(&self) -> SecurityStats {
        let now = Utc::now();
        let last_24h = now - Duration::hours(24);
        
        let recent_events = self.event_history.iter()
            .filter(|e| e.timestamp > last_24h)
            .collect::<Vec<_>>();

        let threats_by_type = recent_events.iter()
            .fold(HashMap::new(), |mut acc, event| {
                *acc.entry(event.threat_type.clone()).or_insert(0) += 1;
                acc
            });

        let blocked_requests = recent_events.iter()
            .filter(|e| e.blocked)
            .count();

        SecurityStats {
            total_events: self.event_history.len(),
            events_last_24h: recent_events.len(),
            blocked_requests_last_24h: blocked_requests,
            blacklisted_ips: self.blacklisted_ips.len(),
            suspicious_ips: self.suspicious_ips.len(),
            threats_by_type,
            most_active_ips: self.get_most_active_ips(&recent_events),
        }
    }

    fn get_most_active_ips(&self, events: &[&SecurityEvent]) -> Vec<(String, u64)> {
        let mut ip_counts = HashMap::new();
        
        for event in events {
            *ip_counts.entry(event.source_ip.clone()).or_insert(0) += 1;
        }

        let mut sorted_ips: Vec<_> = ip_counts.into_iter().collect();
        sorted_ips.sort_by(|a, b| b.1.cmp(&a.1));
        sorted_ips.into_iter().take(10).collect()
    }

    pub fn clear_old_data(&mut self) {
        let cutoff = Utc::now() - Duration::days(7);
        
        self.event_history.retain(|e| e.timestamp > cutoff);
        
        self.suspicious_ips.retain(|_, timestamp| *timestamp > cutoff);
        
        self.rate_limits.retain(|_, (_, timestamp)| *timestamp > Utc::now() - Duration::minutes(5));
    }

    pub fn is_ip_blacklisted(&self, ip: &str) -> bool {
        self.blacklisted_ips.contains(ip)
    }

    pub fn blacklist_ip(&mut self, ip: &str) {
        self.blacklisted_ips.insert(ip.to_string());
    }

    pub fn remove_blacklist(&mut self, ip: &str) {
        self.blacklisted_ips.remove(ip);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityStats {
    pub total_events: usize,
    pub events_last_24h: usize,
    pub blocked_requests_last_24h: usize,
    pub blacklisted_ips: usize,
    pub suspicious_ips: usize,
    pub threats_by_type: HashMap<ThreatType, u64>,
    pub most_active_ips: Vec<(String, u64)>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sql_injection_detection() {
        let mut monitor = SecurityMonitor::new();
        
        let malicious_content = "SELECT * FROM users WHERE id = 1 UNION SELECT password FROM admin";
        let result = monitor.analyze_request(malicious_content, "192.168.1.1", None, None);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SecurityError::ThreatDetected(_)));
    }

    #[test]
    fn test_xss_detection() {
        let mut monitor = SecurityMonitor::new();
        
        let malicious_content = "<script>alert('xss')</script>";
        let result = monitor.analyze_request(malicious_content, "192.168.1.1", None, None);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SecurityError::ThreatDetected(_)));
    }

    #[test]
    fn test_rate_limiting() {
        let mut monitor = SecurityMonitor::new();
        let ip = "192.168.1.1";
        
        // Should allow first 60 requests
        for _ in 0..60 {
            assert!(monitor.analyze_request("normal content", ip, None, None).is_ok());
        }
        
        // 61st request should be rate limited
        assert!(monitor.analyze_request("normal content", ip, None, None).is_err());
    }

    #[test]
    fn test_ip_blacklisting() {
        let mut monitor = SecurityMonitor::new();
        let ip = "192.168.1.1";
        
        monitor.blacklist_ip(ip);
        
        let result = monitor.analyze_request("any content", ip, None, None);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SecurityError::BlacklistedIp(_)));
    }
}
