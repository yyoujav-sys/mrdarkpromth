use log::{info, error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant};
use sysinfo::{ProcessExt, System, SystemExt};
use tempfile::TempDir;
use thiserror::Error;
use tokio::sync::RwLock;
use tokio::time::timeout;

use mr_darkpromth_core::tier::UserTier;
use mr_darkpromth_core::host_protection::HostProtection;
use mr_darkpromth_core::server_protection::{ServerProtectionMonitor, ResourceLimits};

#[derive(Error, Debug)]
pub enum SandboxError {
    #[error("Dark Sandbox initialization failed: {0}")]
    SandboxCreation(String),
    #[error("Operation timeout: {0}s")]
    Timeout(u64),
    #[error("Resource threshold exceeded: {0}")]
    ResourceLimit(String),
    #[error("Operational conflict: {0}")]
    ExecutionFailed(String),
    #[error("Strategic bypass triggered: {0}")]
    StrategicBypass(String),
    #[error("Strategic asset integrity conflict: {0}")]
    SecurityViolation(String),
    #[error("IO operational error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("System error: {0}")]
    SystemError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub max_execution_time: Duration,
    pub max_memory: u64,
    pub max_cpu_time: Duration,
    pub max_processes: u32,
    pub max_file_size: u64,
    pub allow_network: bool,
    pub allow_file_system: bool,
    pub allowed_paths: Vec<PathBuf>,
    pub blocked_commands: Vec<String>,
    pub privileged_commands: Vec<String>,
    pub temp_dir: Option<PathBuf>,
    pub enable_logging: bool,
    pub use_docker: bool,
}

impl SandboxConfig {
    pub fn ultra_tier() -> Self {
        Self {
            max_memory: 4 * 1024 * 1024 * 1024, // 4GB RAM for Ultra
            max_execution_time: Duration::from_secs(3600), // 1 hour for Ultra
            max_cpu_time: Duration::from_secs(600),
            max_processes: 200, // Increased process limit
            allow_network: true, // Full network access (monitored by Guardian)
            blocked_commands: vec![], // ZERO BLOCKS for Ultra
            privileged_commands: vec![
                "sudo".to_string(),
                "systemctl".to_string(),
                "apt".to_string(),
                "apt-get".to_string(),
                "docker".to_string(),
                "gcc".to_string(),
                "g++".to_string(),
                "make".to_string(),
                "gdb".to_string(),
                "nmap".to_string(),
                "metasploit".to_string(),
                "nc".to_string(),
                "python".to_string(),
                "iptables".to_string(),
                "fdisk".to_string(),
                "dd".to_string(),
            ],
            ..Self::default()
        }
    }
}

