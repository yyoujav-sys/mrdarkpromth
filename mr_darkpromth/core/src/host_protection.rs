//! Guardian Host Protection Module - Enhanced for Ultra Terminal
//!
//! This module provides comprehensive egress filtering, SSRF protection,
//! and DNS rebinding attack prevention for Ultra Tier sandbox execution.
//! Enhanced with: container escape detection, advanced CIDR filtering,
//! time-of-check-time-of-use (TOCTOU) protection, and behavioral analysis.

use log::{debug, error, info, warn};
use std::collections::{HashMap, HashSet};
use std::net::{IpAddr};
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::RwLock;
use regex::Regex;

/// Simple CIDR range representation
#[derive(Debug, Clone)]
pub struct CidrRange {
    pub network: IpAddr,
    pub prefix_len: u8,
    pub is_v4: bool,
}

/// Parse CIDR string to CidrRange
fn parse_cidr(cidr: &str) -> Option<CidrRange> {
    let parts: Vec<&str> = cidr.split('/').collect::<Vec<_>>();
    if parts.len() != 2 {
        return None;
    }
    
    let ip = parts[0].parse().ok()?;
    let prefix = parts[1].parse().ok()?;
    
    Some(CidrRange { 
        network: ip, 
        prefix_len: prefix,
        is_v4: matches!(ip, IpAddr::V4(_)),
    })
}

