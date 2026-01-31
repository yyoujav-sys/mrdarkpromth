use crate::agent::{Agent, AgentMessage, AgentState, Task, TaskStatus, AgentContext};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::process::{Command as TokioCommand};
use tokio::time::timeout;
use uuid::Uuid;

pub struct TerminalAgent {
    context: AgentContext,
    working_directory: String,
    command_history: Vec<CommandExecution>,
    environment_variables: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct CommandExecution {
    pub id: Uuid,
    pub command: String,
    pub args: Vec<String>,
    pub working_dir: String,
    pub status: ExecutionStatus,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub execution_time: Duration,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Timeout,
    Killed,
}

impl TerminalAgent {
    pub fn new(working_directory: String) -> Self {
        let context = AgentContext::new("terminal".to_string());
        
        Self {
            context,
            working_directory,
            command_history: Vec::new(),
            environment_variables: HashMap::new(),
        }
    }
    
    pub fn set_working_directory(&mut self, dir: String) {
        self.working_directory = dir;
    }
    
    pub fn set_environment_variable(&mut self, key: String, value: String) {
        self.environment_variables.insert(key, value);
    }
    
    async fn execute_command(&mut self, command: &str, args: Vec<String>, timeout_secs: u64) -> Result<Value> {
        let start_time = std::time::Instant::now();
        
        let execution = CommandExecution {
            id: Uuid::new_v4(),
            command: command.to_string(),
            args: args.clone(),
            working_dir: self.working_directory.clone(),
            status: ExecutionStatus::Running,
            stdout: String::new(),
            stderr: String::new(),
            exit_code: None,
            execution_time: Duration::from_secs(0),
            timestamp: chrono::Utc::now(),
        };
        
        let execution_id = execution.id;
        self.command_history.push(execution);
        
        let mut cmd = TokioCommand::new(command);
        cmd.args(&args)
            .current_dir(&self.working_directory)
            .envs(&self.environment_variables)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        
        let result = timeout(Duration::from_secs(timeout_secs), cmd.output()).await;
        
        let execution_time = start_time.elapsed();
        
        let (status, stdout, stderr, exit_code) = match result {
            Ok(Ok(output)) => {
                let stdout_str = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr_str = String::from_utf8_lossy(&output.stderr).to_string();
                let exit_code = output.status.code();
                
                let status = if output.status.success() {
                    ExecutionStatus::Completed
                } else {
                    ExecutionStatus::Failed
                };
                
                (status, stdout_str, stderr_str, exit_code)
            }
            Ok(Err(e)) => {
                (ExecutionStatus::Failed, String::new(), e.to_string(), None)
            }
            Err(_) => {
                (ExecutionStatus::Timeout, String::new(), "Command timed out".to_string(), None)
            }
        };
        
        // Update the execution record
        if let Some(execution) = self.command_history.iter_mut().find(|e| e.id == execution_id) {
            execution.status = status.clone();
            execution.stdout = stdout.clone();
            execution.stderr = stderr.clone();
            execution.exit_code = exit_code;
            execution.execution_time = execution_time;
        }
        
        Ok(json!({
            "status": "success",
            "execution_id": execution_id,
            "command": command,
            "args": args,
            "working_dir": self.working_directory,
            "execution_status": format!("{:?}", status),
            "stdout": stdout,
            "stderr": stderr,
            "exit_code": exit_code,
            "execution_time_ms": execution_time.as_millis(),
            "timestamp": chrono::Utc::now()
        }))
    }
    
    async fn execute_shell_command(&mut self, shell_command: &str, timeout_secs: u64) -> Result<Value> {
        // Use shell to execute the command
        let shell = if cfg!(windows) { "cmd" } else { "bash" };
        let shell_arg = if cfg!(windows) { "/C" } else { "-c" };
        
        self.execute_command(shell, vec![shell_arg.to_string(), shell_command.to_string()], timeout_secs).await
    }
    
