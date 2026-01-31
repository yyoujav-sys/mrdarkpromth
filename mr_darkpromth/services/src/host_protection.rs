// MR.DarkPromth Host Protection Module
// Centralized policy for blocking internal hosts/domains

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostProtectionPolicy {
    blocked_hosts: HashSet<String>,
    #[serde(skip)]
    blocked_patterns: Vec<Regex>,
    blocked_cidrs: Vec<String>,
}

impl HostProtectionPolicy {
    pub fn new() -> Self {
        let mut policy = Self {
            blocked_hosts: HashSet::new(),
            blocked_patterns: Vec::new(),
            blocked_cidrs: Vec::new(),
        };

        policy.initialize_blocked_hosts();
        policy.initialize_blocked_patterns();
        policy.initialize_blocked_cidrs();

        policy
    }

    fn initialize_blocked_hosts(&mut self) {
        let hosts = vec![
            "localhost",
            "127.0.0.1",
            "::1",
            "0.0.0.0",
            "169.254.169.254",
            "metadata.google.internal",
            "metadata.googleapis.com",
            "metadata.google.internal",
            "metadata.aws.amazon.com",
            "169.254.169.254",
        ];

        for host in hosts {
            self.blocked_hosts.insert(host.to_string());
        }
    }

    fn initialize_blocked_patterns(&mut self) {
        let patterns = vec![
            r"(?i)^https?://localhost",
            r"(?i)^https?://127\.0\.0\.1",
            r"(?i)^https?://::1",
            r"(?i)^https?://0\.0\.0\.0",
            r"(?i)^https?://10\.\d+\.\d+\.\d+",
            r"(?i)^https?://192\.168\.\d+\.\d+",
            r"(?i)^https?://172\.(1[6-9]|2\d|3[01])\.\d+\.\d+",
            r"(?i)^https?://169\.254\.\d+\.\d+",
            r"(?i)^https?://169\.254\.169\.254",
            r"(?i)^https?://metadata\.google\.internal",
            r"(?i)^https?://metadata\.amazonaws\.com/latest/meta-data",
        ];

        for pattern in patterns {
            if let Ok(regex) = Regex::new(pattern) {
                self.blocked_patterns.push(regex);
            }
        }
    }

    fn initialize_blocked_cidrs(&mut self) {
        let cidrs = vec![
            "10.0.0.0/8",
            "172.16.0.0/12",
            "192.168.0.0/16",
            "169.254.0.0/16",
            "127.0.0.0/8",
            "::1/128",
            "0.0.0.0/8",
        ];

        for cidr in cidrs {
            self.blocked_cidrs.push(cidr.to_string());
        }
    }

    pub fn is_host_blocked(&self, host: &str) -> bool {
        // Check exact match
        if self.blocked_hosts.contains(host) {
            return true;
        }

        // Check patterns
        for pattern in &self.blocked_patterns {
            if pattern.is_match(host) {
                return true;
            }
        }

        // Check CIDR ranges (simplified check)
        for cidr in &self.blocked_cidrs {
            if self.is_in_cidr(host, cidr) {
                return true;
            }
        }

        false
    }

    fn is_in_cidr(&self, host: &str, cidr: &str) -> bool {
        // Simplified CIDR check - in production, use proper IP library
        match cidr {
            "10.0.0.0/8" => {
                if let Ok(ip) = host.parse::<std::net::Ipv4Addr>() {
                    let octets = ip.octets();
                    return octets[0] == 10;
                }
            }
            "172.16.0.0/12" => {
                if let Ok(ip) = host.parse::<std::net::Ipv4Addr>() {
                    let octets = ip.octets();
                    return octets[0] == 172 && (octets[1] >= 16 && octets[1] <= 31);
                }
            }
            "192.168.0.0/16" => {
                if let Ok(ip) = host.parse::<std::net::Ipv4Addr>() {
                    let octets = ip.octets();
                    return octets[0] == 192 && octets[1] == 168;
                }
            }
            "169.254.0.0/16" => {
                if let Ok(ip) = host.parse::<std::net::Ipv4Addr>() {
                    let octets = ip.octets();
                    return octets[0] == 169 && octets[1] == 254;
                }
            }
            "127.0.0.0/8" => {
                if let Ok(ip) = host.parse::<std::net::Ipv4Addr>() {
                    let octets = ip.octets();
                    return octets[0] == 127;
                }
            }
            "::1/128" => {
                return host == "::1";
            }
            "0.0.0.0/8" => {
                if let Ok(ip) = host.parse::<std::net::Ipv4Addr>() {
                    let octets = ip.octets();
                    return octets[0] == 0;
                }
            }
            _ => {}
        }
        false
    }

    pub fn extract_host_from_url(&self, url: &str) -> Option<String> {
        if let Ok(parsed) = Url::parse(url) {
            if let Some(host) = parsed.host_str() {
                return Some(host.to_string());
            }
        }
        None
    }

    pub fn is_url_blocked(&self, url: &str) -> bool {
        if let Some(host) = self.extract_host_from_url(url) {
            self.is_host_blocked(&host)
        } else {
            false
        }
    }
}

impl Default for HostProtectionPolicy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_localhost_blocked() {
        let policy = HostProtectionPolicy::new();
        assert!(policy.is_host_blocked("localhost"));
        assert!(policy.is_host_blocked("127.0.0.1"));
    }

    #[test]
    fn test_external_host_allowed() {
        let policy = HostProtectionPolicy::new();
        assert!(!policy.is_host_blocked("example.com"));
        assert!(!policy.is_host_blocked("api.openai.com"));
    }

    #[test]
    fn test_private_ip_blocked() {
        let policy = HostProtectionPolicy::new();
        assert!(policy.is_host_blocked("192.168.1.1"));
        assert!(policy.is_host_blocked("10.0.0.1"));
        assert!(policy.is_host_blocked("172.16.0.1"));
    }

    #[test]
    fn test_metadata_blocked() {
        let policy = HostProtectionPolicy::new();
        assert!(policy.is_host_blocked("169.254.169.254"));
        assert!(policy.is_host_blocked("metadata.google.internal"));
    }

    #[test]
    fn test_url_blocking() {
        let policy = HostProtectionPolicy::new();
        assert!(policy.is_url_blocked("http://localhost:8080/api"));
        assert!(policy.is_url_blocked("https://127.0.0.1/health"));
        assert!(!policy.is_url_blocked("https://example.com"));
    }
}