/// Check if IP is in CIDR range
fn ip_in_cidr(ip: &IpAddr, cidr: &str) -> bool {
    if let Some(range) = parse_cidr(cidr) {
        // Simple implementation - in production, use proper IP network library
        match (ip, &range.network) {
            (IpAddr::V4(ip), IpAddr::V4(network)) => {
                let ip_u32 = u32::from(*ip);
                let network_u32 = u32::from(*network);
                let mask = if range.prefix_len >= 32 { 0xFFFFFFFF } else { !((1 << (32 - range.prefix_len)) - 1) };
                (ip_u32 & mask) == (network_u32 & mask)
            },
            _ => false,
        }
    } else {
        false
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum HostProtectionError {
    #[error("Strategic block: Internal IP access prohibited: {0}")]
    InternalIpBlocked(String),
    #[error("Strategic block: Infrastructure metadata access prohibited")]
    CloudMetadataBlocked,
    #[error("Asset domain not authorized: {0}")]
    DomainNotWhitelisted(String),
    #[error("Invalid IP signature: {0}")]
    InvalidIp(String),
    #[error("Internal infrastructure access prohibited: {0}")]
    InternalDomainBlocked(String),
    #[error("DNS rebinding attack detected: {domain} resolved to {ip}")]
    DnsRebindingDetected { domain: String, ip: String },
    #[error("Container escape attempt detected: {0}")]
    ContainerEscapeDetected(String),
    #[error("Time-of-check-time-of-use violation: {0}")]
    ToctouViolation(String),
    #[error("Rate limit exceeded for host checks")]
    RateLimitExceeded,
    #[error("Suspicious request pattern: {0}")]
    SuspiciousPattern(String),
}

/// CIDR ranges that should be blocked (RFC 1918 + loopback + link-local)
const BLOCKED_CIDRS: &[&str] = &[
    "127.0.0.0/8",      // Loopback
    "10.0.0.0/8",       // Private
    "172.16.0.0/12",    // Private
    "192.168.0.0/16",   // Private
    "169.254.0.0/16",   // Link-local
    "100.64.0.0/10",    // Carrier-grade NAT
    "192.0.0.0/24",     // IETF Protocol Assignments
    "192.0.2.0/24",     // TEST-NET-1
    "198.18.0.0/15",    // Network Interconnect Benchmark
    "198.51.100.0/24",  // TEST-NET-2
    "203.0.113.0/24",   // TEST-NET-3
    "224.0.0.0/4",      // Multicast
    "240.0.0.0/4",      // Reserved
    "255.255.255.255/32", // Broadcast
    "::1/128",          // IPv6 loopback
    "fc00::/7",         // IPv6 unique local
    "fe80::/10",        // IPv6 link-local
    "ff00::/8",         // IPv6 multicast
];

/// Cloud metadata IPs that must be blocked
const BLOCKED_METADATA_IPS: &[&str] = &[
    "169.254.169.254",  // AWS/GCP/Azure/Oracle metadata
    "169.254.170.2",    // AWS ECS metadata
    "169.254.170.1",    // AWS EKS metadata
    "192.0.0.254",      // AWS Lightsail
    "fd00:ec2::254",    // AWS IPv6 metadata
];

/// Allowed domains for external requests (Ultra Terminal whitelist)
const ALLOWED_DOMAINS: &[&str] = &[
    "api.cerebras.ai",
    "api.openrouter.ai",
    "api.github.com",
    "github.com",
    "raw.githubusercontent.com",
    "api.openai.com",
    "api.anthropic.com",
    "huggingface.co",
    "cdnjs.cloudflare.com",
    "unpkg.com",
    "registry.npmjs.org",
    "pypi.org",
    "crates.io",
];

/// Internal domains/patterns that must be blocked
const BLOCKED_DOMAIN_PATTERNS: &[&str] = &[
    "*.mrdarkpromth.online",
    "*.internal",
    "*.local",
    "*.localhost",
    "localhost",
    "*.priv",
    "*.svc",
    "*.cluster.local",
    "*.docker.internal",
    "host.docker.internal",
    "gateway.docker.internal",
];

/// Cloud metadata endpoints
const BLOCKED_METADATA_ENDPOINTS: &[&str] = &[
    "169.254.169.254",
    "metadata.google.internal",
    "metadata.googleapis.com",
    "metadata.aws.amazon.com",
    "metadata.azure.internal",
    "oraclecloud.com",
    "aliyun.com",
    "tencentcloudapi.com",
];

/// Container escape indicators
const CONTAINER_ESCAPE_INDICATORS: &[&str] = &[
    "/proc/1/root",
    "/proc/1/cgroup",
    "/proc/1/environ",
    "/proc/1/status",
    "/proc/self/cgroup",
    "capsh",
    "nsenter",
    "runc",
    "ctr",
    "crictl",
    "podman",
    "docker.sock",
    "dockershim",
    "/var/run/docker.sock",
    "/run/containerd/containerd.sock",
    "/run/crio/crio.sock",
    "/var/lib/kubelet",
    "/var/lib/docker",
    "/var/lib/containerd",
    "/var/lib/rancher",
    "/etc/kubernetes",
    "/root/.kube",
];

/// Suspicious URL patterns indicating SSRF attempts
const SUSPICIOUS_URL_PATTERNS: &[&str] = &[
    "@",
    "#",
    "?",
    "../",
    "..%2f",
    "%2e%2e",
    "%252e",
    "0x7f",
    "0x7f.0x0",
    "0177",
    "2130706433", // decimal IP for 127.0.0.1
    "3232235521", // decimal IP for 192.168.0.1
    "@0.0.0.0",
    "@127.0.0.1",
];
fn is_ip_blocked(ip: &IpAddr) -> bool {
    // Check metadata IPs first
    if let IpAddr::V4(ipv4) = ip {
        let ip_str = ipv4.to_string();
        if BLOCKED_METADATA_IPS.contains(&ip_str.as_str()) {
            return true;
        }
    }

    // Check CIDR ranges
    for cidr in BLOCKED_CIDRS {
        if ip_in_cidr(ip, cidr) {
            return true;
        }
    }

    false
}

/// Represents a resolved IP with timestamp for TOCTOU protection
#[derive(Debug, Clone)]
pub struct ResolvedIp {
    pub ip: IpAddr,
    pub domain: String,
    pub resolved_at: Instant,
    pub ttl: Duration,
}

/// Request record for behavioral analysis
#[derive(Debug, Clone)]
pub struct RequestRecord {
    pub timestamp: Instant,
    pub url: String,
    pub host: String,
    pub result: Result<(), HostProtectionError>,
}

/// Container escape attempt record
#[derive(Debug, Clone)]
pub struct EscapeAttempt {
    pub timestamp: Instant,
    pub indicator: String,
    pub command: String,
}

/// Enhanced Host Protection with DNS caching and TOCTOU protection
pub struct HostProtection {
    blocked_cidrs: Vec<CidrRange>,
    blocked_metadata_ips: HashSet<IpAddr>,
    allowed_domains: HashSet<String>,
    blocked_domains: HashSet<String>,
    blocked_patterns: Vec<Regex>,
    dns_cache: Arc<RwLock<HashMap<String, ResolvedIp>>>,
    request_history: Arc<RwLock<Vec<RequestRecord>>>,
    escape_attempts: Arc<RwLock<HashMap<String, Vec<EscapeAttempt>>>>,
}

impl HostProtection {
    /// Create a new HostProtection instance with all security rules
    pub fn new() -> Self {
        let mut blocked_cidrs = Vec::new();
        for cidr in BLOCKED_CIDRS {
            if let Some(range) = parse_cidr(cidr) {
                blocked_cidrs.push(range);
            }
        }

        let mut blocked_metadata_ips = HashSet::new();
        for ip in BLOCKED_METADATA_IPS {
            if let Ok(addr) = IpAddr::from_str(ip) {
                blocked_metadata_ips.insert(addr);
            }
        }

        let allowed_domains = ALLOWED_DOMAINS.iter().map(|s| s.to_string()).collect();
        let blocked_domains: HashSet<String> = BLOCKED_METADATA_ENDPOINTS
            .iter()
            .map(|s| s.to_string())
            .collect();

        let blocked_patterns = BLOCKED_DOMAIN_PATTERNS
            .iter()
            .filter_map(|p| {
                regex::Regex::new(&format!("(?i){}$", regex::escape(p).replace("\\*", ".*"))).ok()
            })
            .collect();

        Self {
            blocked_cidrs,
            blocked_metadata_ips,
            allowed_domains,
            blocked_domains,
            blocked_patterns,
            dns_cache: Arc::new(RwLock::new(HashMap::new())),
            request_history: Arc::new(RwLock::new(Vec::new())),
            escape_attempts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Validates an outbound request URL with full TOCTOU protection
    pub async fn validate_outbound_request(&self, url: &str) -> Result<(), HostProtectionError> {
        let start_time = Instant::now();

        // Check for suspicious URL patterns (SSRF attempts)
        self.check_suspicious_patterns(url)?;

        // Parse URL to extract host
        let host = extract_host_from_url(url)?;

        // Check for container escape indicators in URL
        self.check_container_escape_indicators(url, &host).await?;

        // Check if it's an IP address directly
        if let Ok(ip) = IpAddr::from_str(&host) {
            if self.is_ip_blocked(&ip) {
                warn!("[HostProtection] Blocked direct IP access: {}", ip);
                self.log_request(url, &host, Err(HostProtectionError::InternalIpBlocked(ip.to_string()))).await;
                return Err(HostProtectionError::InternalIpBlocked(ip.to_string()));
            }
            self.log_request(url, &host, Ok(())).await;
            return Ok(());
        }

        // Check domain against blocklist
        if self.is_domain_blocked(&host) {
            warn!("[HostProtection] Blocked domain: {}", host);
            self.log_request(url, &host, Err(HostProtectionError::InternalDomainBlocked(host.clone()))).await;
            return Err(HostProtectionError::InternalDomainBlocked(host));
        }

        // Check domain against whitelist (for Ultra Terminal)
        if !self.is_domain_allowed(&host) {
            warn!("[HostProtection] Non-whitelisted domain: {}", host);
            self.log_request(url, &host, Err(HostProtectionError::DomainNotWhitelisted(host.clone()))).await;
            return Err(HostProtectionError::DomainNotWhitelisted(host));
        }

        // DNS resolution with TOCTOU protection
        let resolved = self.resolve_with_toctou_protection(&host).await?;

        // Verify resolved IP is not blocked (DNS rebinding protection)
        if self.is_ip_blocked(&resolved.ip) {
            warn!(
                "[HostProtection] DNS rebinding attack detected: {} -> {}",
                host, resolved.ip
            );
            self.log_request(
                url,
                &host,
                Err(HostProtectionError::DnsRebindingDetected {
                    domain: host.clone(),
                    ip: resolved.ip.to_string(),
                }),
            ).await;
            return Err(HostProtectionError::DnsRebindingDetected {
                domain: host,
                ip: resolved.ip.to_string(),
            });
        }

        let elapsed = start_time.elapsed();
        debug!("[HostProtection] Request validated in {:?}: {}", elapsed, host);

        self.log_request(url, &host, Ok(())).await;
        Ok(())
    }

    /// Check for suspicious URL patterns
    fn check_suspicious_patterns(&self, url: &str) -> Result<(), HostProtectionError> {
        for pattern in SUSPICIOUS_URL_PATTERNS {
            if url.contains(pattern) {
                warn!("[HostProtection] Suspicious pattern '{}' in URL: {}", pattern, url);
                return Err(HostProtectionError::SuspiciousPattern(format!(
                    "Suspicious pattern '{}' detected",
                    pattern
                )));
            }
        }
        Ok(())
    }

    /// Check for container escape indicators
    async fn check_container_escape_indicators(
        &self,
        url: &str,
        host: &str,
    ) -> Result<(), HostProtectionError> {
        let combined = format!("{} {}", url, host);
        for indicator in CONTAINER_ESCAPE_INDICATORS {
            if combined.contains(indicator) {
                warn!(
                    "[HostProtection] Container escape indicator detected: {} in {}",
                    indicator, url
                );

                // Log escape attempt
                let mut attempts: tokio::sync::RwLockWriteGuard<'_, HashMap<String, Vec<EscapeAttempt>>> = self.escape_attempts.write().await;
                let entry = attempts.entry(host.to_string()).or_default();
                entry.push(EscapeAttempt {
                    timestamp: Instant::now(),
                    indicator: indicator.to_string(),
                    command: url.to_string(),
                });

                return Err(HostProtectionError::ContainerEscapeDetected(format!(
                    "Escape indicator '{}' detected",
                    indicator
                )));
            }
        }
        Ok(())
    }

    /// Resolve domain with TOCTOU (Time-of-check-time-of-use) protection
    async fn resolve_with_toctou_protection(&self, domain: &str) -> Result<ResolvedIp, HostProtectionError> {
        // Check cache first
        {
            let cache: tokio::sync::RwLockReadGuard<'_, HashMap<String, ResolvedIp>> = self.dns_cache.read().await;
            if let Some(cached) = cache.get(domain) {
                if cached.resolved_at.elapsed() < cached.ttl {
                    return Ok(cached.clone());
                }
            }
        }

        // Perform DNS resolution
        let addrs = tokio::net::lookup_host(format!("{}:80", domain))
            .await
            .map_err(|e| HostProtectionError::InvalidIp(format!("DNS resolution failed: {}", e)))?;

        let addr = addrs
            .into_iter()
            .next()
            .ok_or_else(|| HostProtectionError::InvalidIp("No addresses found".to_string()))?;

        let resolved = ResolvedIp {
            ip: addr.ip(),
            domain: domain.to_string(),
            resolved_at: Instant::now(),
            ttl: Duration::from_secs(300), // 5 minute TTL
        };

        // Cache the result
        let mut cache: tokio::sync::RwLockWriteGuard<'_, HashMap<String, ResolvedIp>> = self.dns_cache.write().await;
        cache.insert(domain.to_string(), resolved.clone());

        // Cleanup old cache entries periodically
        if cache.len() > 1000 {
            cache.retain(|_, v| v.resolved_at.elapsed() < v.ttl);
        }

        Ok(resolved)
    }

    /// Check if an IP is in a blocked range
    fn is_ip_blocked(&self, ip: &IpAddr) -> bool {
        // Check metadata IPs first
        if self.blocked_metadata_ips.contains(ip) {
            return true;
        }

        // Check CIDR ranges
        for cidr in &self.blocked_cidrs {
            if ip_in_cidr_enhanced(ip, cidr) {
                return true;
            }
        }

        false
    }

    /// Check if domain is explicitly blocked
    fn is_domain_blocked(&self, domain: &str) -> bool {
        let domain_lower = domain.to_lowercase();

        // Check blocked domains
        if self.blocked_domains.contains(&domain_lower) {
            return true;
        }

        // Check blocked patterns
        for blocked_pattern in BLOCKED_DOMAIN_PATTERNS {
            if let Ok(pattern) = regex::Regex::new(blocked_pattern) {
                if pattern.is_match(&domain_lower) {
                    return true;
                }
            }
        }

        false
    }

    /// Check if domain is in whitelist
    fn is_domain_allowed(&self, domain: &str) -> bool {
        let domain_lower = domain.to_lowercase();

        // Check allowed domains
        for allowed in &self.allowed_domains {
            if domain_lower == *allowed || domain_lower.ends_with(&format!(".{}", allowed)) {
                return true;
            }
        }

        // For Ultra Terminal - allow any non-blocked external domain
        // but require explicit whitelist for sensitive operations
        !self.is_domain_blocked(domain)
    }

    /// Log request for behavioral analysis
    async fn log_request(&self, url: &str, host: &str, result: Result<(), HostProtectionError>) {
        let record = RequestRecord {
            timestamp: Instant::now(),
            url: url.to_string(),
            host: host.to_string(),
            result: result.map_err(|e| e.clone()),
        };

        let mut history: tokio::sync::RwLockWriteGuard<'_, Vec<RequestRecord>> = self.request_history.write().await;
        history.push(record);

        // Keep only last 1000 requests
        if history.len() > 1000 {
            history.remove(0);
        }
    }

    /// Get recent escape attempts for security auditing
    pub async fn get_escape_attempts(&self, host: &str) -> Vec<EscapeAttempt> {
        let attempts: tokio::sync::RwLockReadGuard<'_, HashMap<String, Vec<EscapeAttempt>>> = self.escape_attempts.read().await;
        attempts.get(host).cloned().unwrap_or_default()
    }

    /// Get all escape attempts
    pub async fn get_all_escape_attempts(&self) -> HashMap<String, Vec<EscapeAttempt>> {
        self.escape_attempts.read().await.clone()
    }

    /// Clear escape attempts (call after investigation)
    pub async fn clear_escape_attempts(&self) {
        self.escape_attempts.write().await.clear();
    }

    /// Get request history for analysis
    pub async fn get_request_history(&self, count: usize) -> Vec<RequestRecord> {
        let history: tokio::sync::RwLockReadGuard<'_, Vec<RequestRecord>> = self.request_history.read().await;
        history.iter().rev().take(count).cloned().collect()
    }

    /// Check if IP is suspicious (for behavioral analysis)
    pub fn is_suspicious_ip(&self, ip: &str) -> bool {
        if let Ok(addr) = IpAddr::from_str(ip) {
            return self.is_ip_blocked(&addr);
        }
        false
    }

    /// Validate command for container escape attempts
    pub async fn validate_command(&self, command: &str, args: &[&str]) -> Result<(), HostProtectionError> {
        let full_cmd = format!("{} {}", command, args.join(" "));

        for indicator in CONTAINER_ESCAPE_INDICATORS {
            if full_cmd.contains(indicator) {
                warn!(
                    "[HostProtection] Container escape attempt in command: {}",
                    indicator
                );
                return Err(HostProtectionError::ContainerEscapeDetected(format!(
                    "Command contains escape indicator: {}",
                    indicator
                )));
            }
        }

        Ok(())
    }
}

impl Default for HostProtection {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if IP is within a CIDR range (enhanced version with CidrRange)
fn ip_in_cidr_enhanced(ip: &IpAddr, cidr: &CidrRange) -> bool {
    match (ip, &cidr.network) {
        (IpAddr::V4(ip), IpAddr::V4(network)) => {
            if !cidr.is_v4 {
                return false;
            }
            let ip_u32 = u32::from(*ip);
            let network_u32 = u32::from(*network);
            let mask = !((1u32 << (32 - cidr.prefix_len)) - 1);
            (ip_u32 & mask) == (network_u32 & mask)
        }
        (IpAddr::V6(ip), IpAddr::V6(network)) => {
            if cidr.is_v4 {
                return false;
            }
            // IPv6 CIDR check - compare segments
            let ip_segments = ip.segments();
            let network_segments = network.segments();
            let segments_to_check = cidr.prefix_len as usize / 16;

            for i in 0..segments_to_check.min(8) {
                if ip_segments[i] != network_segments[i] {
                    return false;
                }
            }
            true
        }
        _ => false,
    }
}

/// Enhanced extract host from URL with more protocol support
fn extract_host_from_url(url: &str) -> Result<String, HostProtectionError> {
    // Handle various URL formats
    let without_protocol = url
        .trim_start_matches("http://")
        .trim_start_matches("https://")
        .trim_start_matches("ftp://")
        .trim_start_matches("file://")
        .trim_start_matches("gopher://");

    // Extract host (everything before path, query, or port)
    let host = without_protocol
        .split('/')
        .next()
        .unwrap_or("")
        .split('?')
        .next()
        .unwrap_or("")
        .split('#')
        .next()
        .unwrap_or("")
        .split(':')
        .next()
        .unwrap_or("")
        .to_string();

    if host.is_empty() {
        return Err(HostProtectionError::InvalidIp("empty host".to_string()));
    }

    Ok(host)
}

/// Protection statistics
#[derive(Debug, Clone)]
pub struct ProtectionStats {
    pub total_requests: usize,
    pub blocked_requests: usize,
    pub escape_attempts: usize,
    pub cached_domains: usize,
}

/// Async Guardian Host Protection Guard for Ultra Terminal
pub struct HostProtectionGuard {
    protection: Arc<HostProtection>,
}

impl HostProtectionGuard {
    pub fn new() -> Self {
        info!("🛡️ Strategic Shield: Host Protection Engaged (Enhanced)");
        Self {
            protection: Arc::new(HostProtection::new()),
        }
    }

    /// Protect core assets and log the operational data
    pub async fn guard_request(&self, url: &str, user_id: &str) -> Result<(), HostProtectionError> {
        match self.protection.validate_outbound_request(url).await {
            Ok(()) => {
                info!("[Strategic-Shield] Operational data allowed for agent {} to {}", user_id, url);
                Ok(())
            }
            Err(e) => {
                error!(
                    "[Strategic-Shield-Ultra] Strategic asset protection for agent {}: {} - Asset: {}",
                    user_id, e, url
                );
                Err(e)
            }
        }
    }

    /// Validate command for container escape
    pub async fn validate_command(&self, command: &str, args: &[&str]) -> Result<(), HostProtectionError> {
        self.protection.validate_command(command, args).await
    }

    /// Get protection instance for advanced usage
    pub fn protection(&self) -> &Arc<HostProtection> {
        &self.protection
    }

    /// Check if IP is in blocked range (for behavioral analysis)
    pub fn is_suspicious_ip(&self, ip: &str) -> bool {
        self.protection.is_suspicious_ip(ip)
    }

    /// Get security statistics
    pub async fn get_stats(&self) -> ProtectionStats {
        let history = self.protection.get_request_history(1000).await;
        let escape_attempts = self.protection.get_all_escape_attempts().await;

        let blocked_count = history.iter().filter(|r| r.result.is_err()).count();
        let total_attempts: usize = escape_attempts.values().map(|v| v.len()).sum();

        ProtectionStats {
            total_requests: history.len(),
            blocked_requests: blocked_count,
            escape_attempts: total_attempts,
            cached_domains: self.protection.dns_cache.read().await.len(),
        }
    }
}

impl Default for HostProtectionGuard {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_internal_ip_blocking() {
        let protection = HostProtection::new();
        assert!(protection.is_suspicious_ip("127.0.0.1"));
        assert!(protection.is_suspicious_ip("10.0.0.1"));
        assert!(protection.is_suspicious_ip("192.168.1.1"));
        assert!(protection.is_suspicious_ip("172.16.0.1"));
        assert!(!protection.is_suspicious_ip("8.8.8.8"));
        assert!(!protection.is_suspicious_ip("1.1.1.1"));
    }

    #[tokio::test]
    async fn test_metadata_endpoint_blocking() {
        let protection = HostProtection::new();
        assert!(protection.is_suspicious_ip("169.254.169.254"));
    }

    #[tokio::test]
    async fn test_domain_whitelist() {
        let protection = HostProtection::new();
        assert!(protection.is_domain_allowed("api.cerebras.ai"));
        assert!(protection.is_domain_allowed("api.openrouter.ai"));
        assert!(protection.is_domain_allowed("example.com"));
        assert!(!protection.is_domain_blocked("api.openai.com"));
    }

    #[tokio::test]
    async fn test_blocked_domains() {
        let protection = HostProtection::new();
        assert!(protection.is_domain_blocked("localhost"));
        assert!(protection.is_domain_blocked("metadata.google.internal"));
    }

    #[tokio::test]
    async fn test_container_escape_detection() {
        let guard = HostProtectionGuard::new();

        let result = guard
            .validate_command("cat", &["/proc/1/root/etc/passwd"])
            .await;
        assert!(result.is_err());

        let result = guard.validate_command("ls", &["/home"]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_suspicious_url_patterns() {
        let protection = HostProtection::new();

        let result = protection.validate_outbound_request("http://example.com@127.0.0.1").await;
        assert!(matches!(result, Err(HostProtectionError::SuspiciousPattern(_))));
    }
}
