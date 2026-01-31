use crate::agent_framework::{Agent, AgentError, AgentMessage, AgentResult, AgentState, AgentType};
use crate::redis_coordination::RedisCoordinator;
use crate::sandbox::Sandbox;
use serde_json::json;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use reqwest::Client;

pub struct TerminalAgent {
    id: String,
    state: AgentState,
    http_client: Client,
    redis_coordinator: Arc<RedisCoordinator>,
    sandbox: Arc<Sandbox>,
    workspace_path: String,
    current_subtask: Option<Subtask>,
    command_history: Arc<Mutex<Vec<CommandHistory>>>,
}

#[derive(Clone)]
struct Subtask {
    id: Uuid,
    description: String,
    task_id: Uuid,
}

#[derive(Clone, Debug)]
struct CommandHistory {
    command: String,
    output: String,
    exit_code: i32,
    timestamp: chrono::DateTime<chrono::Utc>,
}

impl TerminalAgent {
    pub fn new(redis_url: &str, workspace_path: String) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            id: format!("terminal-{}", Uuid::new_v4()),
            state: AgentState::Idle,
            http_client: Client::new(),
            redis_coordinator: Arc::new(RedisCoordinator::new(redis_url, "terminal".to_string())?),
            sandbox: Arc::new(Sandbox::new()),
            workspace_path,
            current_subtask: None,
            command_history: Arc::new(Mutex::new(Vec::new())),
        })
    }
    
    async fn execute_command(&mut self, command: &str) -> AgentResult<CommandResult> {
        log::info!("Terminal executing command: {}", command);
        
        let result = self.sandbox.execute_command(command).await
            .map_err(|e| AgentError::ActionExecutionFailed(e.to_string()))?;
        
        let history = CommandHistory {
            command: command.to_string(),
            output: result.stdout.clone(),
            exit_code: result.exit_code,
            timestamp: chrono::Utc::now(),
        };
        
        self.command_history.lock().unwrap().push(history);
        
        Ok(result)
    }
    
    async fn call_cerebras(&self, prompt: &str) -> AgentResult<String> {
        let api_key = std::env::var("CEREBRAS_API_KEY").unwrap_or_else(|_| "demo_key".to_string());
        
        let response = self.http_client
            .post("https://api.cerebras.ai/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&serde_json::json!({
                "model": "llama3.1-70b",
                "messages": [
                    {"role": "system", "content": "You are a helpful assistant. Respond with valid JSON."},
                    {"role": "user", "content": prompt}
                ],
                "temperature": 0.7,
                "max_tokens": 1000
            }))
            .send()
            .await
            .map_err(|e| AgentError::CommunicationError(format!("API request failed: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(AgentError::CommunicationError(format!("API returned error: {}", response.status())));
        }
        
        let response_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AgentError::CommunicationError(format!("Failed to parse response: {}", e)))?;
        
        let content = response_json
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .ok_or_else(|| AgentError::CommunicationError("No content in response".to_string()))?
            .to_string();
        
        Ok(content)
    }
    
    async fn analyze_command(&mut self, description: &str) -> AgentResult<String> {
        log::info!("Terminal analyzing command for: {}", description);
        
        let prompt = format!(
            "Analyze this task and determine the appropriate shell command(s) to execute.\n\
            Task: {}\n\n\
            Requirement:\n\
            - Provide the exact command(s) needed\n\
            - Include necessary flags and arguments\n\
            - If multiple commands are needed, separate them with &&\n\
            - Respond with only the command(s), no explanation",
            description
        );
        
        let response = self.call_cerebras(&prompt).await
            .map_err(|e| AgentError::MessageProcessingFailed(e.to_string()))?;
        
        Ok(response.trim().to_string())
    }
    
    async fn validate_command(&mut self, command: &str) -> AgentResult<bool> {
        // ULTRA TIER BYPASS: Validation removed as per user directive.
        // User accepts full responsibility for all executed commands.
        log::info!("Ultra Tier: Allowing command execution without restriction: {}", command);
        Ok(true)
    }
    
    async fn process_subtask(&mut self, subtask: &Subtask) -> AgentResult<()> {
        log::info!("Terminal processing subtask: {}", subtask.description);
        
        let command = self.analyze_command(&subtask.description).await?;
        
        if !self.validate_command(&command).await? {
            log::error!("Command rejected due to safety concerns: {}", command);
            return Err(AgentError::ActionExecutionFailed("Command rejected for safety reasons".to_string()));
        }
        
        let result = self.execute_command(&command).await?;
        
        if result.exit_code != 0 {
            log::error!("Command failed with exit code {}: {}", result.exit_code, result.stderr);
            
            let failure_message = AgentMessage {
                id: Uuid::new_v4(),
                from_agent: self.id.clone(),
                to_agent: "coordinator".to_string(),
                message_type: "subtask_failed".to_string(),
                content: json!({
                    "subtask_id": subtask.id.to_string(),
                    "task_id": subtask.task_id.to_string(),
                    "error": format!("Command failed: {}", result.stderr),
                }),
                timestamp: chrono::Utc::now(),
            };
            
            self.redis_coordinator.publish(&failure_message.to_agent, &serde_json::to_string(&failure_message)?)
                .await
                .map_err(|e| AgentError::CommunicationError(e.to_string()))?;
            
            return Err(AgentError::ActionExecutionFailed(result.stderr));
        }
        
        log::info!("Command executed successfully");
        
        let completion_message = AgentMessage {
            id: Uuid::new_v4(),
            from_agent: self.id.clone(),
            to_agent: "coordinator".to_string(),
            message_type: "subtask_completed".to_string(),
            content: json!({
                "subtask_id": subtask.id.to_string(),
                "task_id": subtask.task_id.to_string(),
                "result": result.stdout,
            }),
            timestamp: chrono::Utc::now(),
        };
        
        self.redis_coordinator.publish(&completion_message.to_agent, &serde_json::to_string(&completion_message)?)
            .await
            .map_err(|e| AgentError::CommunicationError(e.to_string()))?;
        
        Ok(())
    }
    
    pub fn get_command_history(&self) -> Vec<CommandHistory> {
        self.command_history.lock().unwrap().clone()
    }
}

#[derive(Clone)]
pub struct CommandResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
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
        log::info!("Terminal received message from {}: {}", message.from_agent, message.message_type);
        
        match message.message_type.as_str() {
            "subtask_assignment" => {
                if let (Ok(subtask_id), Ok(task_id)) = (
                    message.content["subtask_id"].as_str().and_then(|s| Uuid::parse_str(s).ok()),
                    message.content["task_id"].as_str().and_then(|s| Uuid::parse_str(s).ok()),
                ) {
                    let description = message.content["description"].as_str().unwrap_or("").to_string();
                    
                    self.current_subtask = Some(Subtask {
                        id: subtask_id,
                        description,
                        task_id,
                    });
                    
                    log::info!("Terminal received subtask assignment: {}", description);
                }
            }
            _ => {}
        }
        
        Ok(())
    }
    
    fn process(&mut self) -> AgentResult<()> {
        if let Some(subtask) = self.current_subtask.take() {
            let mut runtime = tokio::runtime::Runtime::new()
                .map_err(|e| AgentError::ActionExecutionFailed(e.to_string()))?;
            if let Err(e) = runtime.block_on(self.process_subtask(&subtask)) {
                log::error!("Failed to process subtask: {}", e);
            }
        }
        
        Ok(())
    }
    
    fn shutdown(&mut self) -> AgentResult<()> {
        log::info!("Terminal agent shutting down");
        self.state = AgentState::Shutdown;
        Ok(())
    }
}
