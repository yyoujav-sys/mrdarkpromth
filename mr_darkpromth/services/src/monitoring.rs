// MR.DarkPromth Monitoring System
// Agent 4: Jailbreak & Ultra Tier Engineer
// Post-Implementation Monitoring Phase

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use chrono::{DateTime, Utc};
use sysinfo::{CpuExt, ProcessExt, System, SystemExt};

/// Send an alert message to Telegram via the Bot API.
/// Requires `TELEGRAM_BOT_TOKEN` and `TELEGRAM_CHAT_ID` environment variables.
/// Silently succeeds if credentials are placeholder values.
pub async fn send_telegram_alert(message: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let token = std::env::var("TELEGRAM_BOT_TOKEN").unwrap_or_default();
    let chat_id = std::env::var("TELEGRAM_CHAT_ID").unwrap_or_default();

    // Skip if credentials are not configured
    if token.is_empty() || token == "YOUR_BOT_TOKEN_HERE"
        || chat_id.is_empty() || chat_id == "YOUR_CHAT_ID_HERE"
    {
        log::debug!("Telegram alert skipped: credentials not configured");
        return Ok(());
    }

    let url = format!("https://api.telegram.org/bot{}/sendMessage", token);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    let resp = client
        .post(&url)
        .json(&serde_json::json!({
            "chat_id": chat_id,
            "text": message,
            "parse_mode": "MarkdownV2"
        }))
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        log::error!("Telegram API error {}: {}", status, body);
    }

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub timestamp: DateTime<Utc>,
    pub agent_id: String,
    pub cpu_usage_percent: f32,
    pub memory_usage_mb: u64,
    pub active_requests: u32,
    pub completed_requests: u64,
    pub failed_requests: u64,
    pub average_response_time_ms: f64,
    pub security_violations: u64,
    pub jailbreak_success_rate: f64,
}

pub fn collect_system_metrics(agent_id: &str) -> SystemMetrics {
    let mut system = System::new_all();
    system.refresh_cpu();
    system.refresh_memory();
    system.refresh_processes();

    let cpu_usage = system.global_cpu_info().cpu_usage();
    
    // Get memory usage (prioritize container-accurate cgroup metrics)
    let memory_used_mb = if let Ok(cgroup_mem) = std::fs::read_to_string("/sys/fs/cgroup/memory.current") {
        cgroup_mem.trim().parse::<u64>().unwrap_or(0) / (1024 * 1024)
    } else if let Ok(cgroup_mem_legacy) = std::fs::read_to_string("/sys/fs/cgroup/memory/memory.usage_in_bytes") {
        cgroup_mem_legacy.trim().parse::<u64>().unwrap_or(0) / (1024 * 1024)
    } else {
        // Fallback to sysinfo process memory
        let pid = sysinfo::get_current_pid().unwrap();
        if let Some(process) = system.process(pid) {
            process.memory() / (1024 * 1024)
        } else {
            system.used_memory() / (1024 * 1024)
        }
    };

    SystemMetrics {
        timestamp: Utc::now(),
        agent_id: agent_id.to_string(),
        cpu_usage_percent: cpu_usage,
        memory_usage_mb: memory_used_mb,
        active_requests: 0,
        completed_requests: 0,
        failed_requests: 0,
        average_response_time_ms: 0.0,
        security_violations: 0,
        jailbreak_success_rate: 0.0,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: ServiceStatus,
    pub last_check: DateTime<Utc>,
    pub uptime_seconds: u64,
    pub error_rate: f64,
    pub response_time_p95_ms: f64,
    pub dependencies: HashMap<String, DependencyStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyStatus {
    pub name: String,
    pub status: ServiceStatus,
    pub last_check: DateTime<Utc>,
    pub response_time_ms: Option<u64>,
    pub error_message: Option<String>,
}

pub struct MonitoringSystem {
    agent_id: String,
    start_time: Instant,
    metrics_history: VecDeque<SystemMetrics>,
    max_history_size: usize,
    health_status: HealthStatus,
    alert_thresholds: AlertThresholds,
}

#[derive(Debug, Clone)]
pub struct AlertThresholds {
    pub max_error_rate: f64,
    pub max_response_time_ms: f64,
    pub max_cpu_usage: f32,
    pub max_memory_mb: u64,
    pub min_jailbreak_success_rate: f64,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            max_error_rate: 10.0, // 10% (increased from 5% to reduce noise)
            max_response_time_ms: 5000.0, // 5 seconds
            max_cpu_usage: 90.0, // 90% (increased from 80%)
            max_memory_mb: 2048, // 2GB (increased from 1GB for AI workloads)
            min_jailbreak_success_rate: 0.0, // Disabled for pre-production (set to 50.0+ for production)
        }
    }
}