    async fn change_directory(&mut self, path: &str) -> Result<Value> {
        let new_path = if path.starts_with('/') || (cfg!(windows) && path.contains(':')) {
            path.to_string()
        } else {
            format!("{}/{}", self.working_directory.trim_end_matches('/'), path.trim_start_matches("./"))
        };
        
        // Check if directory exists
        if std::path::Path::new(&new_path).exists() {
            self.working_directory = new_path.clone();
            
            Ok(json!({
                "status": "success",
                "operation": "cd",
                "previous_directory": self.working_directory,
                "new_directory": new_path
            }))
        } else {
            Ok(json!({
                "status": "error",
                "operation": "cd",
                "message": format!("Directory does not exist: {}", new_path)
            }))
        }
    }
    
    async fn list_directory(&mut self, path: Option<&str>) -> Result<Value> {
        let target_path = path.unwrap_or(&self.working_directory).to_string();
        
        let result = match self.execute_command("ls", vec![target_path.clone()], 10).await {
            Ok(output) => Ok(output),
            Err(_) => self.execute_command("dir", vec![target_path.clone()], 10).await,
        };
        
        let response_path = target_path.clone();

        match result {
            Ok(output) => Ok(json!({
                "status": "success",
                "operation": "ls",
                "directory": response_path,
                "output": output
            })),
            Err(e) => Ok(json!({
                "status": "error",
                "operation": "ls",
                "message": e.to_string()
            })),
        }
    }
    
    fn parse_command_from_task(&self, task_description: &str) -> Option<(String, Vec<String>)> {
        let lower_desc = task_description.to_lowercase();
        
        // Extract command and arguments
        if lower_desc.contains("run") || lower_desc.contains("execute") {
            let words: Vec<&str> = task_description.split_whitespace().collect();
            if words.len() > 1 {
                let command = words[1].to_string();
                let args: Vec<String> = words[2..].iter().map(|s| s.to_string()).collect();
                return Some((command, args));
            }
        } else if lower_desc.contains("command:") {
            if let Some(start) = lower_desc.find("command:") {
                let start = start + 8;
                let command_part = lower_desc[start..].trim();
                let words: Vec<&str> = command_part.split_whitespace().collect();
                if !words.is_empty() {
                    let command = words[0].to_string();
                    let args: Vec<String> = words[1..].iter().map(|s| s.to_string()).collect();
                    return Some((command, args));
                }
            }
        } else {
            // Try to find common commands
            let common_commands = ["npm", "cargo", "git", "python", "node", "docker", "make", "gcc"];
            for cmd in &common_commands {
                if lower_desc.contains(cmd) {
                    let words: Vec<&str> = task_description.split_whitespace().collect();
                    if let Some(pos) = words.iter().position(|w| w.to_lowercase().contains(cmd)) {
                        let command = words[pos].to_string();
                        let args: Vec<String> = words[pos + 1..].iter().map(|s| s.to_string()).collect();
                        return Some((command, args));
                    }
                }
            }
        }
        
        None
    }
}

#[async_trait]
impl Agent for TerminalAgent {
    fn id(&self) -> &str {
        &self.context.agent_id
    }
    
    fn name(&self) -> &str {
        "Terminal Agent"
    }
    
    fn state(&self) -> AgentState {
        self.context.state.clone()
    }
    
    fn set_state(&mut self, state: AgentState) {
        self.context.state = state;
    }
    
