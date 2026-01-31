use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use chrono::{DateTime, Utc, Duration};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RateLimitError {
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),
    #[error("DDoS attack detected: {0}")]
    DDoSDetected(String),
    #[error("Quota exceeded: {0}")]
    QuotaExceeded(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RateLimitType {
    PerSecond,
    PerMinute,
    PerHour,
    PerDay,
    Custom(Duration),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub limit: u32,
    pub window: RateLimitType,
    pub burst_limit: Option<u32>,
    pub penalty_duration: Duration,
    pub auto_block_threshold: Option<u32>,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            limit: 60,
            window: RateLimitType::PerMinute,
            burst_limit: Some(100),
            penalty_duration: Duration::minutes(5),
            auto_block_threshold: Some(200),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitEntry {
    pub count: u32,
    pub first_request: DateTime<Utc>,
    pub last_request: DateTime<Utc>,
    pub blocked_until: Option<DateTime<Utc>>,
    pub penalty_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DDoSMetrics {
    pub total_requests: u64,
    pub unique_ips: u64,
    pub requests_per_second: f64,
    pub attack_detected: bool,
    pub attack_start_time: Option<DateTime<Utc>>,
    pub top_source_ips: Vec<(String, u64)>,
}

#[derive(Debug, Clone)]
pub struct RateLimiter {
    limits: HashMap<String, RateLimitConfig>,
    entries: HashMap<String, RateLimitEntry>,
    global_metrics: GlobalMetrics,
    ddos_detection: DDoSDetection,
    cleanup_interval: Duration,
    last_cleanup: DateTime<Utc>,
}

#[derive(Debug, Clone)]
struct GlobalMetrics {
    total_requests: u64,
    requests_last_minute: VecDeque<DateTime<Utc>>,
    requests_last_hour: VecDeque<DateTime<Utc>>,
    top_ips: HashMap<String, u64>,
}

#[derive(Debug, Clone)]
struct DDoSDetection {
    enabled: bool,
    threshold_rps: f64,
    threshold_unique_ips: u64,
    detection_window: Duration,
    attack_detected: bool,
    attack_start: Option<DateTime<Utc>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            limits: HashMap::new(),
            entries: HashMap::new(),
            global_metrics: GlobalMetrics {
                total_requests: 0,
                requests_last_minute: VecDeque::new(),
                requests_last_hour: VecDeque::new(),
                top_ips: HashMap::new(),
            },
            ddos_detection: DDoSDetection {
                enabled: true,
                threshold_rps: 1000.0,
                threshold_unique_ips: 1000,
                detection_window: Duration::minutes(5),
                attack_detected: false,
                attack_start: None,
            },
            cleanup_interval: Duration::minutes(1),
            last_cleanup: Utc::now(),
        }
    }

    pub fn add_limit(&mut self, key: String, config: RateLimitConfig) {
        self.limits.insert(key, config);
    }

    pub fn check_rate_limit(&mut self, identifier: &str, limit_key: &str) -> Result<(), RateLimitError> {
        let now = Utc::now();
        
        // Cleanup old entries periodically
        if now - self.last_cleanup > self.cleanup_interval {
            self.cleanup_old_entries(now);
            self.last_cleanup = now;
        }

        // Update global metrics
        self.update_global_metrics(identifier, now);

        // Check for DDoS attacks
        if self.ddos_detection.enabled {
            if let Err(e) = self.check_ddos_detection() {
                return Err(e);
            }
        }

        let config = self.limits.get(limit_key)
            .ok_or_else(|| RateLimitError::RateLimitExceeded(
                "Rate limit configuration not found".to_string()
            ))?;

        let entry = self.entries.entry(identifier.to_string()).or_insert_with(|| RateLimitEntry {
            count: 0,
            first_request: now,
            last_request: now,
            blocked_until: None,
            penalty_count: 0,
        });

        // Check if currently blocked
        if let Some(blocked_until) = entry.blocked_until {
            if now < blocked_until {
                return Err(RateLimitError::RateLimitExceeded(
                    format!("Rate limited until {}", blocked_until)
                ));
            } else {
                entry.blocked_until = None;
            }
        }

        let window_duration = match config.window {
            RateLimitType::PerSecond => Duration::seconds(1),
            RateLimitType::PerMinute => Duration::minutes(1),
            RateLimitType::PerHour => Duration::hours(1),
            RateLimitType::PerDay => Duration::days(1),
            RateLimitType::Custom(duration) => duration,
        };

        // Reset count if window has expired
        if now - entry.first_request > window_duration {
            entry.count = 0;
            entry.first_request = now;
        }

        // Check burst limit
        if let Some(burst_limit) = config.burst_limit {
            if entry.count >= burst_limit {
                entry.blocked_until = Some(now + config.penalty_duration);
                entry.penalty_count += 1;
                return Err(RateLimitError::RateLimitExceeded(
                    "Burst limit exceeded".to_string()
                ));
            }
        }

        // Check regular limit
        if entry.count >= config.limit {
            entry.blocked_until = Some(now + config.penalty_duration);
            entry.penalty_count += 1;
            
            // Auto-block if threshold exceeded
            if let Some(auto_block_threshold) = config.auto_block_threshold {
                if entry.penalty_count >= auto_block_threshold {
                    entry.blocked_until = Some(now + Duration::hours(1));
                    return Err(RateLimitError::DDoSDetected(
                        "Auto-blocked due to repeated violations".to_string()
                    ));
                }
            }
            
            return Err(RateLimitError::RateLimitExceeded(
                format!("Rate limit exceeded: {}/{}", entry.count, config.limit)
            ));
        }

        entry.count += 1;
        entry.last_request = now;

        Ok(())
    }

    fn update_global_metrics(&mut self, identifier: &str, now: DateTime<Utc>) {
        self.global_metrics.total_requests += 1;
        
        // Update time windows
        self.global_metrics.requests_last_minute.push_back(now);
        self.global_metrics.requests_last_hour.push_back(now);
        
        // Keep only last minute
        while let Some(&front) = self.global_metrics.requests_last_minute.front() {
            if now - front > Duration::minutes(1) {
                self.global_metrics.requests_last_minute.pop_front();
            } else {
                break;
            }
        }
        
        // Keep only last hour
        while let Some(&front) = self.global_metrics.requests_last_hour.front() {
            if now - front > Duration::hours(1) {
                self.global_metrics.requests_last_hour.pop_front();
            } else {
                break;
            }
        }
        
        // Update top IPs
        *self.global_metrics.top_ips.entry(identifier.to_string()).or_insert(0) += 1;
    }

    fn check_ddos_detection(&mut self) -> Result<(), RateLimitError> {
        let now = Utc::now();
        let rps = self.global_metrics.requests_last_minute.len() as f64 / 60.0;
        let unique_ips = self.global_metrics.top_ips.len() as u64;

        if rps > self.ddos_detection.threshold_rps || unique_ips > self.ddos_detection.threshold_unique_ips {
            if !self.ddos_detection.attack_detected {
                self.ddos_detection.attack_detected = true;
                self.ddos_detection.attack_start = Some(now);
                
                log::warn!("DDoS attack detected! RPS: {}, Unique IPs: {}", rps, unique_ips);
                
                return Err(RateLimitError::DDoSDetected(
                    format!("DDoS attack detected: {:.2} RPS, {} unique IPs", rps, unique_ips)
                ));
            }
        } else if self.ddos_detection.attack_detected {
            // Attack has subsided
            self.ddos_detection.attack_detected = false;
            self.ddos_detection.attack_start = None;
            log::info!("DDoS attack has subsided");
        }

        Ok(())
    }

    fn cleanup_old_entries(&mut self, now: DateTime<Utc>) {
        let max_duration = Duration::days(1);
        
        self.entries.retain(|_, entry| {
            now - entry.last_request < max_duration || entry.blocked_until.is_some()
        });
        
        // Clean up old IP metrics
        self.global_metrics.top_ips.retain(|_, &mut count| count > 0);
    }

    pub fn get_ddos_metrics(&self) -> DDoSMetrics {
        let _now = Utc::now();
        let rps = self.global_metrics.requests_last_minute.len() as f64 / 60.0;
        
        let mut top_ips: Vec<_> = self.global_metrics.top_ips.iter()
            .map(|(ip, count)| (ip.clone(), *count))
            .collect();
        top_ips.sort_by(|a, b| b.1.cmp(&a.1));
        top_ips.truncate(10);

        DDoSMetrics {
            total_requests: self.global_metrics.total_requests,
            unique_ips: self.global_metrics.top_ips.len() as u64,
            requests_per_second: rps,
            attack_detected: self.ddos_detection.attack_detected,
            attack_start_time: self.ddos_detection.attack_start,
            top_source_ips: top_ips,
        }
    }

    pub fn get_rate_limit_status(&self, identifier: &str) -> Option<&RateLimitEntry> {
        self.entries.get(identifier)
    }

    pub fn reset_rate_limit(&mut self, identifier: &str) {
        self.entries.remove(identifier);
    }

    pub fn block_ip(&mut self, identifier: &str, duration: Duration) {
        if let Some(entry) = self.entries.get_mut(identifier) {
            entry.blocked_until = Some(Utc::now() + duration);
        }
    }

    pub fn is_blocked(&self, identifier: &str) -> bool {
        if let Some(entry) = self.entries.get(identifier) {
            if let Some(blocked_until) = entry.blocked_until {
                return Utc::now() < blocked_until;
            }
        }
        false
    }

    pub fn configure_ddos_detection(&mut self, 
                                   threshold_rps: f64, 
                                   threshold_unique_ips: u64, 
                                   detection_window: Duration) {
        self.ddos_detection.threshold_rps = threshold_rps;
        self.ddos_detection.threshold_unique_ips = threshold_unique_ips;
        self.ddos_detection.detection_window = detection_window;
    }

    pub fn enable_ddos_detection(&mut self, enabled: bool) {
        self.ddos_detection.enabled = enabled;
    }

    pub fn get_global_stats(&self) -> GlobalStats {
        GlobalStats {
            total_requests: self.global_metrics.total_requests,
            requests_last_minute: self.global_metrics.requests_last_minute.len(),
            requests_last_hour: self.global_metrics.requests_last_hour.len(),
            unique_ips: self.global_metrics.top_ips.len(),
            active_rate_limits: self.entries.len(),
            ddos_attack_active: self.ddos_detection.attack_detected,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalStats {
    pub total_requests: u64,
    pub requests_last_minute: usize,
    pub requests_last_hour: usize,
    pub unique_ips: usize,
    pub active_rate_limits: usize,
    pub ddos_attack_active: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_rate_limiting() {
        let mut limiter = RateLimiter::new();
        limiter.add_limit("test".to_string(), RateLimitConfig {
            limit: 5,
            window: RateLimitType::PerMinute,
            burst_limit: None,
            penalty_duration: Duration::minutes(1),
            auto_block_threshold: None,
        });

        // Should allow first 5 requests
        for i in 0..5 {
            assert!(limiter.check_rate_limit("user1", "test").is_ok(), 
                   "Request {} should be allowed", i);
        }

        // 6th request should be rate limited
        assert!(limiter.check_rate_limit("user1", "test").is_err());
    }

    #[test]
    fn test_burst_limiting() {
        let mut limiter = RateLimiter::new();
        limiter.add_limit("test".to_string(), RateLimitConfig {
            limit: 10,
            window: RateLimitType::PerMinute,
            burst_limit: Some(15),
            penalty_duration: Duration::minutes(1),
            auto_block_threshold: None,
        });

        // Should allow burst up to 15
        for i in 0..15 {
            assert!(limiter.check_rate_limit("user1", "test").is_ok(), 
                   "Burst request {} should be allowed", i);
        }

        // 16th request should exceed burst limit
        assert!(limiter.check_rate_limit("user1", "test").is_err());
    }

    #[test]
    fn test_ip_blocking() {
        let mut limiter = RateLimiter::new();
        
        limiter.block_ip("192.168.1.1", Duration::minutes(5));
        
        assert!(limiter.is_blocked("192.168.1.1"));
        assert!(!limiter.is_blocked("192.168.1.2"));
    }

    #[test]
    fn test_ddos_detection() {
        let mut limiter = RateLimiter::new();
        limiter.configure_ddos_detection(10.0, 10, Duration::minutes(1));
        
        // Simulate high traffic
        for i in 0..1000 {
            let user_id = format!("user{}", i % 100);
            limiter.check_rate_limit(&user_id, "test").ok();
        }
        
        let metrics = limiter.get_ddos_metrics();
        assert!(metrics.requests_per_second > 0.0);
    }
}
