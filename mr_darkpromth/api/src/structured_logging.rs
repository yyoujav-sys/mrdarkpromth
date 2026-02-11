// Structured JSON logging for production
use serde_json::json;
use std::sync::atomic::{AtomicU64, Ordering};

/// Request/Response tracking information
#[derive(Debug, Clone)]
pub struct RequestContext {
    pub request_id: String,
    pub method: String,
    pub path: String,
    pub user_id: Option<String>,
    pub timestamp: String,
    pub client_ip: String,
}

/// Log a structured request event
pub fn log_request(ctx: &RequestContext) {
    let log_data = json!({
        "event": "http_request",
        "request_id": ctx.request_id,
        "method": ctx.method,
        "path": ctx.path,
        "user_id": ctx.user_id,
        "timestamp": ctx.timestamp,
        "client_ip": ctx.client_ip,
    });

    log::info!("{}", log_data);
}

/// Log a structured response event
pub fn log_response(
    request_id: &str,
    status: u16,
    duration_ms: u64,
    user_id: Option<String>,
) {
    let log_data = json!({
        "event": "http_response",
        "request_id": request_id,
        "status": status,
        "duration_ms": duration_ms,
        "user_id": user_id,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    log::info!("{}", log_data);
}

/// Log security events
pub fn log_security_event(event_type: &str, details: serde_json::Value, user_id: Option<String>) {
    let log_data = json!({
        "event": "security",
        "event_type": event_type,
        "details": details,
        "user_id": user_id,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    log::warn!("{}", log_data);
}

/// Log database operations
pub fn log_database_operation(
    operation: &str,
    table: &str,
    duration_ms: u64,
    success: bool,
    error: Option<String>,
) {
    let log_data = json!({
        "event": "database",
        "operation": operation,
        "table": table,
        "duration_ms": duration_ms,
        "success": success,
        "error": error,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    if success {
        log::debug!("{}", log_data);
    } else {
        log::error!("{}", log_data);
    }
}

/// Log payment/billing events
pub fn log_billing_event(event_type: &str, user_id: &str, amount: Option<f64>, details: serde_json::Value) {
    let log_data = json!({
        "event": "billing",
        "event_type": event_type,
        "user_id": user_id,
        "amount": amount,
        "details": details,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    log::info!("{}", log_data);
}

/// Log authentication events
pub fn log_auth_event(event_type: &str, user_id: Option<String>, success: bool, reason: Option<String>) {
    let log_data = json!({
        "event": "authentication",
        "event_type": event_type,
        "user_id": user_id,
        "success": success,
        "reason": reason,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    if success {
        log::info!("{}", log_data);
    } else {
        log::warn!("{}", log_data);
    }
}

/// Performance metrics collector
pub struct PerformanceMetrics {
    pub total_requests: AtomicU64,
    pub successful_requests: AtomicU64,
    pub failed_requests: AtomicU64,
    pub total_request_time_ms: AtomicU64,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            total_requests: AtomicU64::new(0),
            successful_requests: AtomicU64::new(0),
            failed_requests: AtomicU64::new(0),
            total_request_time_ms: AtomicU64::new(0),
        }
    }

    pub fn record_request(&self, success: bool, duration_ms: u64) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        if success {
            self.successful_requests.fetch_add(1, Ordering::Relaxed);
        } else {
            self.failed_requests.fetch_add(1, Ordering::Relaxed);
        }
        self.total_request_time_ms.fetch_add(duration_ms, Ordering::Relaxed);
    }

    pub fn get_stats(&self) -> serde_json::Value {
        let total = self.total_requests.load(Ordering::Relaxed);
        let successful = self.successful_requests.load(Ordering::Relaxed);
        let failed = self.failed_requests.load(Ordering::Relaxed);
        let total_time = self.total_request_time_ms.load(Ordering::Relaxed);

        let avg_time = if total > 0 { total_time / total } else { 0 };

        json!({
            "total_requests": total,
            "successful_requests": successful,
            "failed_requests": failed,
            "success_rate": if total > 0 { (successful as f64 / total as f64 * 100.0) as u32 } else { 0 },
            "average_response_time_ms": avg_time,
        })
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_metrics() {
        let metrics = PerformanceMetrics::new();
        metrics.record_request(true, 100);
        metrics.record_request(true, 150);
        metrics.record_request(false, 200);

        let stats = metrics.get_stats();
        assert_eq!(stats["total_requests"], 3);
        assert_eq!(stats["successful_requests"], 2);
        assert_eq!(stats["failed_requests"], 1);
    }
}
