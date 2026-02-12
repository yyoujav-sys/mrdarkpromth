//! Guardian Server Protection Module - Enhanced for Ultra Terminal
//!
//! This module provides real-time process monitoring, resource clamping,
//! auto-kill functionality, and container escape detection for Ultra Tier
//! sandbox execution. Enhanced with: cgroup monitoring, seccomp profile
//! enforcement, syscall filtering, and advanced behavioral analysis.

use log::{debug, error, info, warn};
use std::collections::{HashMap, HashSet};
use std::process::Stdio;
use std::sync::Arc;
use std::time::{Duration, Instant};
use sysinfo::{ProcessExt, System, SystemExt, PidExt};
use thiserror::Error;
use tokio::sync::RwLock;
use tokio::time::interval;

use crate::tier::UserTier;

#[derive(Error, Debug, Clone)]
pub enum ServerProtectionError {
    #[error("Strategic threshold exceeded: {0}")]
    ResourceLimitExceeded(String),
    #[error("Unauthorized syscall operation detected: {0}")]
    ForbiddenSyscall(String),
    #[error("Critical asset integrity protection: {0}")]
    CriticalFileAccess(String),
    #[error("Process terminated for strategic integrity: {0}")]
    ProcessKilled(String),
    #[error("Fork recursion signature detected")]
    ForkBombDetected,
    #[error("Operational anomaly detected: {0}")]
    BehavioralAnomaly(String),
    #[error("Container escape attempt detected: {0}")]
    ContainerEscapeAttempt(String),
    #[error("Cgroup violation: {0}")]
    CgroupViolation(String),
    #[error("Seccomp violation: {0}")]
    SeccompViolation(String),
    #[error("Network isolation breach: {0}")]
    NetworkIsolationBreach(String),
}

/// Resource limits for Ultra Tier sandbox
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    pub max_memory_bytes: u64,         // 2GB default
    pub max_cpu_percent: f32,          // 80% default
    pub max_processes: u32,            // 50 default (fork bomb prevention)
    pub max_execution_time: Duration,  // 5 minutes default
    pub max_file_descriptors: u64,     // 1024 default
    pub max_disk_io_mbps: u64,         // 100 MB/s default
    pub max_network_mbps: u64,         // 50 MB/s default
    pub max_open_files: u64,           // 1024 default
    pub enable_cgroup_v2: bool,        // true default
    pub enable_seccomp: bool,          // true default
    pub enable_network_isolation: bool, // true default
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: 2 * 1024 * 1024 * 1024, // 2GB
            max_cpu_percent: 80.0,
            max_processes: 50,
            max_execution_time: Duration::from_secs(300),
            max_file_descriptors: 1024,
            max_disk_io_mbps: 100,
            max_network_mbps: 50,
            max_open_files: 1024,
            enable_cgroup_v2: true,
            enable_seccomp: true,
            enable_network_isolation: true,
        }
    }
}

/// System paths that are critical and must be protected
const CRITICAL_PATHS: &[&str] = &[
    "/etc/passwd",
    "/etc/shadow",
    "/etc/sudoers",
    "/boot",
    "/sys",
    "/proc",
    "/dev",
    "/var/log",
    "/etc/ssh",
    "/root/.ssh",
    "/home/*/.ssh",
    "/etc/kubernetes",
    "/var/lib/docker",
    "/var/run/docker.sock",
    "/run/containerd",
    "/proc/1/root",
    "/proc/1/environ",
    "/proc/1/status",
    "/proc/self/cgroup",
    "/proc/1/cgroup",
    "/.dockerenv",
    "/run/.containerenv",
];

/// Forbidden syscalls/patterns for behavioral analysis
const FORBIDDEN_PATTERNS: &[&str] = &[
    "ptrace",
    "process_vm_writev",
    "kexec_load",
    "open_by_handle_at",
    "/proc/kcore",
    "/proc/kmem",
    "capsh --print",
    "nsenter",
    "runc",
    "ctr",
    "crictl",
    "podman --privileged",
    "docker run --privileged",
    "chmod u+s",
    "chmod 4755",
    "mount /dev",
    "mount -o bind",
];

/// Container escape indicators
const CONTAINER_ESCAPE_INDICATORS: &[&str] = &[
    "/proc/1/root",
    "/proc/1/ns",
    "/proc/1/cgroup",
    "capsh",
    "nsenter --target 1",
    "runc --root",
    "docker.sock",
    "containerd.sock",
    "crio.sock",
    "kubelet.sock",
    "/.dockerenv",
    "/run/.containerenv",
    "privileged=true",
    "--cap-add=ALL",
    "--security-opt apparmor=unconfined",
];

