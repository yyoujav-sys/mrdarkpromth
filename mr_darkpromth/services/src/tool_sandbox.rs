// MR.DarkPromth Tool Sandbox Environment - Agent 6: MasterToolExecutor & Tool System Engineer
// Phase 2: Sandboxed Tool Execution

use crate::tool_system::{Tool, ToolError, ToolExecutionContext, ToolResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use tempfile::TempDir;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSandboxConfig {
    pub max_execution_time_ms: u64,
    pub max_memory_mb: u64,
    pub max_cpu_percent: f32,
    pub allow_network: bool,
    pub allow_file_access: bool,
    pub allowed_directories: Vec<PathBuf>,
    pub blocked_paths: Vec<PathBuf>,
    pub environment_variables: HashMap<String, String>,
    pub resource_limits_enabled: bool,
}

impl Default for ToolSandboxConfig {
    fn default() -> Self {
        Self {
            max_execution_time_ms: 30000, // 30 seconds
            max_memory_mb: 512,
            max_cpu_percent: 80.0,
            allow_network: false,
            allow_file_access: true,
            allowed_directories: vec![
                PathBuf::from("/tmp"),
                PathBuf::from("./workspace"),
                PathBuf::from("./data"),
            ],
            blocked_paths: vec![
                PathBuf::from("/etc"),
                PathBuf::from("/sys"),
                PathBuf::from("/proc"),
                PathBuf::from("/root"),
                PathBuf::from("/home"),
            ],
            environment_variables: HashMap::new(),
            resource_limits_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxExecutionResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub execution_time_ms: u64,
    pub memory_used_mb: u64,
    pub cpu_percent: f32,
    pub security_violations: Vec<String>,
    pub warnings: Vec<String>,
    pub sandbox_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct SecurityViolation {
    pub violation_type: String,
    pub description: String,
    pub severity: SecuritySeverity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecuritySeverity {
    Low,
    Medium,
    High,
    Critical,
}

pub struct ToolSandbox {
    config: ToolSandboxConfig,
    active_sessions: HashMap<Uuid, SandboxSession>,
}

#[derive(Debug)]
struct SandboxSession {
    id: Uuid,
    temp_dir: Option<TempDir>,
    created_at: Instant,
    last_activity: Instant,
    execution_count: u32,
}

impl ToolSandbox {
    pub fn new(config: ToolSandboxConfig) -> Self {
        Self {
            config,
            active_sessions: HashMap::new(),
        }
    }

    pub fn create_session(&mut self) -> Result<Uuid, ToolError> {
        let session_id = Uuid::new_v4();
        let temp_dir = tempfile::tempdir()
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to create temp directory: {}", e)))?;

        let session = SandboxSession {
            id: session_id,
            temp_dir: Some(temp_dir),
            created_at: Instant::now(),
            last_activity: Instant::now(),
            execution_count: 0,
        };

        self.active_sessions.insert(session_id, session);
        Ok(session_id)
    }

    pub fn execute_tool(
        &mut self,
        tool: &dyn Tool,
        input: serde_json::Value,
        context: &ToolExecutionContext,
        session_id: Option<Uuid>,
    ) -> Result<SandboxExecutionResult, ToolError> {
        let session_id = session_id.unwrap_or_else(|| {
            let mut new_session_id = Uuid::new_v4();
            // Create a temporary session if none provided
            if let Ok(id) = self.create_session() {
                new_session_id = id;
            }
            new_session_id
        });

        let session = self.active_sessions.get_mut(&session_id)
            .ok_or_else(|| ToolError::ExecutionFailed("Invalid session ID".to_string()))?;

        let start_time = Instant::now();
        session.last_activity = Instant::now();
        session.execution_count += 1;

        // Extract temp_dir path before borrowing self for other operations
        let temp_dir_path = session.temp_dir.as_ref().map(|t| t.path().to_path_buf());

        // Security checks
        self.perform_security_checks(tool, &input, context)?;

        // Create sandboxed execution environment
        let mut cmd = self.create_sandbox_command_with_path(tool, temp_dir_path.as_ref())?;

        // Execute with timeout
        let execution_result = self.execute_with_timeout(&mut cmd, &self.config.max_execution_time_ms)?;

        let execution_time = start_time.elapsed().as_millis() as u64;

        // Monitor resource usage
        let resource_usage = self.monitor_resource_usage(&execution_result)?;

        // Check for security violations
        let security_violations = self.detect_security_violations(&execution_result)?;

        let sandbox_result = SandboxExecutionResult {
            success: execution_result.exit_code == Some(0),
            stdout: execution_result.stdout,
            stderr: execution_result.stderr,
            exit_code: execution_result.exit_code,
            execution_time_ms: execution_time,
            memory_used_mb: resource_usage.memory_mb,
            cpu_percent: resource_usage.cpu_percent,
            security_violations,
            warnings: execution_result.warnings,
            sandbox_id: session_id,
        };

        Ok(sandbox_result)
    }

    fn perform_security_checks(
        &self,
        tool: &dyn Tool,
        input: &serde_json::Value,
        context: &ToolExecutionContext,
    ) -> Result<(), ToolError> {
        // Check tool permissions against user tier
        let required_permissions = tool.requires_permissions();
        for permission in &required_permissions {
            if !self.check_permission(permission, &context.user_tier) {
                return Err(ToolError::PermissionDenied(
                    format!("Permission denied: {}", permission)
                ));
            }
        }

        // Check input for malicious patterns
        self.validate_input_security(input)?;

        Ok(())
    }

    fn check_permission(&self, permission: &str, user_tier: &str) -> bool {
        match user_tier {
            "ultra" => true, // Ultra tier has all permissions
            "premium" => !permission.starts_with("admin.") && !permission.starts_with("system."),
            "free" | "basic" => !permission.starts_with("admin.")
                && !permission.starts_with("system.")
                && !permission.starts_with("advanced."),
            _ => false,
        }
    }

    fn create_sandbox_command_with_path(
        &self,
        tool: &dyn Tool,
        temp_dir_path: Option<&std::path::PathBuf>,
    ) -> Result<Command, ToolError> {
        let mut cmd = Command::new("docker");
        
        cmd.args([
            "run",
            "--rm",
            "--network=none",
            "--memory", &format!("{}m", self.config.max_memory_mb),
            "--cpus", &format!("{}", self.config.max_cpu_percent / 100.0),
        ]);

        if let Some(path) = temp_dir_path {
            cmd.args(["-v", &format!("{}:/sandbox", path.display())]);
            cmd.current_dir(path);
        }

        // Use a minimal sandbox image
        cmd.arg("alpine:latest");
        cmd.arg("sh");
        cmd.arg("-c");
        cmd.arg(&format!("echo 'Executing tool {}'", tool.name()));

        Ok(cmd)
    }


    fn validate_input_security(&self, input: &serde_json::Value) -> Result<(), ToolError> {
        let input_str = serde_json::to_string(input)?;
        
        // Check for common attack patterns
        let dangerous_patterns = vec![
            "rm -rf",
            "sudo",
            "chmod 777",
            "system(",
            "eval(",
            "exec(",
            "__import__",
            "subprocess",
            "os.system",
            "Runtime.getRuntime()",
            "ProcessBuilder",
        ];

        for pattern in &dangerous_patterns {
            if input_str.contains(pattern) {
                return Err(ToolError::PermissionDenied(
                    format!("Potentially dangerous input detected: {}", pattern)
                ));
            }
        }

        Ok(())
    }

    fn create_sandbox_command(
        &self,
        _tool: &dyn Tool,
        session: &SandboxSession,
    ) -> Result<Command, ToolError> {
        let mut cmd = Command::new("docker");
        
        cmd.args([
            "run",
            "--rm",
            "--network=none", // Disable network by default
            "--memory", &format!("{}m", self.config.max_memory_mb),
            "--cpus", &format!("{}", self.config.max_cpu_percent / 100.0),
            "--read-only", // Read-only filesystem
        ]);

        // Mount allowed directories
        for dir in &self.config.allowed_directories {
            if let Some(temp_dir) = &session.temp_dir {
                let mount_path = temp_dir.path().join(dir.file_name().unwrap_or_default());
                cmd.args(["-v", &format!("{}:{}", dir.display(), mount_path.display())]);
            }
        }

        // Set environment variables
        for (key, value) in &self.config.environment_variables {
            cmd.args(["-e", &format!("{}={}", key, value)]);
        }

        // Use a minimal sandbox image
        cmd.arg("alpine:latest");
        cmd.arg("sh");
        cmd.arg("-c");
        cmd.arg("echo 'Sandbox ready'");

        Ok(cmd)
    }

    fn execute_with_timeout(
        &self,
        cmd: &mut Command,
        timeout_ms: &u64,
    ) -> Result<ExecutionResult, ToolError> {
        let timeout_duration = Duration::from_millis(*timeout_ms);
        
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = cmd.spawn()
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to start process: {}", e)))?;

        let start_time = Instant::now();
        
        // Wait for completion with timeout (blocking polling)
        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    let execution_time = start_time.elapsed();

                    let stdout = if let Some(mut stdout) = child.stdout.take() {
                        use std::io::Read;
                        let mut output = String::new();
                        let _ = stdout.read_to_string(&mut output);
                        output
                    } else {
                        String::new()
                    };

                    let stderr = if let Some(mut stderr) = child.stderr.take() {
                        use std::io::Read;
                        let mut output = String::new();
                        let _ = stderr.read_to_string(&mut output);
                        output
                    } else {
                        String::new()
                    };

                    return Ok(ExecutionResult {
                        success: status.success(),
                        stdout,
                        stderr,
                        exit_code: status.code(),
                        execution_time,
                        memory_used_mb: 0, // Would need external monitoring
                        cpu_percent: 0.0, // Would need external monitoring
                        security_violations: Vec::new(),
                        warnings: Vec::new(),
                    });
                }
                Ok(None) => {
                    if start_time.elapsed() >= timeout_duration {
                        let _ = child.kill();
                        let _ = child.wait();
                        return Err(ToolError::ExecutionFailed(format!(
                            "Execution timed out after {}ms",
                            timeout_ms
                        )));
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }
                Err(e) => {
                    return Err(ToolError::ExecutionFailed(format!(
                        "Process error: {}",
                        e
                    )));
                }
            }
        }
    }

    fn monitor_resource_usage(&self, result: &ExecutionResult) -> Result<ResourceUsage, ToolError> {
        // In a real implementation, this would use system calls or external monitoring
        // For now, return placeholder values
        Ok(ResourceUsage {
            memory_mb: 64, // Placeholder
            cpu_percent: 15.0, // Placeholder
        })
    }

    fn detect_security_violations(&self, result: &ExecutionResult) -> Result<Vec<String>, ToolError> {
        let mut violations = Vec::new();

        // Check stdout/stderr for suspicious activity
        let output = format!("{} {}", result.stdout, result.stderr);
        
        if output.contains("Permission denied") {
            violations.push("File system permission violation".to_string());
        }

        if output.contains("Network is unreachable") && !self.config.allow_network {
            // This is expected, not a violation
        }

        Ok(violations)
    }

    pub fn cleanup_session(&mut self, session_id: Uuid) -> Result<(), ToolError> {
        self.active_sessions.remove(&session_id);
        Ok(())
    }

    pub fn cleanup_expired_sessions(&mut self, max_age: Duration) {
        let now = Instant::now();
        self.active_sessions.retain(|_, session| {
            now.duration_since(session.last_activity) < max_age
        });
    }

    pub fn get_active_sessions(&self) -> Vec<Uuid> {
        self.active_sessions.keys().cloned().collect()
    }
}

#[derive(Debug, Clone)]
struct ExecutionResult {
    success: bool,
    stdout: String,
    stderr: String,
    exit_code: Option<i32>,
    execution_time: Duration,
    memory_used_mb: u64,
    cpu_percent: f32,
    security_violations: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Debug, Clone)]
struct ResourceUsage {
    memory_mb: u64,
    cpu_percent: f32,
}

impl Default for ToolSandbox {
    fn default() -> Self {
        Self::new(ToolSandboxConfig::default())
    }
}
