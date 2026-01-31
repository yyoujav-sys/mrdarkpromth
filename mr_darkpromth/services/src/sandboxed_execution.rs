use log::info;
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

#[derive(Error, Debug)]
pub enum SandboxError {
    #[error("Failed to create sandbox: {0}")]
    SandboxCreation(String),
    #[error("Execution timeout: {0}s")]
    Timeout(u64),
    #[error("Resource limit exceeded: {0}")]
    ResourceLimit(String),
    #[error("Command execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Security violation: {0}")]
    SecurityViolation(String),
    #[error("IO error: {0}")]
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
    pub temp_dir: Option<PathBuf>,
    pub enable_logging: bool,
    pub use_docker: bool,
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
        std::fs::write(&script_path, code).map_err(|e| SandboxError::IoError(e))?;

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
            &temp_dir.path().join("main").to_str().unwrap(),
            &[]
        ).await
    }

    async fn execute_bash_code(&self, code: &str) -> Result<ExecutionResult, SandboxError> {
        let temp_dir = self.get_temp_dir()?;
        let script_path = temp_dir.path().join("script.sh");
        
        fs::write(&script_path, code).map_err(|e| {
            SandboxError::IoError(e)
        })?;

        self.execute_command("bash", &[script_path.to_str().unwrap()]).await
    }

    fn validate_command(&self, command: &str, args: &[&str]) -> Result<(), SandboxError> {
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
        cmd.env("MAX_MEMORY", &self.config.max_memory.to_string());
        cmd.env("MAX_PROCESSES", &self.config.max_processes.to_string());

        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            use nix::unistd;
            
            unsafe {
                cmd.pre_exec(|| {
                    unistd::setuid(unistd::Uid::from_raw(1000)).map_err(|e| {
                        std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
                    })?;
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
}

pub struct SandboxManager {
    executors: Arc<RwLock<HashMap<String, Arc<SandboxedExecutor>>>>,
    default_config: SandboxConfig,
}

impl SandboxManager {
    pub fn new(default_config: SandboxConfig) -> Self {
        Self {
            executors: Arc::new(RwLock::new(HashMap::new())),
            default_config,
        }
    }

    pub async fn create_executor(&self, id: &str, config: Option<SandboxConfig>) -> Result<(), SandboxError> {
        let config = config.unwrap_or_else(|| self.default_config.clone());
        let executor = Arc::new(SandboxedExecutor::new(config)?);
        
        let mut executors = self.executors.write().await;
        executors.insert(id.to_string(), executor);
        
        Ok(())
    }

    pub async fn get_executor(&self, id: &str) -> Result<Arc<SandboxedExecutor>, SandboxError> {
        let executors = self.executors.read().await;
        executors.get(id)
            .cloned()
            .ok_or_else(|| SandboxError::SandboxCreation(format!("Executor {} not found", id)))
    }

    pub async fn remove_executor(&self, id: &str) -> Result<(), SandboxError> {
        let mut executors = self.executors.write().await;
        if let Some(executor) = executors.remove(id) {
            executor.cleanup().await;
        }
        Ok(())
    }

    pub async fn list_executors(&self) -> Vec<String> {
        let executors = self.executors.read().await;
        executors.keys().cloned().collect()
    }

    pub async fn cleanup_all(&self) {
        let executors = self.executors.read().await;
        for executor in executors.values() {
            executor.cleanup().await;
        }
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
        
        manager.create_executor("test", None).await.unwrap();
        let executors = manager.list_executors().await;
        assert!(executors.contains(&"test".to_string()));
        
        manager.remove_executor("test").await.unwrap();
        let executors = manager.list_executors().await;
        assert!(!executors.contains(&"test".to_string()));
    }
}