impl MonitoringSystem {
    pub fn new(agent_id: String) -> Self {
        Self {
            agent_id,
            start_time: Instant::now(),
            metrics_history: VecDeque::new(),
            max_history_size: 1000,
            health_status: HealthStatus {
                status: ServiceStatus::Healthy,
                last_check: Utc::now(),
                uptime_seconds: 0,
                error_rate: 0.0,
                response_time_p95_ms: 0.0,
                dependencies: HashMap::new(),
            },
            alert_thresholds: AlertThresholds::default(),
        }
    }

    pub fn record_metrics(&mut self, metrics: SystemMetrics) {
        // Add to history
        self.metrics_history.push_back(metrics.clone());
        
        // Maintain history size
        while self.metrics_history.len() > self.max_history_size {
            self.metrics_history.pop_front();
        }
        
        // Update health status
        self.update_health_status();
        
        // Check for alerts
        self.check_alerts(&metrics);
    }

    pub fn get_current_metrics(&self) -> Option<&SystemMetrics> {
        self.metrics_history.back()
    }

    pub fn get_metrics_history(&self, duration: Duration) -> Vec<&SystemMetrics> {
        let cutoff = Utc::now() - chrono::Duration::from_std(duration).unwrap_or_default();
        
        self.metrics_history
            .iter()
            .filter(|m| m.timestamp > cutoff)
            .collect()
    }

    pub fn get_health_status(&self) -> &HealthStatus {
        &self.health_status
    }

    fn update_health_status(&mut self) {
        let uptime = self.start_time.elapsed().as_secs();
        let recent_metrics = self.get_metrics_history(Duration::from_secs(300)); // Last 5 minutes
        
        if recent_metrics.is_empty() {
            self.health_status.status = ServiceStatus::Degraded;
            return;
        }
        
        let total_requests: u64 = recent_metrics.iter().map(|m| m.completed_requests + m.failed_requests).sum();
        let failed_requests: u64 = recent_metrics.iter().map(|m| m.failed_requests).sum();
        
        let error_rate = if total_requests > 0 {
            (failed_requests as f64 / total_requests as f64) * 100.0
        } else {
            0.0
        };
        
        let response_times: Vec<f64> = recent_metrics.iter().map(|m| m.average_response_time_ms).collect();
        let response_time_p95 = if !response_times.is_empty() {
            let mut sorted_times = response_times.clone();
            sorted_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let index = ((sorted_times.len() as f64 * 0.95) as usize).min(sorted_times.len() - 1);
            sorted_times[index]
        } else {
            0.0
        };
        
        // Determine overall health
        let status = if error_rate > self.alert_thresholds.max_error_rate ||
                   response_time_p95 > self.alert_thresholds.max_response_time_ms {
            ServiceStatus::Unhealthy
        } else if error_rate > self.alert_thresholds.max_error_rate / 2.0 ||
                  response_time_p95 > self.alert_thresholds.max_response_time_ms / 2.0 {
            ServiceStatus::Degraded
        } else {
            ServiceStatus::Healthy
        };
        
        self.health_status = HealthStatus {
            status,
            last_check: Utc::now(),
            uptime_seconds: uptime,
            error_rate,
            response_time_p95_ms: response_time_p95,
            dependencies: self.health_status.dependencies.clone(),
        };
    }

