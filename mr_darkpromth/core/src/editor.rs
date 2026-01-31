use crate::agent::{Agent, AgentMessage, AgentState, Task, TaskStatus, AgentContext};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use uuid::Uuid;

pub struct EditorAgent {
    context: AgentContext,
    workspace_root: String,
    current_file: Option<String>,
    file_operations: Vec<FileOperation>,
}

#[derive(Debug, Clone)]
pub struct FileOperation {
    pub id: Uuid,
    pub operation_type: FileOperationType,
    pub file_path: String,
    pub content: Option<String>,
    pub status: OperationStatus,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub enum FileOperationType {
    Create,
    Read,
    Update,
    Delete,
    Search,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OperationStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

impl EditorAgent {
    pub fn new(workspace_root: String) -> Self {
        let context = AgentContext::new("editor".to_string());
        
        Self {
            context,
            workspace_root,
            current_file: None,
            file_operations: Vec::new(),
        }
    }
    
    fn parse_file_path(&self, path: &str) -> String {
        if Path::new(path).is_absolute() {
            path.to_string()
        } else {
            format!("{}/{}", self.workspace_root.trim_end_matches('/'), path.trim_start_matches("./"))
        }
    }
    
    async fn create_file(&mut self, file_path: &str, content: &str) -> Result<Value> {
        let full_path = self.parse_file_path(file_path);
        let full_path_clone = full_path.clone();
        
        // Ensure parent directory exists
        if let Some(parent) = Path::new(&full_path).parent() {
            fs::create_dir_all(parent)?;
        }
        
        fs::write(&full_path, content)?;
        
        let operation = FileOperation {
            id: Uuid::new_v4(),
            operation_type: FileOperationType::Create,
            file_path: full_path,
            content: Some(content.to_string()),
            status: OperationStatus::Completed,
            timestamp: chrono::Utc::now(),
        };
        
        self.file_operations.push(operation);
        let response_path = full_path_clone.clone();
        self.current_file = Some(response_path.clone());
        
        Ok(json!({
            "status": "success",
            "operation": "create",
            "file_path": response_path,
            "bytes_written": content.len()
        }))
    }
    
    async fn read_file(&mut self, file_path: &str) -> Result<Value> {
        let full_path = self.parse_file_path(file_path);
        let full_path_clone = full_path.clone();
        
        let content = fs::read_to_string(&full_path)?;
        
        let operation = FileOperation {
            id: Uuid::new_v4(),
            operation_type: FileOperationType::Read,
            file_path: full_path.clone(),
            content: Some(content.clone()),
            status: OperationStatus::Completed,
            timestamp: chrono::Utc::now(),
        };
        
        self.file_operations.push(operation);
        self.current_file = Some(full_path_clone);
        
        Ok(json!({
            "status": "success",
            "operation": "read",
            "file_path": full_path,
            "content": content,
            "lines": content.lines().count()
        }))
    }
    
    async fn update_file(&mut self, file_path: &str, content: &str) -> Result<Value> {
        let full_path = self.parse_file_path(file_path);
        let full_path_clone = full_path.clone();
        
        // Read existing content first
        let existing_content = fs::read_to_string(&full_path).unwrap_or_default();
        
        fs::write(&full_path, content)?;
        
        let operation = FileOperation {
            id: Uuid::new_v4(),
            operation_type: FileOperationType::Update,
            file_path: full_path_clone,
            content: Some(content.to_string()),
            status: OperationStatus::Completed,
            timestamp: chrono::Utc::now(),
        };
        
        self.file_operations.push(operation);
        let response_path = full_path.clone();
        self.current_file = Some(full_path);
        
        Ok(json!({
            "status": "success",
            "operation": "update",
            "file_path": response_path,
            "previous_lines": existing_content.lines().count(),
            "new_lines": content.lines().count()
        }))
    }
    
    async fn delete_file(&mut self, file_path: &str) -> Result<Value> {
        let full_path = self.parse_file_path(file_path);
        
        fs::remove_file(&full_path)?;
        
        let operation = FileOperation {
            id: Uuid::new_v4(),
            operation_type: FileOperationType::Delete,
            file_path: full_path.clone(),
            content: None,
            status: OperationStatus::Completed,
            timestamp: chrono::Utc::now(),
        };
        
        self.file_operations.push(operation);
        
        if self.current_file.as_ref() == Some(&full_path) {
            self.current_file = None;
        }
        
        Ok(json!({
            "status": "success",
            "operation": "delete",
            "file_path": full_path
        }))
    }
    
    async fn search_in_file(&mut self, file_path: &str, pattern: &str) -> Result<Value> {
        let full_path = self.parse_file_path(file_path);
        
        let content = fs::read_to_string(&full_path)?;
        let lines: Vec<&str> = content.lines().collect();
        let mut matches = Vec::new();
        
        for (line_num, line) in lines.iter().enumerate() {
            if line.contains(pattern) {
                matches.push(json!({
                    "line_number": line_num + 1,
                    "content": line,
                    "match_positions": self.find_pattern_positions(line, pattern)
                }));
            }
        }
        
        let operation = FileOperation {
            id: Uuid::new_v4(),
            operation_type: FileOperationType::Search,
            file_path: full_path.clone(),
            content: None,
            status: OperationStatus::Completed,
            timestamp: chrono::Utc::now(),
        };
        
        self.file_operations.push(operation);
        
        Ok(json!({
            "status": "success",
            "operation": "search",
            "file_path": full_path,
            "pattern": pattern,
            "matches": matches,
            "total_matches": matches.len()
        }))
    }
    
    fn find_pattern_positions(&self, line: &str, pattern: &str) -> Vec<usize> {
        let mut positions = Vec::new();
        let mut start = 0;
        
        while let Some(pos) = line[start..].find(pattern) {
            positions.push(start + pos);
            start += pos + pattern.len();
        }
        
        positions
    }
    
    fn extract_file_operation(&self, task_description: &str) -> Option<(String, String, String)> {
        let lower_desc = task_description.to_lowercase();
        
        // Extract file path
        let file_path = if let Some(start) = lower_desc.find("file:") {
            let start = start + 5;
            let end = lower_desc[start..].find(' ').unwrap_or(lower_desc.len() - start);
            lower_desc[start..start + end].trim_matches('"').trim_matches('\'').to_string()
        } else if let Some(start) = lower_desc.find("path:") {
            let start = start + 5;
            let end = lower_desc[start..].find(' ').unwrap_or(lower_desc.len() - start);
            lower_desc[start..start + end].trim_matches('"').trim_matches('\'').to_string()
        } else {
            // Try to extract from common patterns
            if lower_desc.contains("create") || lower_desc.contains("new") {
                // Look for file extensions
                if let Some(dot_pos) = lower_desc.find('.') {
                    let start = lower_desc[..dot_pos].rfind(' ').map(|p| p + 1).unwrap_or(0);
                    let end = lower_desc[dot_pos..].find(' ').map(|p| dot_pos + p).unwrap_or(lower_desc.len());
                    lower_desc[start..end].to_string()
                } else {
                    return None;
                }
            } else {
                return None;
            }
        };
        
        // Extract operation type
        let operation = if lower_desc.contains("create") || lower_desc.contains("new") {
            "create"
        } else if lower_desc.contains("read") || lower_desc.contains("open") {
            "read"
        } else if lower_desc.contains("update") || lower_desc.contains("edit") || lower_desc.contains("modify") {
            "update"
        } else if lower_desc.contains("delete") || lower_desc.contains("remove") {
            "delete"
        } else if lower_desc.contains("search") || lower_desc.contains("find") {
            "search"
        } else {
            "read" // default
        };
        
        // Extract content for create/update operations
        let content = if operation == "create" || operation == "update" {
            if let Some(start) = lower_desc.find("content:") {
                let start = start + 8;
                lower_desc[start..].trim_matches('"').trim_matches('\'').to_string()
            } else {
                String::new()
            }
        } else if operation == "search" {
            if let Some(start) = lower_desc.find("pattern:") {
                let start = start + 8;
                lower_desc[start..].trim_matches('"').trim_matches('\'').to_string()
            } else {
                String::new()
            }
        } else {
            String::new()
        };
        
        Some((operation.to_string(), file_path, content))
    }
}

#[async_trait]
impl Agent for EditorAgent {
    fn id(&self) -> &str {
        &self.context.agent_id
    }
    
    fn name(&self) -> &str {
        "Editor Agent"
    }
    
    fn state(&self) -> AgentState {
        self.context.state.clone()
    }
    
    fn set_state(&mut self, state: AgentState) {
        self.context.state = state;
    }
    
    async fn handle_message(&mut self, message: AgentMessage) -> Result<AgentMessage> {
        match message.message_type.as_str() {
            "file_operation" => {
                let operation = message.payload["operation"].as_str().unwrap_or("read");
                let file_path = message.payload["file_path"].as_str().unwrap_or("");
                
                let result = match operation {
                    "create" => {
                        let content = message.payload["content"].as_str().unwrap_or("");
                        self.create_file(file_path, content).await?
                    }
                    "read" => self.read_file(file_path).await?,
                    "update" => {
                        let content = message.payload["content"].as_str().unwrap_or("");
                        self.update_file(file_path, content).await?
                    }
                    "delete" => self.delete_file(file_path).await?,
                    "search" => {
                        let pattern = message.payload["pattern"].as_str().unwrap_or("");
                        self.search_in_file(file_path, pattern).await?
                    }
                    _ => json!({"status": "error", "message": "Unknown operation"})
                };
                
                Ok(AgentMessage {
                    id: Uuid::new_v4(),
                    sender: "editor".to_string(),
                    recipient: message.sender.clone(),
                    message_type: "file_operation_result".to_string(),
                    payload: result,
                    timestamp: chrono::Utc::now(),
                })
            }
            _ => {
                Ok(AgentMessage {
                    id: Uuid::new_v4(),
                    sender: "editor".to_string(),
                    recipient: message.sender.clone(),
                    message_type: "unknown_message".to_string(),
                    payload: json!({"error": "Unknown message type"}),
                    timestamp: chrono::Utc::now(),
                })
            }
        }
    }
    
    async fn process_task(&mut self, task: Task) -> Result<TaskStatus> {
        if let Some((operation, file_path, content)) = self.extract_file_operation(&task.description) {
            let result = match operation.as_str() {
                "create" => self.create_file(&file_path, &content).await,
                "read" => self.read_file(&file_path).await,
                "update" => self.update_file(&file_path, &content).await,
                "delete" => self.delete_file(&file_path).await,
                "search" => self.search_in_file(&file_path, &content).await,
                _ => return Ok(TaskStatus::Failed),
            };
            
            match result {
                Ok(_) => Ok(TaskStatus::Completed),
                Err(_) => Ok(TaskStatus::Failed),
            }
        } else {
            Ok(TaskStatus::Failed)
        }
    }
    
    async fn make_decision(&self, context: &HashMap<String, Value>) -> Result<Value> {
        let has_pending_operations = self.file_operations.iter()
            .any(|op| op.status == OperationStatus::Pending);
        
        Ok(json!({
            "should_act": has_pending_operations,
            "pending_operations": self.file_operations.iter().filter(|op| op.status == OperationStatus::Pending).count(),
            "current_file": self.current_file,
            "action": if has_pending_operations { "process_file_operations" } else { "wait" }
        }))
    }
    
    async fn execute_action(&mut self, action: &str, params: &HashMap<String, Value>) -> Result<Value> {
        match action {
            "process_file_operations" => {
                let mut results = Vec::new();
                
                // Process pending operations
                let pending_indices: Vec<usize> = self
                    .file_operations
                    .iter()
                    .enumerate()
                    .filter(|(_, operation)| operation.status == OperationStatus::Pending)
                    .map(|(index, _)| index)
                    .collect();

                for index in pending_indices {
                    let (operation_type, file_path, content) = {
                        let operation = &mut self.file_operations[index];
                        operation.status = OperationStatus::InProgress;
                        (
                            operation.operation_type.clone(),
                            operation.file_path.clone(),
                            operation.content.clone(),
                        )
                    };

                    let result = match operation_type {
                        FileOperationType::Create => {
                            if let Some(content) = content.as_deref() {
                                self.create_file(&file_path, content).await
                            } else {
                                Ok(json!({"status": "error", "message": "No content for create operation"}))
                            }
                        }
                        FileOperationType::Read => self.read_file(&file_path).await,
                        FileOperationType::Update => {
                            if let Some(content) = content.as_deref() {
                                self.update_file(&file_path, content).await
                            } else {
                                Ok(json!({"status": "error", "message": "No content for update operation"}))
                            }
                        }
                        FileOperationType::Delete => self.delete_file(&file_path).await,
                        FileOperationType::Search => {
                            if let Some(content) = content.as_deref() {
                                self.search_in_file(&file_path, content).await
                            } else {
                                Ok(json!({"status": "error", "message": "No pattern for search operation"}))
                            }
                        }
                    };

                    let operation = &mut self.file_operations[index];
                    match result {
                        Ok(res) => {
                            operation.status = OperationStatus::Completed;
                            results.push(res);
                        }
                        Err(_) => {
                            operation.status = OperationStatus::Failed;
                        }
                    }
                }
                
                Ok(json!({
                    "status": "success",
                    "processed_operations": results.len(),
                    "results": results
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
            "file_creation".to_string(),
            "file_reading".to_string(),
            "file_editing".to_string(),
            "file_deletion".to_string(),
            "text_search".to_string(),
            "code_generation".to_string(),
        ]
    }
}
