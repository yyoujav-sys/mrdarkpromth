use crate::agent_framework::{Agent, AgentError, AgentResult, AgentMessage, AgentState, AgentType};
use crate::redis_coordination::RedisCoordinator;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub description: String,
    pub agent_type: AgentType,
    pub status: TaskStatus,
    pub subtasks: Vec<Subtask>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subtask {
    pub id: String,
    pub description: String,
    pub agent_type: AgentType,
    pub status: TaskStatus,
    pub dependencies: Vec<String>,
}

pub struct CoordinatorAgent {
    id: String,
    state: AgentState,
    redis: Arc<Mutex<RedisCoordinator>>,
    tasks: HashMap<String, Task>,
    pending_messages: Vec<AgentMessage>,
}

impl CoordinatorAgent {
    pub fn new(redis_url: &str) -> AgentResult<Self> {
        let redis = RedisCoordinator::new(redis_url, "coordinator_agent".to_string())
            .map_err(|e| AgentError::CommunicationError(e.to_string()))?;
        
        Ok(Self {
            id: "coordinator_agent".to_string(),
            state: AgentState::Idle,
            redis: Arc::new(Mutex::new(redis)),
            tasks: HashMap::new(),
            pending_messages: Vec::new(),
        })
    }

    pub fn receive_task(&mut self, description: String) -> AgentResult<String> {
        let task_id = Uuid::new_v4().to_string();
        let task = Task {
            id: task_id.clone(),
            description,
            agent_type: AgentType::Coordinator,
            status: TaskStatus::Pending,
            subtasks: Vec::new(),
            created_at: Utc::now(),
        };
        
        self.tasks.insert(task_id.clone(), task);
        Ok(task_id)
    }

    pub fn plan_task(&mut self, task_id: &str) -> AgentResult<()> {
        let description = {
            let task = self.tasks.get_mut(task_id)
                .ok_or_else(|| AgentError::MessageProcessingFailed("Task not found".to_string()))?;
            
            task.status = TaskStatus::InProgress;
            task.description.clone()
        };
        
        // Break down task into subtasks
        let subtasks = self.create_subtasks(&description);
        
        // Update task with subtasks
        if let Some(task) = self.tasks.get_mut(task_id) {
            task.subtasks = subtasks;
        }
        
        Ok(())
    }

    fn create_subtasks(&self, description: &str) -> Vec<Subtask> {
        let mut subtasks = Vec::new();
        
        // Analyze task and create subtasks
        if description.contains("code") || description.contains("implement") || description.contains("create") {
            subtasks.push(Subtask {
                id: Uuid::new_v4().to_string(),
                description: format!("Generate code for: {}", description),
                agent_type: AgentType::Editor,
                status: TaskStatus::Pending,
                dependencies: Vec::new(),
            });
        }
        
        if description.contains("test") || description.contains("run") || description.contains("execute") {
            subtasks.push(Subtask {
                id: Uuid::new_v4().to_string(),
                description: format!("Execute command for: {}", description),
                agent_type: AgentType::Terminal,
                status: TaskStatus::Pending,
                dependencies: Vec::new(),
            });
        }
        
        if description.contains("build") || description.contains("compile") {
            subtasks.push(Subtask {
                id: Uuid::new_v4().to_string(),
                description: "Build project".to_string(),
                agent_type: AgentType::Terminal,
                status: TaskStatus::Pending,
                dependencies: Vec::new(),
            });
        }
        
        subtasks
    }

    pub fn delegate_subtask(&mut self, task_id: &str, subtask_id: &str) -> AgentResult<()> {
        let task = self.tasks.get_mut(task_id)
            .ok_or_else(|| AgentError::MessageProcessingFailed("Task not found".to_string()))?;
        
        let subtask = task.subtasks.iter_mut()
            .find(|st| st.id == subtask_id)
            .ok_or_else(|| AgentError::MessageProcessingFailed("Subtask not found".to_string()))?;
        
        subtask.status = TaskStatus::InProgress;
        
        let message = AgentMessage {
            id: Uuid::new_v4(),
            from_agent: self.id.clone(),
            to_agent: format!("{:?}", subtask.agent_type).to_lowercase(),
            message_type: "delegate_task".to_string(),
            content: serde_json::json!({
                "subtask_id": subtask_id,
                "description": subtask.description.clone(),
                "task_id": task_id
            }),
            timestamp: Utc::now(),
        };
        
        self.publish_message(message)?;
        Ok(())
    }