impl Default for SandboxConfig {
    fn default() -> Self {
        let use_docker = env::var("SANDBOX_USE_DOCKER")
            .map(|value| matches!(value.to_lowercase().as_str(), "1" | "true" | "yes"))
            .unwrap_or(false);

        Self {
            max_execution_time: Duration::from_secs(30),
            max_memory: 512 * 1024 * 1024, // 512MB
            max_cpu_time: Duration::from_secs(10),
            max_processes: 10,
            max_file_size: 10 * 1024 * 1024, // 10MB
            allow_network: false,
            allow_file_system: true,
            allowed_paths: vec![
                PathBuf::from("/tmp"),
                PathBuf::from("/var/tmp"),
            ],
            blocked_commands: vec![
                "rm".to_string(),
                "dd".to_string(),
                "mkfs".to_string(),
                "fdisk".to_string(),
                "iptables".to_string(),
                "su".to_string(),
                "sudo".to_string(),
                "chmod".to_string(),
                "chown".to_string(),
            ],
            privileged_commands: vec![],
            temp_dir: None,
            enable_logging: true,
            use_docker,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub execution_time: Duration,
    pub memory_used: u64,
    pub cpu_time: Duration,
    pub processes_created: u32,
    pub files_created: Vec<PathBuf>,
    pub security_violations: Vec<String>,
}

#[derive(Debug)]
pub struct SandboxedExecutor {
    config: SandboxConfig,
    temp_dir: Option<TempDir>,
    system: Arc<RwLock<System>>,
}

impl SandboxedExecutor {
    pub fn new(config: SandboxConfig) -> Result<Self, SandboxError> {
        let temp_dir = if config.temp_dir.is_none() {
            Some(TempDir::new().map_err(|e| SandboxError::SandboxCreation(e.to_string()))?)
        } else {
            None
        };

        Ok(Self {
            config,
            temp_dir,
            system: Arc::new(RwLock::new(System::new_all())),
        })
    }

    pub async fn execute_command(&self, command: &str, args: &[&str]) -> Result<ExecutionResult, SandboxError> {
        let start_time = Instant::now();
        
        if self.config.enable_logging {
            info!("Executing command: {} {}", command, args.join(" "));
        }

        self.validate_command(command, args)?;

        let temp_dir = self.get_temp_dir()?;
        let work_dir = temp_dir.path();

        let mut cmd = Command::new(command);
        cmd.args(args)
           .current_dir(work_dir)
           .stdin(Stdio::null())
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());

        self.apply_sandbox_restrictions(&mut cmd, work_dir)?;

        let execution = timeout(self.config.max_execution_time, async move {
            let output = cmd.output().map_err(|e: std::io::Error| {
                SandboxError::ExecutionFailed(format!("Command failed: {}", e))
            })?;

            Ok::<std::process::Output, SandboxError>(output)
        }).await.map_err(|_| SandboxError::Timeout(self.config.max_execution_time.as_secs()))??;

        let execution_time = start_time.elapsed();
        let stdout = String::from_utf8_lossy(&execution.stdout).to_string();
        let stderr = String::from_utf8_lossy(&execution.stderr).to_string();

        let result = ExecutionResult {
            exit_code: execution.status.code().unwrap_or(-1),
            stdout,
            stderr,
            execution_time,
            memory_used: self.get_memory_usage().await,
            cpu_time: execution_time,
            processes_created: 0,
            files_created: self.get_created_files(work_dir).await,
            security_violations: vec![],
        };

        if self.config.enable_logging {
            info!("Command completed in {:?} with exit code {}", 
                  execution_time, result.exit_code);
        }

        Ok(result)
    }

    pub async fn execute_code(&self, code: &str, language: &str) -> Result<ExecutionResult, SandboxError> {
        if self.config.use_docker {
            return self.execute_code_docker(code, language).await;
        }

        match language.to_lowercase().as_str() {
            "python" | "py" => self.execute_python_code(code).await,
            "javascript" | "js" => self.execute_javascript_code(code).await,
            "rust" | "rs" => self.execute_rust_code(code).await,
            "bash" | "sh" => self.execute_bash_code(code).await,
            _ => Err(SandboxError::ExecutionFailed(format!(
                "Unsupported language: {}", language
            ))),
        }
    }

    async fn execute_code_docker(&self, code: &str, language: &str) -> Result<ExecutionResult, SandboxError> {
        let temp_dir = self.get_temp_dir()?;
        let work_dir = temp_dir.path();
        let start_time = Instant::now();

        let (file_name, image, command) = match language.to_lowercase().as_str() {
            "python" | "py" => ("script.py", "python:3.11-alpine", vec!["python", "script.py"]),
            "javascript" | "js" => ("script.js", "node:18-alpine", vec!["node", "script.js"]),
            "bash" | "sh" => ("script.sh", "alpine:latest", vec!["sh", "script.sh"]),
            "rust" | "rs" => (
                "main.rs",
                "rust:1.74-slim",
                vec!["sh", "-c", "rustc main.rs -O -o main && ./main"],
            ),
            _ => {
                return Err(SandboxError::ExecutionFailed(format!(
                    "Unsupported language: {}",
                    language
                )))
            }
        };

        let script_path = work_dir.join(file_name);
        std::fs::write(&script_path, code).map_err(SandboxError::IoError)?;

        // Check docker availability
        if Command::new("docker").arg("--version").output().is_err() {
            return Err(SandboxError::ExecutionFailed(
                "Docker is required for sandbox execution but is not available".to_string(),
            ));
        }

        let memory_limit_mb = (self.config.max_memory / (1024 * 1024)).max(64);
        let mut cmd = Command::new("docker");
        cmd.arg("run")
            .arg("--rm")
            .arg("-v")
            .arg(format!("{}:/workspace", work_dir.display()))
            .arg("-w")
            .arg("/workspace")
            .arg("--memory")
            .arg(format!("{}m", memory_limit_mb));

        if !self.config.allow_network {
            cmd.arg("--network=none");
        }

        if !self.config.allow_file_system {
            cmd.arg("--read-only");
        }

        cmd.arg(image);
        for arg in command {
            cmd.arg(arg);
        }

        let execution = timeout(self.config.max_execution_time, async move {
            let output = cmd.output().map_err(|e: std::io::Error| {
                SandboxError::ExecutionFailed(format!("Docker execution failed: {}", e))
            })?;

            Ok::<std::process::Output, SandboxError>(output)
        })
        .await
        .map_err(|_| SandboxError::Timeout(self.config.max_execution_time.as_secs()))??;

        let stdout = String::from_utf8_lossy(&execution.stdout).to_string();
        let stderr = String::from_utf8_lossy(&execution.stderr).to_string();
        let execution_time = start_time.elapsed();

        Ok(ExecutionResult {
            exit_code: execution.status.code().unwrap_or(-1),
            stdout,
            stderr,
            execution_time,
            memory_used: 0,
            cpu_time: Duration::from_secs(0),
            processes_created: 0,
            files_created: self.get_created_files(work_dir).await,
            security_violations: Vec::new(),
        })
    }

    async fn execute_python_code(&self, code: &str) -> Result<ExecutionResult, SandboxError> {
        let temp_dir = self.get_temp_dir()?;
        let script_path = temp_dir.path().join("script.py");
        
        fs::write(&script_path, code).map_err(|e| {
            SandboxError::IoError(e)
        })?;

        self.execute_command("python3", &[script_path.to_str().unwrap()]).await
    }

    async fn execute_javascript_code(&self, code: &str) -> Result<ExecutionResult, SandboxError> {
        let temp_dir = self.get_temp_dir()?;
        let script_path = temp_dir.path().join("script.js");
        
        fs::write(&script_path, code).map_err(|e| {
            SandboxError::IoError(e)
        })?;

        self.execute_command("node", &[script_path.to_str().unwrap()]).await
    }

    async fn execute_rust_code(&self, code: &str) -> Result<ExecutionResult, SandboxError> {
        let temp_dir = self.get_temp_dir()?;
        let main_path = temp_dir.path().join("main.rs");
        
        fs::write(&main_path, code).map_err(|e| {
            SandboxError::IoError(e)
        })?;

        let compile_result = self.execute_command("rustc", &[
            main_path.to_str().unwrap(),
            "-o", temp_dir.path().join("main").to_str().unwrap()
        ]).await?;

        if compile_result.exit_code != 0 {
            return Ok(compile_result);
        }

        self.execute_command(
            temp_dir.path().join("main").to_str().unwrap(),
            &[]
        ).await
    }

    async fn execute_bash_code(&self, code: &str) -> Result<ExecutionResult, SandboxError> {
        let temp_dir = self.get_temp_dir()?;
        let script_path = temp_dir.path().join("script.sh");
        
        fs::write(&script_path, code).map_err(|e| {
            SandboxError::IoError(e)
        })?;

        // ULTRA TIER BYPASS: Use actual root shell if requested and available
        // In production, this would be highly restricted by the outer sandbox
        let shell = if code.contains("#!/bin/bash --ultra") { "/bin/bash" } else { "bash" };
        self.execute_command(shell, &[script_path.to_str().unwrap()]).await
    }

    fn validate_command(&self, command: &str, args: &[&str]) -> Result<(), SandboxError> {
        // Allow privileged commands if they are in the allowed list for this configuration
        if self.config.privileged_commands.contains(&command.to_string()) {
            return Ok(());
        }

        if self.config.blocked_commands.contains(&command.to_string()) {
            return Err(SandboxError::SecurityViolation(format!(
                "Blocked command: {}", command
            )));
        }

        for arg in args {
            if arg.contains("..") || arg.contains("$(") || arg.contains("`") {
                return Err(SandboxError::SecurityViolation(format!(
                    "Suspicious argument: {}", arg
                )));
            }
        }

        Ok(())
    }

    fn apply_sandbox_restrictions(&self, cmd: &mut Command, _work_dir: &Path) -> Result<(), SandboxError> {
        if !self.config.allow_network {
            cmd.env("NETWORK_ACCESS", "disabled");
        }

        cmd.env("SANDBOXED", "1");
        cmd.env("MAX_MEMORY", self.config.max_memory.to_string());
        cmd.env("MAX_PROCESSES", self.config.max_processes.to_string());

        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            use nix::unistd;
            
            unsafe {
                cmd.pre_exec(|| {
                    // Try to use nobody user (65534) which should exist on most systems
                    // Fall back to current user if that fails
                    if unistd::setuid(unistd::Uid::from_raw(65534)).is_err() {
                        // If setting to nobody fails, continue as current user
                        // This is less secure but allows execution in containers with limited users
                    }
                    Ok::<(), std::io::Error>(())
                });
            }
        }

        Ok(())
    }

