use crate::agent::{Agent, AgentMessage, AgentState, Task, TaskStatus, AgentContext};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use uuid::Uuid;

pub struct CoordinatorAgent {
    context: AgentContext,
    available_agents: Vec<String>,
    task_queue: Vec<Task>,
    #[allow(dead_code)]
    completed_tasks: Vec<Task>,
}

impl Default for CoordinatorAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl CoordinatorAgent {
    pub fn new() -> Self {
        let context = AgentContext::new("coordinator".to_string());
        
        Self {
            context,
            available_agents: vec![
                "editor".to_string(),
                "terminal".to_string(),
            ],
            task_queue: Vec::new(),
            completed_tasks: Vec::new(),
        }
    }
    
    pub fn add_task(&mut self, task: Task) {
        self.task_queue.push(task);
    }
    
    fn break_down_task(&self, task: &Task) -> Vec<Task> {
        let mut subtasks = Vec::new();
        
        // Simple task breakdown logic - this would be enhanced with AI
        if task.description.to_lowercase().contains("code") || task.description.to_lowercase().contains("file") {
            // Code-related task - assign to editor
            let editor_task = Task {
                id: Uuid::new_v4(),
                description: format!("Edit/Create file: {}", task.description),
                priority: task.priority,
                assigned_to: Some("editor".to_string()),
                status: TaskStatus::Pending,
                dependencies: Vec::new(),
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("parent_task".to_string(), json!(task.id));
                    meta.insert("task_type".to_string(), json!("code_edit"));
                    meta
                },
            };
            subtasks.push(editor_task);
        }
        
        if task.description.to_lowercase().contains("run") || task.description.to_lowercase().contains("execute") || task.description.to_lowercase().contains("test") {
            // Execution-related task - assign to terminal
            let terminal_task = Task {
                id: Uuid::new_v4(),
                description: format!("Execute command: {}", task.description),
                priority: task.priority,
                assigned_to: Some("terminal".to_string()),
                status: TaskStatus::Pending,
                dependencies: Vec::new(),
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("parent_task".to_string(), json!(task.id));
                    meta.insert("task_type".to_string(), json!("command_execution"));
                    meta
                },
            };
            subtasks.push(terminal_task);
        }
        
        // If no specific breakdown possible, keep as is
        if subtasks.is_empty() {
            let generic_task = Task {
                id: task.id,
                description: task.description.clone(),
                priority: task.priority,
                assigned_to: self.find_best_agent(&task.description),
                status: TaskStatus::Pending,
                dependencies: Vec::new(),
                metadata: task.metadata.clone(),
            };
            subtasks.push(generic_task);
        }
        
        subtasks
    }
    
    fn find_best_agent(&self, description: &str) -> Option<String> {
        let lower_desc = description.to_lowercase();
        
        for agent in &self.available_agents {
            match agent.as_str() {
                "editor" if lower_desc.contains("code") || lower_desc.contains("file") || lower_desc.contains("edit") => {
                    return Some(agent.clone());
                }
                "terminal" if lower_desc.contains("run") || lower_desc.contains("execute") || lower_desc.contains("command") => {
                    return Some(agent.clone());
                }
                _ => continue,
            }
        }
        
        // Default to first available agent
        self.available_agents.first().cloned()
    }
    
    fn create_assignment_message(&self, task: &Task) -> AgentMessage {
        AgentMessage {
            id: Uuid::new_v4(),
            sender: "coordinator".to_string(),
            recipient: task.assigned_to.clone().unwrap_or_default(),
            message_type: "task_assignment".to_string(),
            payload: json!({
                "task": task,
                "instructions": format!("Please execute this task: {}", task.description)
            }),
            timestamp: chrono::Utc::now(),
        }
    }
}

#[async_trait]
impl Agent for CoordinatorAgent {
    fn id(&self) -> &str {
        &self.context.agent_id
    }
    
    fn name(&self) -> &str {
        "Coordinator Agent"
    }
    
    fn state(&self) -> AgentState {
        self.context.state.clone()
    }
    
    fn set_state(&mut self, state: AgentState) {
        self.context.state = state;
    }
    
