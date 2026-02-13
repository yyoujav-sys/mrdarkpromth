// Rate limiting middleware for protecting API endpoints
use axum::{
    extract::ConnectInfo,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::time::{Duration, Instant};

// Prometheus-scrapable rate limit rejection counters
static RATE_LIMIT_REJECTED_TOTAL: AtomicU64 = AtomicU64::new(0);
static RATE_LIMIT_REJECTED_AUTH: AtomicU64 = AtomicU64::new(0);
static RATE_LIMIT_REJECTED_CHAT: AtomicU64 = AtomicU64::new(0);

/// Get current rate limit rejection counts for Prometheus metrics
pub fn get_rate_limit_metrics() -> (u64, u64, u64) {
    (
        RATE_LIMIT_REJECTED_TOTAL.load(Ordering::Relaxed),
        RATE_LIMIT_REJECTED_AUTH.load(Ordering::Relaxed),
        RATE_LIMIT_REJECTED_CHAT.load(Ordering::Relaxed),
    )
}

/// Rate limit configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum requests per window
    pub max_requests: u32,
    
    /// Time window in seconds
    pub window_secs: u64,
    
    /// Whether to enable rate limiting
    pub enabled: bool,
}

impl RateLimitConfig {
    /// Standard rate limit: 100 requests per minute
    pub fn standard() -> Self {
        Self {
            max_requests: 100,
            window_secs: 60,
            enabled: true,
        }
    }

    /// Strict rate limit: 20 requests per minute
    pub fn strict() -> Self {
        Self {
            max_requests: 20,
            window_secs: 60,
            enabled: true,
        }
    }

    /// Lenient rate limit: 1000 requests per minute
    pub fn lenient() -> Self {
        Self {
            max_requests: 1000,
            window_secs: 60,
            enabled: true,
        }
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self::standard()
    }
}

/// Track request counts per IP
struct ClientRateLimit {
    request_count: u32,
    window_start: Instant,
}

/// Rate limiter for IPs
pub struct IpRateLimiter {
    config: RateLimitConfig,
    clients: Arc<RwLock<HashMap<String, ClientRateLimit>>>,
}

impl IpRateLimiter {
    /// Create a new rate limiter with standard configuration
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Check if a client IP is allowed to make a request
    pub async fn check_limit(&self, client_ip: &str) -> bool {
        if !self.config.enabled {
            return true;
        }

        let mut clients = self.clients.write().await;
        let now = Instant::now();
        let window_duration = Duration::from_secs(self.config.window_secs);

        // Get or create client entry
        let client = clients
            .entry(client_ip.to_string())
            .or_insert_with(|| ClientRateLimit {
                request_count: 0,
                window_start: now,
            });

        // Check if window has expired
        if now.duration_since(client.window_start) >= window_duration {
            // Reset counter for new window
            client.request_count = 1;
            client.window_start = now;
            true
        } else {
            // Check if max requests exceeded
            if client.request_count >= self.config.max_requests {
                false
            } else {
                client.request_count += 1;
                true
            }
        }
    }

    /// Get current request count for a client
    pub async fn get_request_count(&self, client_ip: &str) -> u32 {
        let clients = self.clients.read().await;
        clients
            .get(client_ip)
            .map(|c| c.request_count)
            .unwrap_or(0)
    }

    /// Reset all rate limits (for testing)
    #[allow(dead_code)]
    pub async fn reset_all(&self) {
        self.clients.write().await.clear();
    }
}

/// Extract client IP from headers or socket address
fn extract_client_ip(request: &Request<axum::body::Body>) -> String {
    // Try X-Forwarded-For first (behind Nginx proxy)
    if let Some(xff) = request.headers().get("x-forwarded-for") {
        if let Ok(xff_str) = xff.to_str() {
            if let Some(first_ip) = xff_str.split(',').next() {
                return first_ip.trim().to_string();
            }
        }
    }

    // Try X-Real-IP
    if let Some(xri) = request.headers().get("x-real-ip") {
        if let Ok(xri_str) = xri.to_str() {
            return xri_str.trim().to_string();
        }
    }

    // Fallback to ConnectInfo
    if let Some(connect_info) = request.extensions().get::<ConnectInfo<SocketAddr>>() {
        return connect_info.0.ip().to_string();
    }

    "127.0.0.1".to_string()
}

/// General rate limiting middleware (100 req/min)
pub async fn rate_limit_middleware(
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, impl IntoResponse> {
    static LIMITER: std::sync::OnceLock<IpRateLimiter> = std::sync::OnceLock::new();
    let limiter = LIMITER.get_or_init(|| IpRateLimiter::new(RateLimitConfig::standard()));

    let client_ip = extract_client_ip(&request);

    if !limiter.check_limit(&client_ip).await {
        RATE_LIMIT_REJECTED_TOTAL.fetch_add(1, Ordering::Relaxed);
        log::warn!(
            "Rate limit exceeded for IP: {} ({} requests in window)",
            client_ip,
            limiter.get_request_count(&client_ip).await
        );
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            "Too many requests. Please try again later.",
        ));
    }

    Ok(next.run(request).await)
}

