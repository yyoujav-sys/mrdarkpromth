// MR.DarkPromth Telemetry Service
// Real-time metrics broadcasting

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::{broadcast, Mutex};
use tokio::time::{self, Duration};
use serde::{Serialize, Deserialize};
use crate::monitoring::{MonitoringSystem, collect_system_metrics, SystemMetrics};

/// Request-level metrics for API tracking
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RequestMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub total_latency_ms: u64,
    pub avg_latency_ms: f64,
    pub requests_per_second: f64,
    pub error_rate: f64,
}

/// Atomic counters for thread-safe request tracking
pub struct RequestCounters {
    total: AtomicU64,
    success: AtomicU64,
    failed: AtomicU64,
    total_latency_ms: AtomicU64,
    last_window_requests: AtomicU64,
    last_window_time_ms: AtomicU64,
}

impl Default for RequestCounters {
    fn default() -> Self {
        Self {
            total: AtomicU64::new(0),
            success: AtomicU64::new(0),
            failed: AtomicU64::new(0),
            total_latency_ms: AtomicU64::new(0),
            last_window_requests: AtomicU64::new(0),
            last_window_time_ms: AtomicU64::new(chrono::Utc::now().timestamp_millis() as u64),
        }
    }
}

impl RequestCounters {
    pub fn record(&self, latency_ms: u64, success: bool) {
        self.total.fetch_add(1, Ordering::Relaxed);
        self.total_latency_ms.fetch_add(latency_ms, Ordering::Relaxed);
        
        if success {
            self.success.fetch_add(1, Ordering::Relaxed);
        } else {
            self.failed.fetch_add(1, Ordering::Relaxed);
        }
    }
    
    pub fn get_metrics(&self) -> RequestMetrics {
        let total = self.total.load(Ordering::Relaxed);
        let success = self.success.load(Ordering::Relaxed);
        let failed = self.failed.load(Ordering::Relaxed);
        let total_latency = self.total_latency_ms.load(Ordering::Relaxed);
        
        let avg_latency = if total > 0 {
            total_latency as f64 / total as f64
        } else {
            0.0
        };
        
        let error_rate = if total > 0 {
            (failed as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        
        // Calculate requests per second (using sliding window)
        let now_ms = chrono::Utc::now().timestamp_millis() as u64;
        let last_window_time = self.last_window_time_ms.load(Ordering::Relaxed);
        let last_window_reqs = self.last_window_requests.load(Ordering::Relaxed);
        
        let elapsed_ms = now_ms.saturating_sub(last_window_time);
        let new_requests = total.saturating_sub(last_window_reqs);
        
        let rps = if elapsed_ms > 0 {
            (new_requests as f64 / elapsed_ms as f64) * 1000.0
        } else {
            0.0
        };
        
        // Update sliding window every 5 seconds
        if elapsed_ms > 5000 {
            self.last_window_requests.store(total, Ordering::Relaxed);
            self.last_window_time_ms.store(now_ms, Ordering::Relaxed);
        }
        
        RequestMetrics {
            total_requests: total,
            successful_requests: success,
            failed_requests: failed,
            total_latency_ms: total_latency,
            avg_latency_ms: avg_latency,
            requests_per_second: rps,
            error_rate,
        }
    }
}

pub struct TelemetryService {
    agent_id: String,
    monitoring: Arc<Mutex<MonitoringSystem>>,
    broadcast_tx: broadcast::Sender<SystemMetrics>,
    request_counters: Arc<RequestCounters>,
}

impl TelemetryService {
    pub fn new(agent_id: String) -> Self {
        let (tx, _) = broadcast::channel(100);
        Self {
            agent_id: agent_id.clone(),
            monitoring: Arc::new(Mutex::new(MonitoringSystem::new(agent_id))),
            broadcast_tx: tx,
            request_counters: Arc::new(RequestCounters::default()),
        }
    }

    /// Record a request completion for metrics tracking
    pub fn record_request(&self, latency_ms: u64, success: bool) {
        self.request_counters.record(latency_ms, success);
    }
    
    /// Get current request metrics
    pub fn get_request_metrics(&self) -> RequestMetrics {
        self.request_counters.get_metrics()
    }
    
    /// Get counters Arc for middleware use
    pub fn get_counters(&self) -> Arc<RequestCounters> {
        self.request_counters.clone()
    }

    pub async fn start_background_collection(&self) {
        let monitoring = self.monitoring.clone();
        let tx = self.broadcast_tx.clone();
        let agent_id = self.agent_id.clone();
        
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(2)); // Record every 2 seconds
            
            loop {
                interval.tick().await;
                let metrics = collect_system_metrics(&agent_id);
                
                {
                    let mut m = monitoring.lock().await;
                    m.record_metrics(metrics.clone());
                }

                // Broadcast metrics to all listeners
                let _ = tx.send(metrics);
            }
        });
    }

    pub fn subscribe(&self) -> broadcast::Receiver<SystemMetrics> {
        self.broadcast_tx.subscribe()
    }

    pub async fn get_current_metrics(&self) -> Option<SystemMetrics> {
        let m = self.monitoring.lock().await;
        m.get_current_metrics().cloned()
    }

    pub async fn get_history(&self, duration_secs: u64) -> Vec<SystemMetrics> {
        let m = self.monitoring.lock().await;
        m.get_metrics_history(Duration::from_secs(duration_secs)).into_iter().cloned().collect()
    }
    
    pub async fn get_dashboard_summary(&self) -> serde_json::Value {
        let m = self.monitoring.lock().await;
        let current = m.get_current_metrics();
        let request_metrics = self.get_request_metrics();
        
        if let Some(metrics) = current {
            serde_json::json!({
                "agent_id": self.agent_id,
                "status": "active",
                "system": {
                    "cpu_usage": metrics.cpu_usage_percent,
                    "memory_usage_mb": metrics.memory_usage_mb,
                },
                "requests": {
                    "total": request_metrics.total_requests,
                    "successful": request_metrics.successful_requests,
                    "failed": request_metrics.failed_requests,
                    "avg_latency_ms": request_metrics.avg_latency_ms,
                    "requests_per_second": request_metrics.requests_per_second,
                    "error_rate": request_metrics.error_rate,
                },
                "timestamp": metrics.timestamp
            })
        } else {
            serde_json::json!({
                "agent_id": self.agent_id,
                "status": "initializing",
                "requests": request_metrics,
                "timestamp": chrono::Utc::now()
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_telemetry_initialization() {
        let agent_id = "test_agent".to_string();
        let service = TelemetryService::new(agent_id.clone());
        
        let summary = service.get_dashboard_summary().await;
        assert_eq!(summary["agent_id"], agent_id);
        assert_eq!(summary["status"], "initializing");
    }

    #[tokio::test]
    async fn test_telemetry_metrics_flow() {
        let agent_id = "test_agent_flow".to_string();
        let service = TelemetryService::new(agent_id.clone());
        let mut rx = service.subscribe();
        
        // Start collection
        service.start_background_collection().await;
        
        // Wait for first metric
        let timeout = tokio::time::timeout(Duration::from_secs(5), rx.recv()).await;
        assert!(timeout.is_ok(), "Timed out waiting for metrics");
        
        let metrics = timeout.unwrap().unwrap();
        assert_eq!(metrics.agent_id, agent_id);
        
        // Check dashboard summary again
        let summary = service.get_dashboard_summary().await;
        assert_eq!(summary["status"], "active");
        assert!(summary["system"]["cpu_usage"].is_f64());
        // Verify requests section exists
        assert!(summary.get("requests").is_some());
    }
}