    async fn handle_message(&mut self, message: AgentMessage) -> Result<AgentMessage> {
        match message.message_type.as_str() {
            "command_execution" => {
                let command = message.payload["command"].as_str().unwrap_or("");
                let args = message.payload["args"].as_array()
                    .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect())
                    .unwrap_or_default();
                let timeout_secs = message.payload["timeout_secs"].as_u64().unwrap_or(30);
                
                let result = self.execute_command(command, args, timeout_secs).await?;
                
                Ok(AgentMessage {
                    id: Uuid::new_v4(),
                    sender: "terminal".to_string(),
                    recipient: message.sender.clone(),
                    message_type: "command_result".to_string(),
                    payload: result,
                    timestamp: chrono::Utc::now(),
                })
            }
            "shell_command" => {
                let shell_command = message.payload["command"].as_str().unwrap_or("");
                let timeout_secs = message.payload["timeout_secs"].as_u64().unwrap_or(30);
                
                let result = self.execute_shell_command(shell_command, timeout_secs).await?;
                
                Ok(AgentMessage {
                    id: Uuid::new_v4(),
                    sender: "terminal".to_string(),
                    recipient: message.sender.clone(),
                    message_type: "shell_command_result".to_string(),
                    payload: result,
                    timestamp: chrono::Utc::now(),
                })
            }
            "change_directory" => {
                let path = message.payload["path"].as_str().unwrap_or("");
                let result = self.change_directory(path).await?;
                
                Ok(AgentMessage {
                    id: Uuid::new_v4(),
                    sender: "terminal".to_string(),
                    recipient: message.sender.clone(),
                    message_type: "directory_changed".to_string(),
                    payload: result,
                    timestamp: chrono::Utc::now(),
                })
            }
            "list_directory" => {
                let path = message.payload["path"].as_str();
                let result = self.list_directory(path).await?;
                
                Ok(AgentMessage {
                    id: Uuid::new_v4(),
                    sender: "terminal".to_string(),
                    recipient: message.sender.clone(),
                    message_type: "directory_list".to_string(),
                    payload: result,
                    timestamp: chrono::Utc::now(),
                })
            }
            _ => {
                Ok(AgentMessage {
                    id: Uuid::new_v4(),
                    sender: "terminal".to_string(),
                    recipient: message.sender.clone(),
                    message_type: "unknown_message".to_string(),
                    payload: json!({"error": "Unknown message type"}),
                    timestamp: chrono::Utc::now(),
                })
            }
        }
    }
    
    async fn process_task(&mut self, task: Task) -> Result<TaskStatus> {
        if let Some((command, args)) = self.parse_command_from_task(&task.description) {
            let result = self.execute_command(&command, args, 60).await;
            
            match result {
                Ok(output) => {
                    if let Some(status) = output["execution_status"].as_str() {
                        if status.contains("Completed") {
                            Ok(TaskStatus::Completed)
                        } else {
                            Ok(TaskStatus::Failed)
                        }
                    } else {
                        Ok(TaskStatus::Failed)
                    }
                }
                Err(_) => Ok(TaskStatus::Failed),
            }
        } else {
            Ok(TaskStatus::Failed)
        }
    }
    
    async fn make_decision(&self, context: &HashMap<String, Value>) -> Result<Value> {
        let recent_failures = self.command_history.iter()
            .rev()
            .take(5)
            .filter(|exec| matches!(exec.status, ExecutionStatus::Failed))
            .count();
        
        Ok(json!({
            "should_act": true, // Terminal agent should always be ready to execute commands
            "recent_failures": recent_failures,
            "working_directory": self.working_directory,
            "total_commands_executed": self.command_history.len(),
            "action": "ready_for_commands"
        }))
    }
    
    async fn execute_action(&mut self, action: &str, params: &HashMap<String, Value>) -> Result<Value> {
        match action {
            "ready_for_commands" => {
                Ok(json!({
                    "status": "ready",
                    "message": "Terminal agent is ready to execute commands",
                    "working_directory": self.working_directory
                }))
            }
            "cleanup_history" => {
                let keep_count = params.get("keep_count").and_then(|v| v.as_u64()).unwrap_or(100) as usize;
                let total_before = self.command_history.len();
                
                if self.command_history.len() > keep_count {
                    self.command_history.drain(0..self.command_history.len() - keep_count);
                }
                
                Ok(json!({
                    "status": "success",
                    "cleaned_entries": total_before - self.command_history.len(),
                    "remaining_entries": self.command_history.len()
                }))
            }
            _ => Ok(json!({
                "status": "error",
                "message": format!("Unknown action: {}", action)
            }))
        }
    }
    
    fn capabilities(&self) -> Vec<String> {
        vec![
            "command_execution".to_string(),
            "shell_command_execution".to_string(),
            "file_system_operations".to_string(),
            "process_management".to_string(),
            "environment_management".to_string(),
            "build_tools".to_string(),
            "package_management".to_string(),
        ]
    }
}