/// Strict rate limiting for auth endpoints (10 req/min)
pub async fn auth_rate_limit_middleware(
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, impl IntoResponse> {
    static AUTH_LIMITER: std::sync::OnceLock<IpRateLimiter> = std::sync::OnceLock::new();
    let limiter = AUTH_LIMITER.get_or_init(|| IpRateLimiter::new(RateLimitConfig {
        max_requests: 10,
        window_secs: 60,
        enabled: true,
    }));

    let client_ip = extract_client_ip(&request);

    if !limiter.check_limit(&client_ip).await {
        RATE_LIMIT_REJECTED_TOTAL.fetch_add(1, Ordering::Relaxed);
        RATE_LIMIT_REJECTED_AUTH.fetch_add(1, Ordering::Relaxed);
        log::warn!("Auth rate limit exceeded for IP: {}", client_ip);
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            "Too many authentication attempts. Please try again later.",
        ));
    }

    Ok(next.run(request).await)
}

/// Chat rate limiting (30 req/min)
pub async fn chat_rate_limit_middleware(
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, impl IntoResponse> {
    static CHAT_LIMITER: std::sync::OnceLock<IpRateLimiter> = std::sync::OnceLock::new();
    let limiter = CHAT_LIMITER.get_or_init(|| IpRateLimiter::new(RateLimitConfig {
        max_requests: 30,
        window_secs: 60,
        enabled: true,
    }));

    let client_ip = extract_client_ip(&request);

    if !limiter.check_limit(&client_ip).await {
        RATE_LIMIT_REJECTED_TOTAL.fetch_add(1, Ordering::Relaxed);
        RATE_LIMIT_REJECTED_CHAT.fetch_add(1, Ordering::Relaxed);
        log::warn!("Chat rate limit exceeded for IP: {}", client_ip);
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            "Chat rate limit exceeded. Please slow down.",
        ));
    }

    Ok(next.run(request).await)
}

/// Per-endpoint rate limit configurations
pub struct RateLimitPerEndpoint {
    pub auth_login: RateLimitConfig,
    pub auth_register: RateLimitConfig,
    pub chat: RateLimitConfig,
    pub billing: RateLimitConfig,
    pub admin: RateLimitConfig,
}

impl Default for RateLimitPerEndpoint {
    fn default() -> Self {
        Self {
            auth_login: RateLimitConfig::strict(),      // 20 req/min - prevent brute force
            auth_register: RateLimitConfig::strict(),   // 20 req/min
            chat: RateLimitConfig::standard(),          // 100 req/min
            billing: RateLimitConfig::strict(),         // 20 req/min - sensitive
            admin: RateLimitConfig::very_strict(),      // 10 req/min - admin only
        }
    }
}

impl RateLimitConfig {
    /// Very strict rate limit: 10 requests per minute (for sensitive endpoints)
    pub fn very_strict() -> Self {
        Self {
            max_requests: 10,
            window_secs: 60,
            enabled: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_allows_within_limit() {
        let limiter = IpRateLimiter::new(RateLimitConfig {
            max_requests: 1000,
            window_secs: 60,
            enabled: true,
        });

        let client_ip = "192.168.1.1";

        for _ in 0..5 {
            assert!(limiter.check_limit(client_ip).await);
        }
    }

    #[tokio::test]
    async fn test_rate_limiter_blocks_over_limit() {
        let limiter = IpRateLimiter::new(RateLimitConfig {
            max_requests: 3,
            window_secs: 60,
            enabled: true,
        });

        let client_ip = "192.168.1.1";

        // Use up the limit
        for _ in 0..3 {
            assert!(limiter.check_limit(client_ip).await);
        }

        // Next request should be blocked
        assert!(!limiter.check_limit(client_ip).await);
    }

    #[tokio::test]
    async fn test_rate_limiter_resets_on_window_expiry() {
        let limiter = IpRateLimiter::new(RateLimitConfig {
            max_requests: 1,
            window_secs: 1,  // 1 second window
            enabled: true,
        });

        let client_ip = "192.168.1.1";

        // First request allowed
        assert!(limiter.check_limit(client_ip).await);

        // Second request blocked
        assert!(!limiter.check_limit(client_ip).await);

        // Wait for window to expire
        tokio::time::sleep(Duration::from_millis(1100)).await;

        // Should be allowed again
        assert!(limiter.check_limit(client_ip).await);
    }

    #[test]
    fn test_rate_limit_configs() {
        let standard = RateLimitConfig::standard();
        assert_eq!(standard.max_requests, 100);

        let strict = RateLimitConfig::strict();
        assert_eq!(strict.max_requests, 20);

        let lenient = RateLimitConfig::lenient();
        assert_eq!(lenient.max_requests, 1000);
    }
}
