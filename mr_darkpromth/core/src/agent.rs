use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AgentState {
    Idle,
    Thinking,
    Acting,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    pub id: Uuid,
    pub sender: String,
    pub recipient: String,
    pub message_type: String,
    pub payload: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub description: String,
    pub priority: u8,
    pub assigned_to: Option<String>,
    pub status: TaskStatus,
    pub dependencies: Vec<Uuid>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

#[async_trait]
pub trait Agent: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn state(&self) -> AgentState;
    fn set_state(&mut self, state: AgentState);
    
    async fn handle_message(&mut self, message: AgentMessage) -> anyhow::Result<AgentMessage>;
    async fn process_task(&mut self, task: Task) -> anyhow::Result<TaskStatus>;
    async fn make_decision(&self, context: &HashMap<String, serde_json::Value>) -> anyhow::Result<serde_json::Value>;
    async fn execute_action(&mut self, action: &str, params: &HashMap<String, serde_json::Value>) -> anyhow::Result<serde_json::Value>;
    
    fn capabilities(&self) -> Vec<String>;
    fn can_handle_task(&self, task: &Task) -> bool {
        self.capabilities().iter().any(|cap| {
            task.description.to_lowercase().contains(&cap.to_lowercase())
        })
    }
}

pub struct AgentContext {
    pub agent_id: String,
    pub state: AgentState,
    pub current_tasks: Vec<Task>,
    pub message_history: Vec<AgentMessage>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl AgentContext {
    pub fn new(agent_id: String) -> Self {
        Self {
            agent_id,
            state: AgentState::Idle,
            current_tasks: Vec::new(),
            message_history: Vec::new(),
            metadata: HashMap::new(),
        }
    }
    
    pub fn add_task(&mut self, task: Task) {
        self.current_tasks.push(task);
    }
    
    pub fn complete_task(&mut self, task_id: Uuid) -> Option<Task> {
        let index = self.current_tasks.iter().position(|t| t.id == task_id);
        if let Some(index) = index {
            Some(self.current_tasks.remove(index))
        } else {
            None
        }
    }
    
    pub fn add_message(&mut self, message: AgentMessage) {
        self.message_history.push(message);
    }
}
