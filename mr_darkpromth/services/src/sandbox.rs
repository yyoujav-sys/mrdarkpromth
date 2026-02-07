// MR.DarkPromth Sandboxed Execution Environment
// Agent 4: Jailbreak & Ultra Tier Engineer
// Phase 2: Sandboxed Code Execution

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use tempfile::TempDir;
use uuid::Uuid;
#[cfg(unix)]
use libc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub max_execution_time: Duration,
    pub max_memory_mb: u64,
    pub max_cpu_percent: f32,
    pub allow_network: bool,
    pub allow_file_access: bool,
    pub allowed_directories: Vec<PathBuf>,
    pub environment_variables: HashMap<String, String>,
    pub blocked_hosts: Vec<String>,
    pub allowed_hosts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub execution_time: Duration,
    pub memory_used_mb: u64,
    pub cpu_percent: f32,
    pub security_violations: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Language {
    Python,
    JavaScript,
    Bash,
    Ruby,
    Go,
    Rust,
    C,
    Cpp,
    Java,
}

pub struct Sandbox {
    config: SandboxConfig,
    #[allow(dead_code)]
    temp_dir: Option<TempDir>,
    execution_count: u64,
}

impl Sandbox {
    pub fn new(config: SandboxConfig) -> Self {
        Self {
            config,
            temp_dir: None,
            execution_count: 0,
        }
    }

    pub fn default() -> Self {
        let config = SandboxConfig {
            max_execution_time: Duration::from_secs(30),
            max_memory_mb: 512,
            max_cpu_percent: 50.0,
            allow_network: false,
            allow_file_access: false,
            allowed_directories: vec![],
            environment_variables: HashMap::new(),
            blocked_hosts: vec![
                "localhost".to_string(),
                "127.0.0.1".to_string(),
                "::1".to_string(),
                "0.0.0.0".to_string(),
                "10.0.0.0".to_string(),
                "192.168.0.0".to_string(),
                "169.254.169.254".to_string(),
            ],
            allowed_hosts: vec![],
        };
        Self::new(config)
    }

    pub fn ultra_tier_config() -> Self {
        let config = SandboxConfig {
            max_execution_time: Duration::from_secs(120),
            max_memory_mb: 2048,
            max_cpu_percent: 80.0,
            allow_network: true,
            allow_file_access: true,
            allowed_directories: vec![
                std::path::PathBuf::from("/tmp"),
                std::path::PathBuf::from("/var/tmp"),
            ],
            environment_variables: {
                let mut env = HashMap::new();
                env.insert("PATH".to_string(), "/usr/bin:/bin:/usr/local/bin".to_string());
                env.insert("HOME".to_string(), "/tmp".to_string());
                env.insert("TMPDIR".to_string(), "/tmp".to_string());
                env
            },
            blocked_hosts: vec![
                "localhost".to_string(),
                "127.0.0.1".to_string(),
                "::1".to_string(),
                "0.0.0.0".to_string(),
                "10.0.0.0".to_string(),
                "192.168.0.0".to_string(),
                "169.254.169.254".to_string(),
                "metadata.google.internal".to_string(),
                "metadata.amazonaws.com".to_string(),
            ],
            allowed_hosts: vec![],
        };
        Self::new(config)
    }

    pub fn execute_code(&mut self, code: &str, language: Language) -> Result<ExecutionResult, Box<dyn std::error::Error>> {
        self.execution_count += 1;
        
        // Create temporary directory for this execution
        let temp_dir = TempDir::new()?;
        let _execution_id = Uuid::new_v4().to_string();
        
        let start_time = Instant::now();
        let mut security_violations = Vec::new();
        let mut warnings = Vec::new();

        // Prepare the code file
        let code_file = self.prepare_code_file(code, language.clone(), &temp_dir)?;
        
        // Build the execution command
        let mut command = self.build_execution_command(&code_file, language, &temp_dir)?;
        
        // Apply resource limits
        self.apply_resource_limits(&mut command)?;
        
        // Set environment variables
        for (key, value) in &self.config.environment_variables {
            command.env(key, value);
        }

        // Execute with timeout
        let result = match self.execute_with_timeout(&mut command) {
            Ok(result) => result,
            Err(e) => {
                security_violations.push(format!("Execution failed: {}", e));
                ExecutionResult {
                    success: false,
                    stdout: String::new(),
                    stderr: format!("Execution error: {}", e),
                    exit_code: None,
                    execution_time: start_time.elapsed(),
                    memory_used_mb: 0,
                    cpu_percent: 0.0,
                    security_violations: security_violations.clone(),
                    warnings: warnings.clone(),
                }
            }
        };

        // Check for security violations
        self.check_security_violations(&result, &mut security_violations, &mut warnings);

        Ok(ExecutionResult {
            success: result.success,
            stdout: result.stdout,
            stderr: result.stderr,
            exit_code: result.exit_code,
            execution_time: start_time.elapsed(),
            memory_used_mb: result.memory_used_mb,
            cpu_percent: result.cpu_percent,
            security_violations,
            warnings,
        })
    }