    pub fn check_task_completion(&mut self, task_id: &str) -> AgentResult<bool> {
        let task = self.tasks.get(task_id)
            .ok_or_else(|| AgentError::MessageProcessingFailed("Task not found".to_string()))?;
        
        let all_completed = task.subtasks.iter()
            .all(|st| st.status == TaskStatus::Completed);
        
        if all_completed {
            let mut task = self.tasks.get_mut(task_id).unwrap();
            task.status = TaskStatus::Completed;
        }
        
        Ok(all_completed)
    }

    fn publish_message(&self, message: AgentMessage) -> AgentResult<()> {
        let mut redis = self.redis.lock()
            .map_err(|e| AgentError::CommunicationError(e.to_string()))?;
        
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

    pub fn get_task_status(&self, task_id: &str) -> Option<&Task> {
        self.tasks.get(task_id)
    }
}

impl Agent for CoordinatorAgent {
    fn agent_id(&self) -> &str {
        &self.id
    }

    fn agent_type(&self) -> AgentType {
        AgentType::Coordinator
    }

    fn state(&self) -> AgentState {
        self.state.clone()
    }

    fn handle_message(&mut self, message: AgentMessage) -> AgentResult<()> {
        log::info!("Coordinator received message from {}: {:?}", message.from_agent, message.message_type);
        
        match message.message_type.as_str() {
            "task_request" => {
                if let Some(desc) = message.content.get("description").and_then(|v| v.as_str()) {
                    let task_id = self.receive_task(desc.to_string())?;
                    self.plan_task(&task_id)?;
                    
                    // Delegate subtasks - extract subtasks first
                    let subtasks: Vec<_> = if let Some(task) = self.tasks.get(&task_id) {
                        task.subtasks.clone()
                    } else {
                        Vec::new()
                    };
                    
                    for subtask in &subtasks {
                        self.delegate_subtask(&task_id, &subtask.id)?;
                    }
                }
            }
            "subtask_completed" => {
                if let (Some(task_id), Some(subtask_id)) = (
                    message.content.get("task_id").and_then(|v| v.as_str()),
                    message.content.get("subtask_id").and_then(|v| v.as_str())
                ) {
                    let task = self.tasks.get_mut(task_id)
                        .ok_or_else(|| AgentError::MessageProcessingFailed("Task not found".to_string()))?;
                    
                    if let Some(subtask) = task.subtasks.iter_mut()
                        .find(|st| st.id == subtask_id) {
                        subtask.status = TaskStatus::Completed;
                    }
                    
                    self.check_task_completion(task_id)?;
                }
            }
            "subtask_failed" => {
                if let (Some(task_id), Some(subtask_id), Some(error)) = (
                    message.content.get("task_id").and_then(|v| v.as_str()),
                    message.content.get("subtask_id").and_then(|v| v.as_str()),
                    message.content.get("error").and_then(|v| v.as_str())
                ) {
                    let task = self.tasks.get_mut(task_id)
                        .ok_or_else(|| AgentError::MessageProcessingFailed("Task not found".to_string()))?;
                    
                    if let Some(subtask) = task.subtasks.iter_mut()
                        .find(|st| st.id == subtask_id) {
                        subtask.status = TaskStatus::Failed;
                    }
                    
                    task.status = TaskStatus::Failed;
                    log::error!("Task {} failed: {}", task_id, error);
                }
            }
            _ => {}
        }
        
        Ok(())
    }

    fn process(&mut self) -> AgentResult<()> {
        self.state = AgentState::Thinking;
        
        // Process pending messages
        let messages: Vec<AgentMessage> = self.pending_messages.drain(..).collect();
        for message in messages {
            self.handle_message(message)?;
        }
        
        self.state = AgentState::Idle;
        Ok(())
    }

    fn shutdown(&mut self) -> AgentResult<()> {
        self.state = AgentState::Shutdown;
        log::info!("Coordinator agent shutting down");
        Ok(())
    }
}