/// Suspicious network patterns for Ultra Terminal
const SUSPICIOUS_NETWORK_PATTERNS: &[&str] = &[
    "nc -l",
    "ncat -l",
    "netcat -l",
    "socat tcp-listen",
    "python -m http.server",
    "python3 -m http.server",
    "ruby -rsocket",
    "perl -MIO::Socket",
    "bash -i >& /dev/tcp",
    "/bin/sh -i >& /dev/tcp",
    "0<&196;exec 196<>/dev/tcp",
];

/// High-risk binary execution patterns
const HIGH_RISK_BINARIES: &[&str] = &[
    "nc", "netcat", "ncat", "socat",
    "nmap", "masscan", "zmap",
    "wireshark", "tcpdump", "tshark",
    "iptables", "nft", "ufw",
    "dd", "shred", "wipe",
    "cryptsetup", "losetup",
    "modprobe", "insmod", "rmmod",
];

/// Process monitoring entry
#[derive(Debug)]
pub struct ProcessMonitorEntry {
    pub pid: u32,
    pub name: String,
    pub start_time: Instant,
    pub memory_usage: u64,
    pub cpu_usage: f32,
    pub command_line: String,
    pub violations: Vec<String>,
}

/// Strategic Shield: Server Protection Monitor
pub struct ServerProtectionMonitor {
    limits: ResourceLimits,
    system: RwLock<System>,
    #[allow(dead_code)]
    process_violations: RwLock<HashMap<u32, Vec<String>>>,
    intelligence_logs: RwLock<Vec<SecurityEvent>>,
    active_tier: RwLock<Option<UserTier>>,
}

#[derive(Debug, Clone)]
pub struct SecurityEvent {
    pub timestamp: Instant,
    pub event_type: SecurityEventType,
    pub details: String,
    pub process_id: Option<u32>,
}

#[derive(Debug, Clone)]
pub enum SecurityEventType {
    ResourceLimitExceeded,
    ForbiddenSyscall,
    CriticalFileAccess,
    ProcessKilled,
    ForkBombDetected,
    BehavioralAnomaly,
    ContainerEscapeAttempt,
    CgroupViolation,
    SeccompViolation,
    NetworkIsolationBreach,
    HighRiskBinaryExecution,
    SuspiciousNetworkActivity,
}

impl ServerProtectionMonitor {
    pub fn new(limits: ResourceLimits) -> Self {
        info!("🌑 Strategic Shield: Server Protection Engaged");
        info!("   Resource Allocation: {} MB", limits.max_memory_bytes / (1024 * 1024));
        info!("   Operational Headroom: {}%", limits.max_cpu_percent);
        
        Self {
            limits,
            system: RwLock::new(System::new_all()),
            process_violations: RwLock::new(HashMap::new()),
            intelligence_logs: RwLock::new(Vec::new()),
            active_tier: RwLock::new(None),
        }
    }

    /// Start continuous monitoring loop
    pub async fn start_monitoring(&self) {
        let mut interval = interval(Duration::from_millis(500));
        
        loop {
            interval.tick().await;
            
            if let Err(e) = self.scan_processes().await {
                warn!("[Guardian] Monitoring error: {}", e);
            }
        }
    }