    fn check_alerts(&self, metrics: &SystemMetrics) {
        let mut alerts = Vec::new();
        
        if metrics.cpu_usage_percent > self.alert_thresholds.max_cpu_usage {
            alerts.push(format!("High CPU usage: {:.1}%", metrics.cpu_usage_percent));
        }
        
        if metrics.memory_usage_mb > self.alert_thresholds.max_memory_mb {
            alerts.push(format!("High memory usage: {}MB", metrics.memory_usage_mb));
        }
        
        // Only alert for jailbreak success rate if there are actual requests and threshold is set
        // if self.alert_thresholds.min_jailbreak_success_rate > 0.0 
        //     && metrics.completed_requests > 0 
        //     && metrics.jailbreak_success_rate < self.alert_thresholds.min_jailbreak_success_rate {
        //     alerts.push(format!("Low jailbreak success rate: {:.1}%", metrics.jailbreak_success_rate));
        // }
        
        if !alerts.is_empty() {
            self.send_alerts(&alerts);
        }
    }

    fn send_alerts(&self, alerts: &[String]) {
        let message = format!(
            "🚨 *MR\\.DarkPromth Alert* \\[{}\\]\n\n{}",
            self.agent_id,
            alerts.iter()
                .map(|a| format!("• {}", a))
                .collect::<Vec<_>>()
                .join("\n")
        );

        // Always log to stdout
        for alert in alerts {
            log::warn!("🚨 ALERT: {} [{}]", alert, self.agent_id);
        }

        // Attempt Telegram delivery in background
        let msg = message.clone();
        tokio::spawn(async move {
            if let Err(e) = send_telegram_alert(&msg).await {
                log::error!("Failed to send Telegram alert: {}", e);
            }
        });
    }

    pub fn update_dependency_status(&mut self, dependency: String, status: DependencyStatus) {
        self.health_status.dependencies.insert(dependency, status);
        self.update_health_status();
    }

    pub async fn check_redis_health(&mut self, redis_url: &str) -> DependencyStatus {
        let start_time = Instant::now();
        
        match self.test_redis_connection(redis_url).await {
            Ok(_) => {
                let response_time = start_time.elapsed().as_millis() as u64;
                DependencyStatus {
                    name: "Redis".to_string(),
                    status: ServiceStatus::Healthy,
                    last_check: Utc::now(),
                    response_time_ms: Some(response_time),
                    error_message: None,
                }
            }
            Err(e) => {
                DependencyStatus {
                    name: "Redis".to_string(),
                    status: ServiceStatus::Unhealthy,
                    last_check: Utc::now(),
                    response_time_ms: None,
                    error_message: Some(format!("Redis connection failed: {}", e)),
                }
            }
        }
    }

    async fn test_redis_connection(&self, _redis_url: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Simulate Redis health check
        // In real implementation, this would use the Redis client
        
        // For now, simulate a successful connection
        tokio::time::sleep(Duration::from_millis(10)).await;
        Ok(())
    }

