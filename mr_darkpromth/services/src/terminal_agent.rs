use crate::agent_framework::{Agent, AgentError, AgentResult, AgentMessage, AgentState, AgentType};
use crate::redis_coordination::RedisCoordinator;
use std::sync::{Arc, Mutex};
use std::process::{Command, Stdio};
use std::io::BufReader;
use uuid::Uuid;
use chrono::Utc;

#[derive(Debug, Clone)]
pub struct CommandResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
}

pub struct TerminalAgent {
    id: String,
    state: AgentState,
    redis: Arc<Mutex<RedisCoordinator>>,
    workspace_path: String,
}

impl TerminalAgent {
    pub fn new(redis_url: &str, workspace_path: String) -> AgentResult<Self> {
        let redis = RedisCoordinator::new(redis_url, "terminal_agent".to_string())
            .map_err(|e| AgentError::CommunicationError(e.to_string()))?;
        
        Ok(Self {
            id: "terminal_agent".to_string(),
            state: AgentState::Idle,
            redis: Arc::new(Mutex::new(redis)),
            workspace_path,
        })
    }

    pub fn execute_command(&mut self, command: &str, args: &[String]) -> AgentResult<CommandResult> {
        self.state = AgentState::Acting;
        
        log::info!("Executing command: {} {:?}", command, args);
        
        let start_time = std::time::Instant::now();
        
        let output = Command::new(command)
            .args(args)
            .current_dir(&self.workspace_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| AgentError::ActionExecutionFailed(format!("Command execution failed: {}", e)))?;
        
        let duration = start_time.elapsed();
        
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code().unwrap_or(-1);
        
        let result = CommandResult {
            exit_code,
            stdout,
            stderr,
            duration_ms: duration.as_millis() as u64,
        };
        
        log::info!("Command completed with exit code: {}", exit_code);
        
        self.state = AgentState::Idle;
        Ok(result)
    }

    pub fn execute_command_with_output(&mut self, command: &str, args: &[String]) -> AgentResult<String> {
        let result = self.execute_command(command, args)?;
        
        if result.exit_code != 0 {
            return Err(AgentError::ActionExecutionFailed(format!(
                "Command failed with exit code {}: {}",
                result.exit_code,
                result.stderr
            )));
        }
        
        Ok(result.stdout)
    }

    pub fn run_tests(&mut self) -> AgentResult<CommandResult> {
        log::info!("Running tests");
        self.execute_command("cargo", &["test".to_string()])
    }

    pub fn build_project(&mut self) -> AgentResult<CommandResult> {
        log::info!("Building project");
        self.execute_command("cargo", &["build".to_string()])
    }

    pub fn check_code(&mut self) -> AgentResult<CommandResult> {
        log::info!("Checking code");
        self.execute_command("cargo", &["check".to_string()])
    }

    pub fn format_code(&mut self) -> AgentResult<CommandResult> {
        log::info!("Formatting code");
        self.execute_command("cargo", &["fmt".to_string()])
    }

    pub fn clippy_check(&mut self) -> AgentResult<CommandResult> {
        log::info!("Running clippy");
        self.execute_command("cargo", &["clippy".to_string()])
    }

    pub fn install_dependencies(&mut self) -> AgentResult<CommandResult> {
        log::info!("Installing dependencies");
        self.execute_command("cargo", &["build".to_string()])
    }

    pub fn git_status(&mut self) -> AgentResult<String> {
        self.execute_command_with_output("git", &["status".to_string()])
    }

    pub fn git_add(&mut self, files: &[String]) -> AgentResult<CommandResult> {
        let mut args = vec!["add".to_string()];
        args.extend(files.iter().cloned());
        self.execute_command("git", &args)
    }

    pub fn git_commit(&mut self, message: &str) -> AgentResult<CommandResult> {
        self.execute_command("git", &["commit".to_string(), "-m".to_string(), message.to_string()])
    }

    pub fn git_push(&mut self) -> AgentResult<CommandResult> {
        self.execute_command("git", &["push".to_string()])
    }

    pub fn list_files(&mut self, directory: &str) -> AgentResult<Vec<String>> {
        let result = self.execute_command_with_output("ls", &["-la".to_string(), directory.to_string()])?;
        
        let files: Vec<String> = result
            .lines()
            .skip(1)
            .map(|line| line.split_whitespace().last().unwrap_or("").to_string())
            .filter(|f| !f.is_empty() && *f != "." && *f != "..")
            .collect();
        
        Ok(files)
    }

    pub fn create_directory(&mut self, path: &str) -> AgentResult<CommandResult> {
        log::info!("Creating directory: {}", path);
        self.execute_command("mkdir", &["-p".to_string(), path.to_string()])
    }

    pub fn remove_file(&mut self, path: &str) -> AgentResult<CommandResult> {
        log::info!("Removing file: {}", path);
        self.execute_command("rm", &["-f".to_string(), path.to_string()])
    }