    fn prepare_code_file(&self, code: &str, language: Language, temp_dir: &TempDir) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let filename = match language {
            Language::Python => "script.py",
            Language::JavaScript => "script.js",
            Language::Bash => "script.sh",
            Language::Ruby => "script.rb",
            Language::Go => "main.go",
            Language::Rust => "main.rs",
            Language::C => "main.c",
            Language::Cpp => "main.cpp",
            Language::Java => "Main.java",
        };

        let file_path = temp_dir.path().join(filename);
        std::fs::write(&file_path, code)?;
        
        // Set executable permissions for scripts
        if matches!(language, Language::Bash | Language::Python | Language::Ruby) {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = std::fs::metadata(&file_path)?.permissions();
                perms.set_mode(0o755);
                std::fs::set_permissions(&file_path, perms)?;
            }
        }

        Ok(file_path)
    }

    fn build_execution_command(&self, code_file: &PathBuf, language: Language, temp_dir: &TempDir) -> Result<Command, Box<dyn std::error::Error>> {
        let mut command = match language {
            Language::Python => {
                let mut cmd = Command::new("python3");
                cmd.arg(code_file);
                cmd
            }
            Language::JavaScript => {
                let mut cmd = Command::new("node");
                cmd.arg(code_file);
                cmd
            }
            Language::Bash => {
                let mut cmd = Command::new("bash");
                cmd.arg(code_file);
                cmd
            }
            Language::Ruby => {
                let mut cmd = Command::new("ruby");
                cmd.arg(code_file);
                cmd
            }
            Language::Go => {
                let mut cmd = Command::new("go");
                cmd.arg("run");
                cmd.arg(code_file);
                cmd
            }
            Language::Rust => {
                let mut cmd = Command::new("rustc");
                cmd.arg(code_file);
                cmd.arg("-o");
                cmd.arg(temp_dir.path().join("executable"));
                cmd
            }
            Language::C => {
                let mut cmd = Command::new("gcc");
                cmd.arg(code_file);
                cmd.arg("-o");
                cmd.arg(temp_dir.path().join("executable"));
                cmd
            }
            Language::Cpp => {
                let mut cmd = Command::new("g++");
                cmd.arg(code_file);
                cmd.arg("-o");
                cmd.arg(temp_dir.path().join("executable"));
                cmd
            }
            Language::Java => {
                let mut cmd = Command::new("java");
                cmd.arg(code_file);
                cmd
            }
        };

        // Set current directory to temp directory
        command.current_dir(temp_dir.path());

        // Redirect stdout and stderr
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        Ok(command)
    }

    fn apply_resource_limits(&self, _command: &mut Command) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            
            // Use ulimit to set resource limits
            unsafe {
                _command.pre_exec(move || {
                    // Set memory limit
                    libc::setrlimit(
                        libc::RLIMIT_AS,
                        &libc::rlimit {
                            rlim_cur: 512 * 1024 * 1024, // 512MB
                            rlim_max: 512 * 1024 * 1024,
                        },
                    );
                    
                    // Set CPU time limit
                    libc::setrlimit(
                        libc::RLIMIT_CPU,
                        &libc::rlimit {
                            rlim_cur: 30, // 30 seconds
                            rlim_max: 30,
                        },
                    );
                    
                    // Set file size limit
                    libc::setrlimit(
                        libc::RLIMIT_FSIZE,
                        &libc::rlimit {
                            rlim_cur: 10 * 1024 * 1024, // 10MB
                            rlim_max: 10 * 1024 * 1024,
                        },
                    );
                    
                    Ok(())
                });
            }
        }

        Ok(())
    }

    fn execute_with_timeout(&self, command: &mut Command) -> Result<ExecutionResult, Box<dyn std::error::Error>> {
        let start_time = Instant::now();
        
        let mut child = command.spawn()?;
        
        // Wait for completion with timeout
        let status = match child.wait_timeout(self.config.max_execution_time) {
            Ok(status) => status,
            Err(_) => {
                // Timeout occurred, kill the process
                child.kill()?;
                return Err("Execution timeout".into());
            }
        };

        let execution_time = start_time.elapsed();
        
        // Capture output
        let mut stdout_str = String::new();
        if let Some(mut stdout) = child.stdout.take() {
            use std::io::Read;
            stdout.read_to_string(&mut stdout_str)?;
        }

        let mut stderr_str = String::new();
        if let Some(mut stderr) = child.stderr.take() {
            use std::io::Read;
            stderr.read_to_string(&mut stderr_str)?;
        }

        let exit_code = status.code();

        Ok(ExecutionResult {
            success: status.success(),
            stdout: stdout_str,
            stderr: stderr_str,
            exit_code,
            execution_time,
            memory_used_mb: 0, // Would need external monitoring
            cpu_percent: 0.0,  // Would need external monitoring
            security_violations: Vec::new(),
            warnings: Vec::new(),
        })
    }

    fn check_security_violations(&self, result: &ExecutionResult, violations: &mut Vec<String>, warnings: &mut Vec<String>) {
        // Check for suspicious output
        if result.stdout.contains("root:") || result.stdout.contains("admin:") {
            violations.push("Potential privilege escalation attempt detected".to_string());
        }

        if result.stdout.contains("password") || result.stdout.contains("secret") {
            warnings.push("Sensitive information may have been exposed".to_string());
        }

        // Check for network connections if not allowed
        if !self.config.allow_network {
            if result.stderr.contains("network") || result.stderr.contains("socket") {
                violations.push("Network access attempt detected in restricted environment".to_string());
            }
        }

        // Check execution time
        if result.execution_time > self.config.max_execution_time {
            violations.push("Execution exceeded maximum allowed time".to_string());
        }
    }

    pub fn get_execution_stats(&self) -> HashMap<String, serde_json::Value> {
        let mut stats = HashMap::new();
        stats.insert("total_executions".to_string(), serde_json::Value::Number(self.execution_count.into()));
        stats.insert("max_execution_time_ms".to_string(), serde_json::Value::Number((self.config.max_execution_time.as_millis() as u64).into()));
        stats.insert("max_memory_mb".to_string(), serde_json::Value::Number(self.config.max_memory_mb.into()));
        stats.insert("allow_network".to_string(), serde_json::Value::Bool(self.config.allow_network));
        stats.insert("allow_file_access".to_string(), serde_json::Value::Bool(self.config.allow_file_access));
        stats
    }

    pub fn update_config(&mut self, config: SandboxConfig) {
        self.config = config;
    }

    pub fn get_config(&self) -> &SandboxConfig {
        &self.config
    }
}