    async fn handle_message(&mut self, message: AgentMessage) -> Result<AgentMessage> {
        match message.message_type.as_str() {
            "new_task" => {
                // Handle new task submission
                if let Ok(task_data) = serde_json::from_value::<Task>(message.payload["task"].clone()) {
                    let subtasks = self.break_down_task(&task_data);
                    for subtask in subtasks {
                        self.add_task(subtask);
                    }
                    
                    Ok(AgentMessage {
                        id: Uuid::new_v4(),
                        sender: "coordinator".to_string(),
                        recipient: message.sender.clone(),
                        message_type: "task_received".to_string(),
                        payload: json!({
                            "status": "accepted",
                            "subtasks_created": self.task_queue.len()
                        }),
                        timestamp: chrono::Utc::now(),
                    })
                } else {
                    Ok(AgentMessage {
                        id: Uuid::new_v4(),
                        sender: "coordinator".to_string(),
                        recipient: message.sender.clone(),
                        message_type: "error".to_string(),
                        payload: json!({
                            "error": "Invalid task format"
                        }),
                        timestamp: chrono::Utc::now(),
                    })
                }
            }
            "task_status_update" => {
                // Handle status updates from other agents
                if let Some(task_id) = message.payload["task_id"].as_str() {
                    if let Ok(uuid) = Uuid::parse_str(task_id) {
                        if let Some(index) = self.task_queue.iter().position(|t| t.id == uuid) {
                            let task = &mut self.task_queue[index];
                            if let Some(status_str) = message.payload["status"].as_str() {
                                match status_str {
                                    "completed" => task.status = TaskStatus::Completed,
                                    "failed" => task.status = TaskStatus::Failed,
                                    "in_progress" => task.status = TaskStatus::InProgress,
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                
                Ok(AgentMessage {
                    id: Uuid::new_v4(),
                    sender: "coordinator".to_string(),
                    recipient: message.sender.clone(),
                    message_type: "status_acknowledged".to_string(),
                    payload: json!({"status": "acknowledged"}),
                    timestamp: chrono::Utc::now(),
                })
            }
            _ => {
                Ok(AgentMessage {
                    id: Uuid::new_v4(),
                    sender: "coordinator".to_string(),
                    recipient: message.sender.clone(),
                    message_type: "unknown_message".to_string(),
                    payload: json!({"error": "Unknown message type"}),
                    timestamp: chrono::Utc::now(),
                })
            }
        }
    }
    
    async fn process_task(&mut self, task: Task) -> Result<TaskStatus> {
        // For coordinator, processing means breaking down and delegating
        let subtasks = self.break_down_task(&task);
        
        for subtask in subtasks {
            self.add_task(subtask);
        }
        
        Ok(TaskStatus::InProgress)
    }
    
    async fn make_decision(&self, _context: &HashMap<String, Value>) -> Result<Value> {
        let pending_tasks = self.task_queue.len();
        let should_act = pending_tasks > 0;
        
        Ok(json!({
            "should_act": should_act,
            "pending_tasks": pending_tasks,
            "action": if should_act { "assign_tasks" } else { "wait" },
            "reasoning": format!("{} tasks waiting for assignment", pending_tasks)
        }))
    }
    
    async fn execute_action(&mut self, action: &str, _params: &HashMap<String, Value>) -> Result<Value> {
        match action {
            "assign_tasks" => {
                let mut assigned_tasks = Vec::new();
                
                // Assign tasks to appropriate agents
                let mut tasks_to_assign = Vec::new();
                std::mem::swap(&mut tasks_to_assign, &mut self.task_queue);
                
                for task in tasks_to_assign {
                    if let Some(agent) = &task.assigned_to {
                        let message = self.create_assignment_message(&task);
                        assigned_tasks.push(json!({
                            "task_id": task.id,
                            "assigned_to": agent,
                            "message_id": message.id
                        }));
                        
                        // In a real implementation, this would send the message via Redis
                        self.context.add_message(message);
                    }
                }
                
                Ok(json!({
                    "status": "success",
                    "assigned_tasks": assigned_tasks
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
            "task_planning".to_string(),
            "task_delegation".to_string(),
            "agent_coordination".to_string(),
            "workflow_orchestration".to_string(),
        ]
    }
}