    pub fn copy_file(&mut self, src: &str, dst: &str) -> AgentResult<CommandResult> {
        log::info!("Copying file: {} -> {}", src, dst);
        self.execute_command("cp", &[src.to_string(), dst.to_string()])
    }

    fn publish_completion(&self, task_id: &str, subtask_id: &str, result: &CommandResult) -> AgentResult<()> {
        let mut redis = self.redis.lock()
            .map_err(|e| AgentError::CommunicationError(e.to_string()))?;
        
        let message = AgentMessage {
            id: Uuid::new_v4(),
            from_agent: self.id.clone(),
            to_agent: "coordinator_agent".to_string(),
            message_type: "subtask_completed".to_string(),
            content: serde_json::json!({
                "task_id": task_id,
                "subtask_id": subtask_id,
                "result": {
                    "exit_code": result.exit_code,
                    "stdout": result.stdout,
                    "stderr": result.stderr,
                    "duration_ms": result.duration_ms
                }
            }),
            timestamp: Utc::now(),
        };
        
        let event = crate::redis_coordination::CoordinationEvent {
            event_id: Uuid::new_v4().to_string(),
            agent_id: self.id.clone(),
            event_type: crate::redis_coordination::EventType::TaskCompletion,
            timestamp: Utc::now(),
            correlation_id: Some(message.id.to_string()),
            payload: serde_json::to_value(&message)
                .map_err(|e| AgentError::SerializationError(e.to_string()))?,
        };
        
        redis.publish_event(event)
            .map_err(|e| AgentError::CommunicationError(e.to_string()))?;
        
        Ok(())
    }

    fn publish_error(&self, task_id: &str, subtask_id: &str, error: &str) -> AgentResult<()> {
        let mut redis = self.redis.lock()
            .map_err(|e| AgentError::CommunicationError(e.to_string()))?;
        
        let message = AgentMessage {
            id: Uuid::new_v4(),
            from_agent: self.id.clone(),
            to_agent: "coordinator_agent".to_string(),
            message_type: "subtask_failed".to_string(),
            content: serde_json::json!({
                "task_id": task_id,
                "subtask_id": subtask_id,
                "error": error
            }),
            timestamp: Utc::now(),
        };
        
        let event = crate::redis_coordination::CoordinationEvent {
            event_id: Uuid::new_v4().to_string(),
            agent_id: self.id.clone(),
            event_type: crate::redis_coordination::EventType::ErrorEvent,
            timestamp: Utc::now(),
            correlation_id: Some(message.id.to_string()),
            payload: serde_json::to_value(&message)
                .map_err(|e| AgentError::SerializationError(e.to_string()))?,
        };
        
        redis.publish_event(event)
            .map_err(|e| AgentError::CommunicationError(e.to_string()))?;
        
        Ok(())
    }
}

impl Agent for TerminalAgent {
    fn agent_id(&self) -> &str {
        &self.id
    }

    fn agent_type(&self) -> AgentType {
        AgentType::Terminal
    }

    fn state(&self) -> AgentState {
        self.state.clone()
    }

    fn handle_message(&mut self, message: AgentMessage) -> AgentResult<()> {
        log::info!("Terminal received message from {}: {:?}", message.from_agent, message.message_type);
        
        match message.message_type.as_str() {
            "delegate_task" => {
                if let (Some(subtask_id), Some(description), Some(task_id)) = (
                    message.content.get("subtask_id").and_then(|v| v.as_str()),
                    message.content.get("description").and_then(|v| v.as_str()),
                    message.content.get("task_id").and_then(|v| v.as_str())
                ) {
                    // Determine command to execute based on description
                    let result = if description.contains("test") {
                        self.run_tests()
                    } else if description.contains("build") {
                        self.build_project()
                    } else if description.contains("check") {
                        self.check_code()
                    } else if description.contains("format") {
                        self.format_code()
                    } else if description.contains("clippy") {
                        self.clippy_check()
                    } else {
                        // Default: try to parse as command
                        let parts: Vec<&str> = description.split_whitespace().collect();
                        if parts.len() >= 2 {
                            let command = parts[0];
                            let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();
                            self.execute_command(command, &args)
                        } else {
                            return Err(AgentError::ActionExecutionFailed("Invalid command format".to_string()));
                        }
                    };
                    
                    match result {
                        Ok(cmd_result) => {
                            self.publish_completion(task_id, subtask_id, &cmd_result)?;
                        }
                        Err(e) => {
                            self.publish_error(task_id, subtask_id, &e.to_string())?;
                        }
                    }
                }
            }
            _ => {}
        }
        
        Ok(())
    }

    fn process(&mut self) -> AgentResult<()> {
        self.state = AgentState::Thinking;
        self.state = AgentState::Idle;
        Ok(())
    }

    fn shutdown(&mut self) -> AgentResult<()> {
        self.state = AgentState::Shutdown;
        log::info!("Terminal agent shutting down");
        Ok(())
    }
}