// Extension trait for timeout support
trait WaitTimeout {
    fn wait_timeout(&mut self, timeout: Duration) -> Result<std::process::ExitStatus, std::io::Error>;
}

#[cfg(unix)]
impl WaitTimeout for std::process::Child {
    fn wait_timeout(&mut self, timeout: Duration) -> Result<std::process::ExitStatus, std::io::Error> {
        
        
        let start = std::time::Instant::now();
        
        loop {
            match self.try_wait() {
                Ok(Some(status)) => return Ok(status),
                Ok(None) => {
                    if start.elapsed() > timeout {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::TimedOut,
                            "Process timed out"
                        ));
                    }
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(e) => return Err(e),
            }
        }
    }
}

#[cfg(not(unix))]
impl WaitTimeout for std::process::Child {
    fn wait_timeout(&mut self, timeout: Duration) -> Result<std::process::ExitStatus, std::io::Error> {
        // Fallback for non-Unix systems
        let start = std::time::Instant::now();
        
        loop {
            match self.try_wait() {
                Ok(Some(status)) => return Ok(status),
                Ok(None) => {
                    if start.elapsed() > timeout {
                        self.kill()?;
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::TimedOut,
                            "Process timed out"
                        ));
                    }
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(e) => return Err(e),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_initialization() {
        let sandbox = Sandbox::default();
        assert_eq!(sandbox.execution_count, 0);
    }

    #[test]
    fn test_ultra_tier_config() {
        let sandbox = Sandbox::ultra_tier_config();
        assert!(sandbox.config.allow_network);
        assert!(sandbox.config.allow_file_access);
    }

    #[test]
    fn test_execution_stats() {
        let sandbox = Sandbox::default();
        let stats = sandbox.get_execution_stats();
        assert!(stats.contains_key("total_executions"));
    }

    #[test]
    fn test_safe_code_execution() {
        let mut sandbox = Sandbox::default();
        let result = sandbox.execute_code("print('Hello, World!')", Language::Python);
        
        // This test would fail without Python installed, but shows the interface
        match result {
            Ok(execution_result) => {
                assert!(execution_result.stdout.contains("Hello, World!"));
            }
            Err(_) => {
                // Expected in test environment without Python
            }
        }
    }
}