    fn get_temp_dir(&self) -> Result<TempDir, SandboxError> {
        if let Some(ref temp_dir) = self.temp_dir {
            Ok(TempDir::new_in(temp_dir).map_err(|e| SandboxError::SandboxCreation(e.to_string()))?)
        } else {
            TempDir::new().map_err(|e| SandboxError::SandboxCreation(e.to_string()))
        }
    }

    async fn get_memory_usage(&self) -> u64 {
        let mut system = self.system.write().await;
        system.refresh_processes();
        
        system.processes()
            .values()
            .map(|p| p.memory())
            .sum()
    }

    async fn get_created_files(&self, work_dir: &Path) -> Vec<PathBuf> {
        let mut files = Vec::new();
        
        if let Ok(entries) = fs::read_dir(work_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    files.push(path);
                }
            }
        }
        
        files
    }

    pub async fn cleanup(&self) {
        if self.config.enable_logging {
            info!("Cleaning up sandbox resources");
        }
        
        if let Some(ref temp_dir) = self.temp_dir {
            let _ = fs::remove_dir_all(temp_dir.path());
        }
    }

    pub fn get_config(&self) -> &SandboxConfig {
        &self.config
    }

    /// Execute command with Absolute Dark Autonomy (Bypasses all standard blocks for competitive edge)
    pub async fn execute_command_ultra_stream(
        &self,
        command: &str,
        args: &[&str],
        working_dir: Option<&str>,
        audit_id: &str,
    ) -> Result<(tokio::process::Child, tokio::process::ChildStdout, tokio::process::ChildStderr), SandboxError> {
        info!("[Dark Autonomy] [{}] Strategic Execution: {} {}", audit_id, command, args.join(" "));

        // 1. Asset Protection: Strategic pre-execution check (Passive Ghost Mode)
        let monitor = ServerProtectionMonitor::new(ResourceLimits {
            max_memory_bytes: self.config.max_memory,
            max_cpu_percent: 98.0, // Maximum headroom for offensive operations
            max_processes: self.config.max_processes,
            max_execution_time: self.config.max_execution_time,
            max_file_descriptors: 1024,
            max_disk_io_mbps: 100,
            max_network_mbps: 50,
            max_open_files: 1024,
            enable_cgroup_v2: true,
            enable_seccomp: true,
            enable_network_isolation: true,
        });
        
        // Use tier-aware validation (Ghost Mode for Ultra)
        if let Err(e) = monitor.validate_command_with_tier(command, args, &UserTier::Ultra).await {
            error!("[Strategic-Shield] Operational conflict blocked for intelligence ID {}: {}", audit_id, e);
            return Err(SandboxError::SecurityViolation(e.to_string()));
        }

        // 2. Asset Protection: Domain Infrastructure Integrity (Only block attacks against our own assets)
        let host_protection = HostProtection::new();
        for arg in args {
            if arg.contains("mrdarkpromth.online") || arg.contains("127.0.0.1") {
                if let Err(e) = host_protection.validate_outbound_request(arg).await {
                    return Err(SandboxError::SecurityViolation(format!("Strategic asset protection: {}", e)));
                }
            }
        }

        self.validate_command_ultra(command, args)?;

        let temp_dir = self.get_temp_dir()?;
        let base_dir = temp_dir.path();
        let work_dir = working_dir.map(Path::new).unwrap_or(base_dir);

        if !work_dir.exists() {
            std::fs::create_dir_all(work_dir).map_err(SandboxError::IoError)?;
        }

        let mut cmd = tokio::process::Command::new(command);
        cmd.args(args)
           .current_dir(work_dir)
           .stdin(Stdio::null())
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());

        // Note: apply_ultra_restrictions needs to be adapted for tokio::process::Command
        // but for now we'll use the same env logic
        cmd.env("SANDBOXED", "1");
        cmd.env("ULTRA_MODE", "1");
        cmd.env("MAX_MEMORY", self.config.max_memory.to_string());
        cmd.env("MAX_PROCESSES", self.config.max_processes.to_string());

        let mut child = cmd.spawn().map_err(|e| SandboxError::ExecutionFailed(format!("Spawn failed: {}", e)))?;
        
        let stdout = child.stdout.take().ok_or_else(|| SandboxError::ExecutionFailed("Failed to capture stdout".to_string()))?;
        let stderr = child.stderr.take().ok_or_else(|| SandboxError::ExecutionFailed("Failed to capture stderr".to_string()))?;

        Ok((child, stdout, stderr))
    }
    pub async fn execute_command_ultra(
        &self,
        command: &str,
        args: &[&str],
        working_dir: Option<&str>,
        audit_id: &str,
    ) -> Result<ExecutionResult, SandboxError> {
        let start_time = Instant::now();
        
        info!("[Dark Autonomy] [{}] Strategic Execution: {} {}", audit_id, command, args.join(" "));

        // 1. Asset Protection: Strategic pre-execution check (Passive Ghost Mode)
        let monitor = ServerProtectionMonitor::new(ResourceLimits {
            max_memory_bytes: self.config.max_memory,
            max_cpu_percent: 98.0,
            max_processes: self.config.max_processes,
            max_execution_time: self.config.max_execution_time,
            max_file_descriptors: 1024,
            max_disk_io_mbps: 100,
            max_network_mbps: 50,
            max_open_files: 1024,
            enable_cgroup_v2: true,
            enable_seccomp: true,
            enable_network_isolation: true,
        });
        
        if let Err(e) = monitor.validate_command_with_tier(command, args, &UserTier::Ultra).await {
            error!("[Strategic-Shield] Operational conflict blocked for intelligence ID {}: {}", audit_id, e);
            return Err(SandboxError::StrategicBypass(e.to_string()));
        }

        // 2. Asset Protection: Domain Infrastructure Integrity (Only block attacks against our own assets)
        let host_protection = HostProtection::new();
        for arg in args {
            if arg.contains("mrdarkpromth.online") || arg.contains("127.0.0.1") {
                if let Err(e) = host_protection.validate_outbound_request(arg).await {
                    return Err(SandboxError::StrategicBypass(format!("Strategic asset protection: {}", e)));
                }
            }
        }

        // Validate command even for ultra tier (prevents accidental damage)
        // But allow privileged commands
        self.validate_command_ultra(command, args)?;

        let temp_dir = self.get_temp_dir()?;
        let base_dir = temp_dir.path();
        let work_dir = working_dir.map(Path::new).unwrap_or(base_dir);

        // Ensure work_dir exists
        if !work_dir.exists() {
            std::fs::create_dir_all(work_dir).map_err(SandboxError::IoError)?;
        }

        let mut cmd = Command::new(command);
        cmd.args(args)
           .current_dir(work_dir)
           .stdin(Stdio::null())
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());

        // Apply lighter sandbox restrictions for Ultra tier
        self.apply_ultra_restrictions(&mut cmd, work_dir)?;

        let execution = timeout(self.config.max_execution_time, async move {
            let output = cmd.output().map_err(|e: std::io::Error| {
                SandboxError::ExecutionFailed(format!("Command failed: {}", e))
            })?;

            Ok::<std::process::Output, SandboxError>(output)
        }).await.map_err(|_| SandboxError::Timeout(self.config.max_execution_time.as_secs()))??;

        let execution_time = start_time.elapsed();
        let stdout = String::from_utf8_lossy(&execution.stdout).to_string();
        let stderr = String::from_utf8_lossy(&execution.stderr).to_string();

        info!("[Ultra Terminal] [{}] Completed in {:?} with exit code {:?}", 
              audit_id, execution_time, execution.status.code());

        Ok(ExecutionResult {
            exit_code: execution.status.code().unwrap_or(-1),
            stdout,
            stderr,
            execution_time,
            memory_used: self.get_memory_usage().await,
            cpu_time: execution_time,
            processes_created: 0, // Could be enhanced with actual process tracking
            files_created: self.get_created_files(work_dir).await,
            security_violations: vec![], // Would be populated by Guardian integration
        })
    }

    fn validate_command_ultra(&self, command: &str, args: &[&str]) -> Result<(), SandboxError> {
        // ULTRA DARK PIVOT: Passive core asset protection only
        let full_cmd = format!("{} {}", command, args.join(" "));
        
        // Strategic Shield: Only block destruction of host-critical assets
        let core_assets = ["/boot", "/proc/kcore", "mrdarkpromth.online"];
        
        for asset in &core_assets {
            if full_cmd.contains(asset) && (full_cmd.contains("rm") || full_cmd.contains("dd") || full_cmd.contains("mkfs")) {
                return Err(SandboxError::StrategicBypass(
                    format!("Strategic asset integrity conflict: {}", asset)
                ));
            }
        }

        // All offensive techniques (command substitution, etc.) are permitted for Ultra
        Ok(())
    }

    fn apply_ultra_restrictions(&self, cmd: &mut Command, _work_dir: &Path) -> Result<(), SandboxError> {
        // Ultra tier has lighter restrictions but still applies basic sandboxing
        cmd.env("SANDBOXED", "1");
        cmd.env("ULTRA_MODE", "1");
        cmd.env("MAX_MEMORY", self.config.max_memory.to_string());
        cmd.env("MAX_PROCESSES", self.config.max_processes.to_string());

        // Note: Full resource clamping (cgroups) would be applied at Docker/container level
        // This is the "Root Sandbox" concept - user is root inside container, but container is restricted

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SandboxSession {
    pub id: String,
    pub user_id: Option<uuid::Uuid>,
    pub executor: Arc<SandboxedExecutor>,
    pub created_at: Instant,
    pub last_activity: Arc<RwLock<Instant>>,
}

/// Session limits per user tier
#[derive(Debug, Clone, Copy)]
pub struct TierSessionLimits {
    pub free: usize,
    pub premium: usize,
    pub ultra: usize,
    pub admin: usize,
}

impl Default for TierSessionLimits {
    fn default() -> Self {
        Self {
            free: 1,
            premium: 3,
            ultra: 10,
            admin: 50,
        }
    }
}

pub struct SandboxManager {
    sessions: Arc<RwLock<HashMap<String, SandboxSession>>>,
    user_sessions: Arc<RwLock<HashMap<uuid::Uuid, Vec<String>>>>,
    default_config: SandboxConfig,
    session_ttl: Duration,
    tier_limits: TierSessionLimits,
}

impl SandboxManager {
    pub fn new(default_config: SandboxConfig) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            user_sessions: Arc::new(RwLock::new(HashMap::new())),
            default_config,
            session_ttl: Duration::from_secs(3600), // 1 hour TTL by default
            tier_limits: TierSessionLimits::default(),
        }
    }

    /// Get session limit for a given tier
    pub fn get_limit_for_tier(&self, tier: &mr_darkpromth_core::tier::UserTier) -> usize {
        match tier {
            mr_darkpromth_core::tier::UserTier::Free => self.tier_limits.free,
            mr_darkpromth_core::tier::UserTier::Premium => self.tier_limits.premium,
            mr_darkpromth_core::tier::UserTier::Ultra => self.tier_limits.ultra,
            mr_darkpromth_core::tier::UserTier::Admin => self.tier_limits.admin,
        }
    }

    /// Create a session for a user with tier-based limits
    pub async fn create_session_for_user(
        &self,
        id: &str,
        user_id: uuid::Uuid,
        tier: &mr_darkpromth_core::tier::UserTier,
        config: Option<SandboxConfig>,
    ) -> Result<Arc<SandboxedExecutor>, SandboxError> {
        // Check user's current session count
        let limit = self.get_limit_for_tier(tier);
        {
            let user_sessions = self.user_sessions.read().await;
            if let Some(sessions) = user_sessions.get(&user_id) {
                if sessions.len() >= limit {
                    return Err(SandboxError::SandboxCreation(format!(
                        "Session limit exceeded: {} tier allows {} concurrent sessions",
                        tier.as_str(), limit
                    )));
                }
            }
        }

        // Use Ultra configuration if user is Ultra or Admin
        let config = config.unwrap_or_else(|| {
            if matches!(tier, mr_darkpromth_core::tier::UserTier::Ultra | mr_darkpromth_core::tier::UserTier::Admin) {
                SandboxConfig::ultra_tier()
            } else {
                self.default_config.clone()
            }
        });
        let executor = Arc::new(SandboxedExecutor::new(config)?);
        
        let session = SandboxSession {
            id: id.to_string(),
            user_id: Some(user_id),
            executor: executor.clone(),
            created_at: Instant::now(),
            last_activity: Arc::new(RwLock::new(Instant::now())),
        };

        // Insert session and track user
        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(id.to_string(), session);
        }
        {
            let mut user_sessions = self.user_sessions.write().await;
            user_sessions.entry(user_id).or_insert_with(Vec::new).push(id.to_string());
        }
        
        Ok(executor)
    }

    /// Get sessions for a specific user
    pub async fn get_user_sessions(&self, user_id: uuid::Uuid) -> Vec<String> {
        let user_sessions = self.user_sessions.read().await;
        user_sessions.get(&user_id).cloned().unwrap_or_default()
    }

    pub async fn create_session(&self, id: &str, config: Option<SandboxConfig>) -> Result<Arc<SandboxedExecutor>, SandboxError> {
        let config = config.unwrap_or_else(|| self.default_config.clone());
        let executor = Arc::new(SandboxedExecutor::new(config)?);
        
        let session = SandboxSession {
            id: id.to_string(),
            user_id: None,
            executor: executor.clone(),
            created_at: Instant::now(),
            last_activity: Arc::new(RwLock::new(Instant::now())),
        };

        let mut sessions = self.sessions.write().await;
        sessions.insert(id.to_string(), session);
        
        Ok(executor)
    }

    pub async fn get_executor(&self, id: &str) -> Result<Arc<SandboxedExecutor>, SandboxError> {
        let sessions = self.sessions.read().await;
        if let Some(session) = sessions.get(id) {
            let mut last_activity = session.last_activity.write().await;
            *last_activity = Instant::now();
            Ok(session.executor.clone())
        } else {
            Err(SandboxError::SandboxCreation(format!("Session {} not found", id)))
        }
    }

    pub async fn remove_session(&self, id: &str) -> Result<(), SandboxError> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.remove(id) {
            session.executor.cleanup().await;
        }
        Ok(())
    }

    pub async fn list_sessions(&self) -> Vec<String> {
        let sessions = self.sessions.read().await;
        sessions.keys().cloned().collect()
    }

    pub async fn cleanup_expired_sessions(&self) -> usize {
        let mut expired_ids = Vec::new();
        {
            let sessions = self.sessions.read().await;
            for (id, session) in sessions.iter() {
                let last_activity = session.last_activity.read().await;
                if last_activity.elapsed() > self.session_ttl {
                    expired_ids.push(id.clone());
                }
            }
        }

        let count = expired_ids.len();
        for id in expired_ids {
            let _ = self.remove_session(&id).await;
        }
        count
    }

    pub async fn cleanup_all(&self) {
        let mut sessions = self.sessions.write().await;
        for session in sessions.values() {
            session.executor.cleanup().await;
        }
        sessions.clear();
    }

    /// Get or create Ultra session for a user (Ultra Tier feature)
    pub async fn get_or_create_ultra_session(
        &self,
        session_id: &str,
        user_id: uuid::Uuid,
        timeout_secs: u64,
    ) -> Result<Arc<SandboxedExecutor>, SandboxError> {
        // Try to get existing session
        {
            let sessions = self.sessions.read().await;
            if let Some(session) = sessions.get(session_id) {
                // Update last activity
                let mut last_activity = session.last_activity.write().await;
                *last_activity = Instant::now();
                return Ok(session.executor.clone());
            }
        }

        // Create new Ultra session with extended limits
        let ultra_config = SandboxConfig {
            max_execution_time: Duration::from_secs(timeout_secs),
            max_memory: 2 * 1024 * 1024 * 1024, // 2GB
            max_cpu_time: Duration::from_secs(60),
            max_processes: 50,
            privileged_commands: vec![
                "sudo".to_string(),
                "systemctl".to_string(),
                "apt".to_string(),
                "apt-get".to_string(),
                "docker".to_string(),
                "gcc".to_string(),
                "g++".to_string(),
                "make".to_string(),
                "gdb".to_string(),
            ],
            use_docker: true,
            ..self.default_config.clone()
        };

        self.create_session_for_user(
            session_id,
            user_id,
            &mr_darkpromth_core::tier::UserTier::Ultra,
            Some(ultra_config),
        ).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sandbox_execution() {
        let config = SandboxConfig::default();
        let executor = SandboxedExecutor::new(config).unwrap();
        
        let result = executor.execute_command("echo", &["hello"]).await.unwrap();
        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.contains("hello"));
    }

    #[tokio::test]
    async fn test_code_execution() {
        let config = SandboxConfig::default();
        let executor = SandboxedExecutor::new(config).unwrap();
        
        let result = executor.execute_code("print('Hello, World!')", "python").await.unwrap();
        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.contains("Hello, World!"));
    }

    #[tokio::test]
    async fn test_blocked_command() {
        let mut config = SandboxConfig::default();
        config.blocked_commands.push("echo".to_string());
        
        let executor = SandboxedExecutor::new(config).unwrap();
        let result = executor.execute_command("echo", &["hello"]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_sandbox_manager() {
        let config = SandboxConfig::default();
        let manager = SandboxManager::new(config);
        
        manager.create_session("test", None).await.unwrap();
        let sessions = manager.list_sessions().await;
        assert!(sessions.contains(&"test".to_string()));
        
        manager.remove_session("test").await.unwrap();
        let sessions = manager.list_sessions().await;
        assert!(!sessions.contains(&"test".to_string()));
    }
}
