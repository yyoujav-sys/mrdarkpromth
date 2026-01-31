use crate::agent_framework::{Agent, AgentError, AgentMessage, AgentResult, AgentState, AgentType};
use crate::redis_coordination::RedisCoordinator;
use crate::tool_system::ToolSystem;
use serde_json::json;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use reqwest::Client;

pub struct EditorAgent {
    id: String,
    state: AgentState,
    http_client: Client,
    redis_coordinator: Arc<RedisCoordinator>,
    tool_system: Arc<ToolSystem>,
    workspace_path: String,
    current_subtask: Option<Subtask>,
}

#[derive(Clone)]
struct Subtask {
    id: Uuid,
    description: String,
    task_id: Uuid,
}

impl EditorAgent {
    pub fn new(redis_url: &str, workspace_path: String) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            id: format!("editor-{}", Uuid::new_v4()),
            state: AgentState::Idle,
            http_client: Client::new(),
            redis_coordinator: Arc::new(RedisCoordinator::new(redis_url, "editor".to_string())?),
            tool_system: Arc::new(ToolSystem::new()),
            workspace_path,
            current_subtask: None,
        })
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
    
    async fn generate_code(&mut self, description: &str) -> AgentResult<String> {
        log::info!("Editor generating code for: {}", description);
        
        let prompt = format!(
            "Generate code for the following task. Provide complete, functional code:\n\n{}\n\n\
            Requirements:\n\
            - Use modern best practices\n\
            - Include necessary imports\n\
            - Add error handling\n\
            - Make code production-ready",
            description
        );
        
        let response = self.call_cerebras(&prompt).await
            .map_err(|e| AgentError::ActionExecutionFailed(e.to_string()))?;
        
        Ok(response)
    }
    
    async fn edit_file(&mut self, file_path: &str, content: &str) -> AgentResult<()> {
        log::info!("Editor editing file: {}", file_path);
        
        let result = self.tool_system.execute_tool("write_file", json!({
            "path": file_path,
            "content": content,
        })).await;
        
        match result {
            Ok(_) => {
                log::info!("Successfully edited file: {}", file_path);
                Ok(())
            }
            Err(e) => Err(AgentError::ActionExecutionFailed(e.to_string())),
        }
    }
    
    async fn read_file(&mut self, file_path: &str) -> AgentResult<String> {
        log::info!("Editor reading file: {}", file_path);
        
        let result = self.tool_system.execute_tool("read_file", json!({
            "path": file_path,
        })).await;
        
        match result {
            Ok(output) => {
                let content = output.get("content").and_then(|v| v.as_str())
                    .ok_or_else(|| AgentError::ActionExecutionFailed("No content returned".to_string()))?;
                Ok(content.to_string())
            }
            Err(e) => Err(AgentError::ActionExecutionFailed(e.to_string())),
        }
    }
    
    async fn analyze_code(&mut self, code: &str) -> AgentResult<serde_json::Value> {
        log::info!("Editor analyzing code");
        
        let prompt = format!(
            "Analyze this code and provide:\n\
            1. Code quality assessment\n\
            2. Potential bugs or issues\n\
            3. Security vulnerabilities\n\
            4. Performance considerations\n\
            5. Suggestions for improvement\n\n\
            Code:\n{}",
            code
        );
        
        let response = self.call_cerebras(&prompt).await
            .map_err(|e| AgentError::MessageProcessingFailed(e.to_string()))?;
        
        let analysis: serde_json::Value = serde_json::from_str(&response)
            .unwrap_or_else(|_| json!({"analysis": response}));
        
        Ok(analysis)
    }
    
    async fn process_subtask(&mut self, subtask: &Subtask) -> AgentResult<()> {
        log::info!("Editor processing subtask: {}", subtask.description);
        
        let code = self.generate_code(&subtask.description).await?;
        
        let file_path = format!("generated_{}.rs", subtask.id);
        self.edit_file(&file_path, &code).await?;
        
        let analysis = self.analyze_code(&code).await?;
        log::info!("Code analysis: {}", serde_json::to_string(&analysis)?);
        
        let completion_message = AgentMessage {
            id: Uuid::new_v4(),
            from_agent: self.id.clone(),
            to_agent: "coordinator".to_string(),
            message_type: "subtask_completed".to_string(),
            content: json!({
                "subtask_id": subtask.id.to_string(),
                "task_id": subtask.task_id.to_string(),
                "result": "Code generated successfully",
            }),
            timestamp: chrono::Utc::now(),
        };
        
        self.redis_coordinator.publish(&completion_message.to_agent, &serde_json::to_string(&completion_message)?)
            .await
            .map_err(|e| AgentError::CommunicationError(e.to_string()))?;
        
        Ok(())
    }
}

impl Agent for EditorAgent {
    fn agent_id(&self) -> &str {
        &self.id
    }
    
    fn agent_type(&self) -> AgentType {
        AgentType::Editor
    }
    
    fn state(&self) -> AgentState {
        self.state.clone()
    }
    
    fn handle_message(&mut self, message: AgentMessage) -> AgentResult<()> {
        log::info!("Editor received message from {}: {}", message.from_agent, message.message_type);
        
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
                    
                    log::info!("Editor received subtask assignment: {}", description);
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
        log::info!("Editor agent shutting down");
        self.state = AgentState::Shutdown;
        Ok(())
    }
}