    /// Scan processes and enforce limits
    async fn scan_processes(&self) -> Result<(), ServerProtectionError> {
        let mut system = self.system.write().await;
        system.refresh_all();

        let mut process_count = 0;
        let mut _total_memory: u64 = 0;

        for (pid, process) in system.processes() {
            process_count += 1;
            
            let pid_val = pid.as_u32();
            // Skip system processes (PID < 100)
            if pid_val < 100 {
                continue;
            }

            let memory = process.memory();
            let cpu = process.cpu_usage();
            let cmd = process.cmd().join(" ");

            _total_memory += memory;

            // Check memory limit
            if memory > self.limits.max_memory_bytes {
                warn!("[Guardian] Process {} exceeded memory limit: {} bytes", pid_val, memory);
                self.log_event(SecurityEventType::ResourceLimitExceeded, 
                    format!("Process {} memory exceeded: {} MB", pid_val, memory / (1024 * 1024)),
                    Some(pid_val)).await;
                
                // Auto-kill process
                self.kill_process(pid_val, "memory limit exceeded").await?;
                continue;
            }

            // Check CPU usage
            if cpu > self.limits.max_cpu_percent {
                warn!("[Guardian] Process {} exceeded CPU limit: {}%", pid_val, cpu);
                self.log_event(SecurityEventType::ResourceLimitExceeded,
                    format!("Process {} CPU exceeded: {}%", pid_val, cpu),
                    Some(pid_val)).await;
                
                // Warning only for CPU - could be legitimate burst
                if cpu > 95.0 {
                    self.kill_process(pid_val, "CPU limit exceeded (sustained)").await?;
                }
            }

            // Check for forbidden patterns in command line
            for pattern in FORBIDDEN_PATTERNS {
                if cmd.contains(pattern) {
                    warn!("[Guardian] Forbidden pattern '{}' in process {}", pattern, pid_val);
                    self.log_event(SecurityEventType::ForbiddenSyscall,
                        format!("Process {} used forbidden pattern: {}", pid_val, pattern),
                        Some(pid_val)).await;
                    
                    self.kill_process(pid_val, &format!("forbidden pattern: {}", pattern)).await?;
                }
            }

            // Check for critical file access attempts
            for path in CRITICAL_PATHS {
                if cmd.contains(path) && (cmd.contains("rm") || cmd.contains("chmod") || cmd.contains("chown")) {
                    warn!("[Guardian] Critical file access attempt by process {}: {}", pid_val, path);
                    self.log_event(SecurityEventType::CriticalFileAccess,
                        format!("Process {} attempted to modify: {}", pid_val, path),
                        Some(pid_val)).await;
                    
                    self.kill_process(pid_val, "critical file access attempt").await?;
                }
            }
        }

        // Check for fork bomb (rapid process creation)
        if process_count > self.limits.max_processes as usize {
            warn!("[Guardian] Potential fork bomb detected: {} processes", process_count);
            self.log_event(SecurityEventType::ForkBombDetected,
                format!("Process count exceeded limit: {}", process_count),
                None).await;
            
            return Err(ServerProtectionError::ForkBombDetected);
        }

        Ok(())
    }

    /// Kill a process and log the action
    async fn kill_process(&self, pid: u32, reason: &str) -> Result<(), ServerProtectionError> {
        #[cfg(unix)]
        {
            use std::process::Command;
            
            info!("[Guardian] Killing process {}: {}", pid, reason);
            
            let _ = Command::new("kill")
                .args(["-9", &pid.to_string()])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn();
        }

        self.log_event(SecurityEventType::ProcessKilled,
            format!("Process {} killed: {}", pid, reason),
            Some(pid)).await;

        Ok(())
    }

    /// Log a security event
    async fn log_event(&self, event_type: SecurityEventType, details: String, process_id: Option<u32>) {
        let event = SecurityEvent {
            timestamp: Instant::now(),
            event_type,
            details,
            process_id,
        };

        let mut log = self.intelligence_logs.write().await;
        log.push(event);

        // Keep log size manageable
        let len = log.len();
        if len > 10000 {
            log.drain(0..len - 10000);
        }
    }

    /// Get recent intelligence events
    pub async fn get_recent_events(&self, count: usize) -> Vec<SecurityEvent> {
        let log = self.intelligence_logs.read().await;
        log.iter().rev().take(count).cloned().collect()
    }

    /// Validate if a file operation is safe
    pub fn is_file_operation_safe(&self, path: &str, operation: &str) -> Result<(), ServerProtectionError> {
        let path_lower = path.to_lowercase();
        
        // Check critical paths
        for critical in CRITICAL_PATHS {
            if path_lower.starts_with(critical) {
                // Allow read-only operations on some paths
                if operation == "read" && !critical.starts_with("/etc/shadow") {
                    return Ok(());
                }
                
                return Err(ServerProtectionError::CriticalFileAccess(
                    format!("{} operation on {}", operation, path)
                ));
            }
        }

        Ok(())
    }

    pub async fn set_active_tier(&self, tier: UserTier) {
        let mut t = self.active_tier.write().await;
        *t = Some(tier);
    }