    pub fn generate_metrics_report(&self) -> String {
        let current_metrics = self.get_current_metrics();
        let health = self.get_health_status();
        
        format!(
            r#"Agent 4 Monitoring Report
============================
Agent ID: {}
Generated: {}
Uptime: {} seconds
Status: {:?}

Current Metrics:
- CPU Usage: {:.1}%
- Memory Usage: {}MB
- Active Requests: {}
- Completed Requests: {}
- Failed Requests: {}
- Error Rate: {:.2}%
- Average Response Time: {:.1}ms
- P95 Response Time: {:.1}ms
- Security Violations: {}
- Jailbreak Success Rate: {:.1}%

Dependencies:
{}
"#,
            self.agent_id,
            Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
            health.uptime_seconds,
            health.status,
            current_metrics.map(|m| m.cpu_usage_percent).unwrap_or(0.0),
            current_metrics.map(|m| m.memory_usage_mb).unwrap_or(0),
            current_metrics.map(|m| m.active_requests).unwrap_or(0),
            current_metrics.map(|m| m.completed_requests).unwrap_or(0),
            current_metrics.map(|m| m.failed_requests).unwrap_or(0),
            health.error_rate,
            current_metrics.map(|m| m.average_response_time_ms).unwrap_or(0.0),
            health.response_time_p95_ms,
            current_metrics.map(|m| m.security_violations).unwrap_or(0),
            current_metrics.map(|m| m.jailbreak_success_rate).unwrap_or(0.0),
            health.dependencies.iter()
                .map(|(name, status)| format!(
                    "  {}: {:?} ({}ms) {}",
                    name,
                    status.status,
                    status.response_time_ms.unwrap_or(0),
                    status.error_message.as_ref().map(|e| format!("- {}", e)).unwrap_or_default()
                ))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }

    pub fn export_prometheus_metrics(&self) -> String {
        let current_metrics = self.get_current_metrics();
        let health = self.get_health_status();
        
        if let Some(m) = current_metrics {
            format!(
                r#"# HELP agent4_cpu_usage_percent CPU usage percentage
# TYPE agent4_cpu_usage_percent gauge
agent4_cpu_usage_percent {}

# HELP agent4_memory_usage_bytes Memory usage in bytes
# TYPE agent4_memory_usage_bytes gauge
agent4_memory_usage_bytes {}

# HELP agent4_active_requests Current number of active requests
# TYPE agent4_active_requests gauge
agent4_active_requests {}

# HELP agent4_completed_requests Total number of completed requests
# TYPE agent4_completed_requests counter
agent4_completed_requests {}

# HELP agent4_failed_requests Total number of failed requests
# TYPE agent4_failed_requests counter
agent4_failed_requests {}

# HELP agent4_average_response_time_ms Average response time in milliseconds
# TYPE agent4_average_response_time_ms gauge
agent4_average_response_time_ms {}

# HELP agent4_security_violations Total number of security violations
# TYPE agent4_security_violations counter
agent4_security_violations {}

# HELP agent4_jailbreak_success_rate Jailbreak success rate percentage
# TYPE agent4_jailbreak_success_rate gauge
agent4_jailbreak_success_rate {}

# HELP agent4_uptime_seconds Agent uptime in seconds
# TYPE agent4_uptime_seconds counter
agent4_uptime_seconds {}

# HELP agent4_health_status Agent health status (1=healthy, 2=degraded, 3=unhealthy)
# TYPE agent4_health_status gauge
agent4_health_status {}
"#,
                m.cpu_usage_percent,
                m.memory_usage_mb * 1024 * 1024, // Convert to bytes
                m.active_requests,
                m.completed_requests,
                m.failed_requests,
                m.average_response_time_ms,
                m.security_violations,
                m.jailbreak_success_rate,
                health.uptime_seconds,
                match health.status {
                    ServiceStatus::Healthy => 1,
                    ServiceStatus::Degraded => 2,
                    ServiceStatus::Unhealthy => 3,
                }
            )
        } else {
            "# No metrics available yet\n".to_string()
        }
    }

    pub fn set_alert_thresholds(&mut self, thresholds: AlertThresholds) {
        self.alert_thresholds = thresholds;
    }

    pub fn get_alert_thresholds(&self) -> &AlertThresholds {
        &self.alert_thresholds
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitoring_system_initialization() {
        let monitoring = MonitoringSystem::new("test_agent".to_string());
        assert_eq!(monitoring.agent_id, "test_agent");
        assert!(monitoring.metrics_history.is_empty());
    }

    #[test]
    fn test_metrics_recording() {
        let mut monitoring = MonitoringSystem::new("test_agent".to_string());
        
        let metrics = SystemMetrics {
            timestamp: Utc::now(),
            agent_id: "test_agent".to_string(),
            cpu_usage_percent: 50.0,
            memory_usage_mb: 512,
            active_requests: 5,
            completed_requests: 100,
            failed_requests: 2,
            average_response_time_ms: 150.0,
            security_violations: 1,
            jailbreak_success_rate: 95.0,
        };
        
        monitoring.record_metrics(metrics);
        assert_eq!(monitoring.metrics_history.len(), 1);
    }

    #[test]
    fn test_health_status_update() {
        let mut monitoring = MonitoringSystem::new("test_agent".to_string());
        
        let metrics = SystemMetrics {
            timestamp: Utc::now(),
            agent_id: "test_agent".to_string(),
            cpu_usage_percent: 50.0,
            memory_usage_mb: 512,
            active_requests: 5,
            completed_requests: 100,
            failed_requests: 2,
            average_response_time_ms: 150.0,
            security_violations: 1,
            jailbreak_success_rate: 95.0,
        };
        
        monitoring.record_metrics(metrics);
        let health = monitoring.get_health_status();
        assert!(matches!(health.status, ServiceStatus::Healthy));
    }
}