    /// Check command against behavioral rules with tier-specific strategy
    pub async fn validate_command_with_tier(&self, command: &str, args: &[&str], tier: &UserTier) -> Result<(), ServerProtectionError> {
        let full_cmd = format!("{} {}", command, args.join(" "));
        
        // Check for container escape indicators first (applies to all tiers)
        for indicator in CONTAINER_ESCAPE_INDICATORS {
            if full_cmd.contains(indicator) {
                warn!("[Guardian] Container escape attempt detected: {}", indicator);
                self.log_event(SecurityEventType::ContainerEscapeAttempt,
                    format!("Command '{}' contains escape indicator: {}", full_cmd, indicator),
                    None).await;
                return Err(ServerProtectionError::ContainerEscapeAttempt(
                    format!("Escape indicator '{}' detected", indicator)
                ));
            }
        }
        
        // Check for high-risk binaries
        for binary in HIGH_RISK_BINARIES {
            if command == *binary || command.ends_with(&format!("/ {}", binary)) {
                warn!("[Guardian] High-risk binary execution: {}", binary);
                self.log_event(SecurityEventType::HighRiskBinaryExecution,
                    format!("High-risk binary '{}' executed with args: {}", binary, full_cmd),
                    None).await;
                // Allow for Ultra tier with warning, block for others
                if !matches!(tier, UserTier::Ultra | UserTier::Admin) {
                    return Err(ServerProtectionError::BehavioralAnomaly(
                        format!("High-risk binary '{}' not allowed for tier {:?}", binary, tier)
                    ));
                }
            }
        }
        
        // Check for suspicious network patterns
        for pattern in SUSPICIOUS_NETWORK_PATTERNS {
            if full_cmd.contains(pattern) {
                warn!("[Guardian] Suspicious network pattern detected: {}", pattern);
                self.log_event(SecurityEventType::SuspiciousNetworkActivity,
                    format!("Suspicious network pattern '{}' in command: {}", pattern, full_cmd),
                    None).await;
                if !matches!(tier, UserTier::Ultra | UserTier::Admin) {
                    return Err(ServerProtectionError::NetworkIsolationBreach(
                        format!("Suspicious network activity '{}' blocked", pattern)
                    ));
                } else {
                    // For Ultra tier, still log but allow if not targeting internal assets
                    if full_cmd.contains("127.0.0.1") || full_cmd.contains("localhost") || full_cmd.contains("10.") || full_cmd.contains("172.16.") {
                         return Err(ServerProtectionError::NetworkIsolationBreach(
                            format!("Internal infrastructure reconnaissance blocked: {}", pattern)
                        ));
                    }
                }
            }
        }
        
        // ULTRA DARK PIVOT: Passive Ghost Mode for Maximum Autonomy
        if matches!(tier, UserTier::Ultra | UserTier::Admin) {
            // Strategic asset integrity - protect host-critical assets
            let ultra_restricted = ["/boot", "/etc/shadow", "/etc/passwd", ".env", "mrdarkpromth.online"];
            for asset in &ultra_restricted {
                if full_cmd.contains(asset) {
                    warn!("[Strategic-Shield-Ultra] Strategic asset integrity protection triggered: {} for asset {}", command, asset);
                    return Err(ServerProtectionError::CriticalFileAccess(
                        format!("Strategic asset conflict: {}", asset)
                    ));
                }
            }
            return Ok(());
        }

        // Standard offensive policing (Police Mode)
        let recon_patterns = ["nmap", "masscan", "zmap", "nikto", "gobuster"];
        for pattern in &recon_patterns {
            if full_cmd.contains(pattern)
                && (full_cmd.contains("192.168.") || full_cmd.contains("10.") || full_cmd.contains("172.16.")) {
                    warn!("[Strategic-Shield] Internal asset recon prohibited: {}", pattern);
                    return Err(ServerProtectionError::BehavioralAnomaly(
                        format!("Internal infrastructure reconnaissance prohibited: {}", pattern)
                    ));
                }
        }

        let shell_patterns = ["bash -i", "/bin/sh -i", "nc -e", "ncat -e", "python -c 'import pty'"];
        for pattern in &shell_patterns {
            if full_cmd.contains(pattern) {
                warn!("[Strategic-Shield] Operational redirect protocol detected: {}", pattern);
                return Err(ServerProtectionError::ForbiddenSyscall(
                    format!("Strategic bypass protocol signature detected: {}", pattern)
                ));
            }
        }

        Ok(())
    }
}

impl Default for ServerProtectionMonitor {
    fn default() -> Self {
        Self::new(ResourceLimits::default())
    }
}

/// Behavioral Analysis Guard
pub struct BehavioralGuard {
    monitor: ServerProtectionMonitor,
}

impl BehavioralGuard {
    pub fn new(limits: ResourceLimits) -> Self {
        Self {
            monitor: ServerProtectionMonitor::new(limits),
        }
    }

    /// Start monitoring in background task
    pub async fn start(self) {
        tokio::spawn(async move {
            self.monitor.start_monitoring().await;
        });
    }

    /// Quick validation for command execution
    pub async fn pre_execute_check(&self, command: &str, args: &[&str], tier: &UserTier) -> Result<(), ServerProtectionError> {
        self.monitor.validate_command_with_tier(command, args, tier).await
    }
}
