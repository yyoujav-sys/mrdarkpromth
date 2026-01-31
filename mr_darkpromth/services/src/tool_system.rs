// MR.DarkPromth Tool System - Agent 6: MasterToolExecutor & Tool System Engineer
// Phase 1: Tool Framework Design

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use thiserror::Error;
use uuid::Uuid;
use reqwest::blocking::{Client, RequestBuilder};
use reqwest::header::{HeaderMap, HeaderValue};

#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum ToolError {
    #[error("Tool not found: {0}")]
    ToolNotFound(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

impl From<serde_json::Error> for ToolError {
    fn from(error: serde_json::Error) -> Self {
        ToolError::SerializationError(error.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub output_schema: serde_json::Value,
    pub required_permissions: Vec<String>,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionContext {
    pub agent_id: String,
    pub user_tier: String,
    pub request_id: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
    pub execution_time_ms: u64,
    pub tool_name: String,
}

pub trait Tool: Send + Sync + std::fmt::Debug {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn schema(&self) -> ToolSchema;
    fn execute(&self, input: serde_json::Value, context: &ToolExecutionContext) -> Result<ToolResult, ToolError>;
    fn validate_input(&self, input: &serde_json::Value) -> Result<(), ToolError>;
    fn requires_permissions(&self) -> Vec<String>;
}

#[derive(Debug, Clone)]
pub struct ToolMetadata {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub schema: ToolSchema,
    pub enabled: bool,
}

pub trait ToolPlugin: Send + Sync + std::fmt::Debug {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn tools(&self) -> Vec<Box<dyn Tool>>;
    fn initialize(&mut self) -> Result<(), ToolError>;
    fn shutdown(&mut self) -> Result<(), ToolError>;
}

use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct ToolSystem {
    registry: Arc<crate::tool_registry::ToolRegistry>,
}

impl ToolSystem {
    pub fn new() -> Self {
        let config = crate::tool_registry::ToolRegistryConfig::default();
        let registry = Arc::new(crate::tool_registry::ToolRegistry::new(config));
        Self { registry }
    }

    pub async fn execute_tool(
        &self,
        tool_name: &str,
        input: serde_json::Value,
    ) -> Result<serde_json::Value, ToolError> {
        let registry = Arc::clone(&self.registry);
        let tool_name = tool_name.to_string();
        
        // Execute synchronous tool registry in async context
        tokio::task::spawn_blocking(move || {
            let context = ToolExecutionContext {
                agent_id: "system".to_string(),
                user_tier: "free".to_string(),
                request_id: Uuid::new_v4().to_string(),
                metadata: HashMap::new(),
            };
            let result = registry.execute_tool(&tool_name, input, context)?;
            match result.data {
                Some(data) => Ok(data),
                None => Ok(serde_json::json!({
                    "success": result.success,
                    "error": result.error,
                    "execution_time_ms": result.execution_time_ms
                }))
            }
        }).await.map_err(|e| ToolError::ExecutionFailed(format!("Task join error: {}", e)))?
    }
}

impl Default for ToolSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRegistrationEvent {
    pub tool_id: Uuid,
    pub tool_name: String,
    pub agent_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub event_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionEvent {
    pub tool_name: String,
    pub agent_id: String,
    pub request_id: String,
    pub input_hash: String,
    pub success: bool,
    pub execution_time_ms: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

// Built-in tool implementations will be added in Phase 3
pub mod builtin {
    use super::*;
    use crate::sandboxed_execution::{ExecutionResult as SandboxExecutionResult, SandboxConfig, SandboxedExecutor};

    #[derive(Debug)]
    pub struct FileReadTool;
    #[derive(Debug)]
    pub struct FileWriteTool;
    #[derive(Debug)]
    pub struct FileListTool;
    
    // Placeholder implementations - will be completed in Phase 3
    impl Tool for FileReadTool {
        fn name(&self) -> &str { "file_read" }
        fn description(&self) -> &str { "Read file contents" }
        fn schema(&self) -> ToolSchema {
            ToolSchema {
                name: "file_read".to_string(),
                description: "Read file contents".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string"}
                    },
                    "required": ["path"]
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "content": {"type": "string"}
                    }
                }),
                required_permissions: vec!["file.read".to_string()],
                category: "file_operations".to_string(),
            }
        }
        
        fn execute(&self, input: serde_json::Value, _context: &ToolExecutionContext) -> Result<ToolResult, ToolError> {
            let path = input.get("path")
                .and_then(|p| p.as_str())
                .ok_or_else(|| ToolError::InvalidInput("Missing or invalid 'path' parameter".to_string()))?;

            // Security check - prevent path traversal
            if path.contains("..") || path.starts_with('/') {
                return Err(ToolError::PermissionDenied("Access to absolute paths or parent directories not allowed".to_string()));
            }

            match std::fs::read_to_string(path) {
                Ok(content) => Ok(ToolResult {
                    success: true,
                    data: Some(serde_json::json!({
                        "content": content,
                        "path": path
                    })),
                    error: None,
                    execution_time_ms: 0,
                    tool_name: "file_read".to_string(),
                }),
                Err(e) => Ok(ToolResult {
                    success: false,
                    data: None,
                    error: Some(format!("Failed to read file: {}", e)),
                    execution_time_ms: 0,
                    tool_name: "file_read".to_string(),
                }),
            }
        }
        
        fn validate_input(&self, input: &serde_json::Value) -> Result<(), ToolError> {
            if !input.is_object() || !input.get("path").and_then(|p| p.as_str()).is_some() {
                return Err(ToolError::InvalidInput("Missing 'path' parameter".to_string()));
            }
            Ok(())
        }
        
        fn requires_permissions(&self) -> Vec<String> {
            vec!["file.read".to_string()]
        }
    }

    impl Tool for FileWriteTool {
        fn name(&self) -> &str { "file_write" }
        fn description(&self) -> &str { "Write content to file" }
        fn schema(&self) -> ToolSchema {
            ToolSchema {
                name: "file_write".to_string(),
                description: "Write content to file".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string"},
                        "content": {"type": "string"}
                    },
                    "required": ["path", "content"]
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "success": {"type": "boolean"}
                    }
                }),
                required_permissions: vec!["file.write".to_string()],
                category: "file_operations".to_string(),
            }
        }
        
        fn execute(&self, input: serde_json::Value, _context: &ToolExecutionContext) -> Result<ToolResult, ToolError> {
            let path = input.get("path")
                .and_then(|p| p.as_str())
                .ok_or_else(|| ToolError::InvalidInput("Missing or invalid 'path' parameter".to_string()))?;
            
            let content = input.get("content")
                .and_then(|c| c.as_str())
                .ok_or_else(|| ToolError::InvalidInput("Missing or invalid 'content' parameter".to_string()))?;

            // Security check - prevent path traversal
            if path.contains("..") || path.starts_with('/') {
                return Err(ToolError::PermissionDenied("Access to absolute paths or parent directories not allowed".to_string()));
            }

            match std::fs::write(path, content) {
                Ok(_) => Ok(ToolResult {
                    success: true,
                    data: Some(serde_json::json!({
                        "success": true,
                        "path": path,
                        "bytes_written": content.len()
                    })),
                    error: None,
                    execution_time_ms: 0,
                    tool_name: "file_write".to_string(),
                }),
                Err(e) => Ok(ToolResult {
                    success: false,
                    data: None,
                    error: Some(format!("Failed to write file: {}", e)),
                    execution_time_ms: 0,
                    tool_name: "file_write".to_string(),
                }),
            }
        }
        
        fn validate_input(&self, input: &serde_json::Value) -> Result<(), ToolError> {
            if !input.is_object() || 
               !input.get("path").and_then(|p| p.as_str()).is_some() ||
               !input.get("content").and_then(|c| c.as_str()).is_some() {
                return Err(ToolError::InvalidInput("Missing 'path' or 'content' parameter".to_string()));
            }
            Ok(())
        }
        
        fn requires_permissions(&self) -> Vec<String> {
            vec!["file.write".to_string()]
        }
    }

    impl Tool for FileListTool {
        fn name(&self) -> &str { "file_list" }
        fn description(&self) -> &str { "List directory contents" }
        fn schema(&self) -> ToolSchema {
            ToolSchema {
                name: "file_list".to_string(),
                description: "List directory contents".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string"}
                    },
                    "required": ["path"]
                }),
                output_schema: serde_json::json!({
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "name": {"type": "string"},
                            "type": {"type": "string"},
                            "size": {"type": "integer"}
                        }
                    }
                }),
                required_permissions: vec!["file.read".to_string()],
                category: "file_operations".to_string(),
            }
        }
        
        fn execute(&self, input: serde_json::Value, _context: &ToolExecutionContext) -> Result<ToolResult, ToolError> {
            let path = input.get("path")
                .and_then(|p| p.as_str())
                .ok_or_else(|| ToolError::InvalidInput("Missing or invalid 'path' parameter".to_string()))?;

            // Security check - prevent path traversal
            if path.contains("..") || path.starts_with('/') {
                return Err(ToolError::PermissionDenied("Access to absolute paths or parent directories not allowed".to_string()));
            }

            match std::fs::read_dir(path) {
                Ok(entries) => {
                    let mut files = Vec::new();
                    for entry in entries {
                        if let Ok(entry) = entry {
                            let metadata = entry.metadata();
                            let file_info = serde_json::json!({
                                "name": entry.file_name().to_string_lossy(),
                                "type": metadata.as_ref().map(|m| if m.is_dir() { "dir" } else { "file" }).unwrap_or("unknown"),
                                "size": metadata.as_ref().map(|m| m.len()).unwrap_or(0)
                            });
                            files.push(file_info);
                        }
                    }
                    
                    Ok(ToolResult {
                        success: true,
                        data: Some(serde_json::json!({
                            "files": files,
                            "path": path
                        })),
                        error: None,
                        execution_time_ms: 0,
                        tool_name: "file_list".to_string(),
                    })
                }
                Err(e) => Ok(ToolResult {
                    success: false,
                    data: None,
                    error: Some(format!("Failed to list directory: {}", e)),
                    execution_time_ms: 0,
                    tool_name: "file_list".to_string(),
                }),
            }
        }
        
        fn validate_input(&self, input: &serde_json::Value) -> Result<(), ToolError> {
            if !input.is_object() || !input.get("path").and_then(|p| p.as_str()).is_some() {
                return Err(ToolError::InvalidInput("Missing 'path' parameter".to_string()));
            }
            Ok(())
        }
        
        fn requires_permissions(&self) -> Vec<String> {
            vec!["file.read".to_string()]
        }
    }

    // Web Scraping Tool
    #[derive(Debug)]
    pub struct WebScrapeTool;

    impl Tool for WebScrapeTool {
        fn name(&self) -> &str { "web_scrape" }
        fn description(&self) -> &str { "Make HTTP requests with full support for all methods, headers, body, and authentication" }
        fn schema(&self) -> ToolSchema {
            ToolSchema {
                name: "web_scrape".to_string(),
                description: "Make HTTP requests with full support for all methods, headers, body, and authentication".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "url": {"type": "string"},
                        "method": {
                            "type": "string",
                            "enum": ["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"],
                            "default": "GET"
                        },
                        "headers": {"type": "object"},
                        "body": {"type": ["string", "object"]},
                        "query_params": {"type": "object"},
                        "auth": {
                            "type": "object",
                            "properties": {
                                "type": {"type": "string", "enum": ["bearer", "basic"]},
                                "token": {"type": "string"},
                                "username": {"type": "string"},
                                "password": {"type": "string"}
                            }
                        },
                        "timeout": {"type": "integer", "default": 10}
                    },
                    "required": ["url"]
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "content": {"type": "string"},
                        "status_code": {"type": "integer"},
                        "headers": {"type": "object"}
                    }
                }),
                required_permissions: vec!["network.read".to_string()],
                category: "network".to_string(),
            }
        }

        fn execute(&self, input: serde_json::Value, _context: &ToolExecutionContext) -> Result<ToolResult, ToolError> {
            let url = input.get("url")
                .and_then(|u| u.as_str())
                .ok_or_else(|| ToolError::InvalidInput("Missing or invalid 'url' parameter".to_string()))?;

            let method = input.get("method")
                .and_then(|m| m.as_str())
                .unwrap_or("GET");

            let timeout_secs = input.get("timeout")
                .and_then(|t| t.as_u64())
                .unwrap_or(10);

            // Build request with selected method
            let client = Client::new();
            let mut request_builder = match method.to_uppercase().as_str() {
                "GET" => client.get(url),
                "POST" => client.post(url),
                "PUT" => client.put(url),
                "DELETE" => client.delete(url),
                "PATCH" => client.patch(url),
                "HEAD" => client.head(url),
                "OPTIONS" => client.request(reqwest::Method::OPTIONS, url),
                _ => client.get(url), // Default to GET
            };

            // Add query parameters
            if let Some(query_params) = input.get("query_params").and_then(|q| q.as_object()) {
                let mut query_pairs = Vec::new();
                for (key, value) in query_params {
                    if let Some(v) = value.as_str() {
                        query_pairs.push((key.clone(), v.to_string()));
                    }
                }
                request_builder = request_builder.query(&query_pairs);
            }

            // Add headers
            if let Some(headers) = input.get("headers").and_then(|h| h.as_object()) {
                let mut header_map = HeaderMap::new();
                for (key, value) in headers {
                    if let Some(v) = value.as_str() {
                        if let Ok(header_value) = HeaderValue::from_str(v) {
                            if let Ok(header_name) = reqwest::header::HeaderName::from_bytes(key.as_bytes()) {
                                header_map.insert(header_name, header_value);
                            }
                        }
                    }
                }
                request_builder = request_builder.headers(header_map);
            }

            // Add authentication
            if let Some(auth) = input.get("auth").and_then(|a| a.as_object()) {
                if let Some(auth_type) = auth.get("type").and_then(|t| t.as_str()) {
                    match auth_type {
                        "bearer" => {
                            if let Some(token) = auth.get("token").and_then(|t| t.as_str()) {
                                request_builder = request_builder.bearer_auth(token);
                            }
                        }
                        "basic" => {
                            let username = auth.get("username").and_then(|u| u.as_str()).unwrap_or("");
                            let password = auth.get("password").and_then(|p| p.as_str()).unwrap_or("");
                            request_builder = request_builder.basic_auth(username, Some(password));
                        }
                        _ => {}
                    }
                }
            }

            // Add body for methods that support it
            if matches!(method.to_uppercase().as_str(), "POST" | "PUT" | "PATCH") {
                if let Some(body) = input.get("body") {
                    if let Some(body_str) = body.as_str() {
                        request_builder = request_builder.body(body_str.to_string());
                    } else if body.is_object() {
                        let body_json = serde_json::to_string(body)
                            .map_err(|e| ToolError::SerializationError(e.to_string()))?;
                        request_builder = request_builder.body(body_json);
                    }
                }
            }

            // Send request with timeout
            let response = request_builder
                .timeout(std::time::Duration::from_secs(timeout_secs))
                .send()
                .map_err(|e| ToolError::ExecutionFailed(format!("Failed to send request: {}", e)))?;

            let status = response.status();
            let status_code = status.as_u16();
            
            // Get response headers before consuming response
            let response_headers: HashMap<String, String> = response.headers()
                .iter()
                .filter_map(|(name, value)| {
                    value.to_str().ok().map(|v| (name.to_string(), v.to_string()))
                })
                .collect();
            
            // Get response content
            let content = if status_code >= 200 && status_code < 300 {
                response.text()
                    .map_err(|e| ToolError::ExecutionFailed(format!("Failed to read response: {}", e)))?
            } else {
                String::new()
            };

            Ok(ToolResult {
                success: status.is_success(),
                data: Some(serde_json::json!({
                    "content": content,
                    "status_code": status_code,
                    "headers": response_headers,
                    "url": url,
                    "method": method,
                    "content_length": content.len()
                })),
                error: if status.is_success() { None } else { Some(format!("HTTP error: {}", status)) },
                execution_time_ms: 0,
                tool_name: "web_scrape".to_string(),
            })
        }

        fn validate_input(&self, input: &serde_json::Value) -> Result<(), ToolError> {
            if !input.is_object() || !input.get("url").and_then(|u| u.as_str()).is_some() {
                return Err(ToolError::InvalidInput("Missing 'url' parameter".to_string()));
            }
            Ok(())
        }

        fn requires_permissions(&self) -> Vec<String> {
            vec!["network.read".to_string()]
        }
    }

    // Code Execution Tool
    #[derive(Debug)]
    pub struct CodeExecuteTool;

    impl Tool for CodeExecuteTool {
        fn name(&self) -> &str { "code_execute" }
        fn description(&self) -> &str { "Execute code in a sandboxed environment" }
        fn schema(&self) -> ToolSchema {
            ToolSchema {
                name: "code_execute".to_string(),
                description: "Execute code in a sandboxed environment".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "code": {"type": "string"},
                        "language": {"type": "string"}
                    },
                    "required": ["code", "language"]
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "stdout": {"type": "string"},
                        "stderr": {"type": "string"},
                        "exit_code": {"type": "integer"}
                    }
                }),
                required_permissions: vec!["code.execute".to_string()],
                category: "execution".to_string(),
            }
        }

        fn execute(&self, input: serde_json::Value, _context: &ToolExecutionContext) -> Result<ToolResult, ToolError> {
            let code = input.get("code")
                .and_then(|c| c.as_str())
                .ok_or_else(|| ToolError::InvalidInput("Missing or invalid 'code' parameter".to_string()))?;

            let language = input.get("language")
                .and_then(|l| l.as_str())
                .ok_or_else(|| ToolError::InvalidInput("Missing or invalid 'language' parameter".to_string()))?;

            // Security check - prevent dangerous operations
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
            ];

            for pattern in &dangerous_patterns {
                if code.contains(pattern) {
                    return Err(ToolError::PermissionDenied(
                        format!("Potentially dangerous code detected: {}", pattern)
                    ));
                }
            }

            let executor = SandboxedExecutor::new(SandboxConfig::default())
                .map_err(|e| ToolError::ExecutionFailed(format!("Sandbox init failed: {}", e)))?;

            let execution_result: SandboxExecutionResult = match tokio::runtime::Handle::try_current() {
                Ok(handle) => handle
                    .block_on(executor.execute_code(code, language))
                    .map_err(|e| ToolError::ExecutionFailed(format!("Sandbox execution failed: {}", e)))?,
                Err(_) => {
                    let runtime = tokio::runtime::Runtime::new()
                        .map_err(|e| ToolError::ExecutionFailed(format!("Sandbox runtime failed: {}", e)))?;
                    runtime
                        .block_on(executor.execute_code(code, language))
                        .map_err(|e| ToolError::ExecutionFailed(format!("Sandbox execution failed: {}", e)))?
                }
            };

            Ok(ToolResult {
                success: execution_result.exit_code == 0,
                data: Some(serde_json::json!({
                    "stdout": execution_result.stdout,
                    "stderr": execution_result.stderr,
                    "exit_code": execution_result.exit_code,
                    "language": language,
                    "security_violations": execution_result.security_violations,
                    "memory_used": execution_result.memory_used,
                    "cpu_time_ms": execution_result.cpu_time.as_millis()
                })),
                error: if execution_result.exit_code != 0 {
                    Some(format!("Execution failed with exit code: {}", execution_result.exit_code))
                } else {
                    None
                },
                execution_time_ms: execution_result.execution_time.as_millis() as u64,
                tool_name: "code_execute".to_string(),
            })
        }

        fn validate_input(&self, input: &serde_json::Value) -> Result<(), ToolError> {
            if !input.is_object() || 
               !input.get("code").and_then(|c| c.as_str()).is_some() ||
               !input.get("language").and_then(|l| l.as_str()).is_some() {
                return Err(ToolError::InvalidInput("Missing 'code' or 'language' parameter".to_string()));
            }
            Ok(())
        }

        fn requires_permissions(&self) -> Vec<String> {
            vec!["code.execute".to_string()]
        }
    }

    // Database Query Tool
    #[derive(Debug)]
    pub struct DatabaseQueryTool;

    impl Tool for DatabaseQueryTool {
        fn name(&self) -> &str { "database_query" }
        fn description(&self) -> &str { "Execute SQL queries against the database" }
        fn schema(&self) -> ToolSchema {
            ToolSchema {
                name: "database_query".to_string(),
                description: "Execute SQL queries against the database".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": {"type": "string"},
                        "params": {"type": "array"}
                    },
                    "required": ["query"]
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "rows": {"type": "array"},
                        "row_count": {"type": "integer"}
                    }
                }),
                required_permissions: vec!["database.read".to_string()],
                category: "database".to_string(),
            }
        }

        fn execute(&self, input: serde_json::Value, context: &ToolExecutionContext) -> Result<ToolResult, ToolError> {
            let query = input.get("query")
                .and_then(|q| q.as_str())
                .ok_or_else(|| ToolError::InvalidInput("Missing or invalid 'query' parameter".to_string()))?;

            // Security check - prevent SQL injection and dangerous queries
            let dangerous_keywords = vec![
                "DROP TABLE",
                "DELETE FROM",
                "TRUNCATE",
                "ALTER TABLE",
                "GRANT",
                "REVOKE",
                "CREATE USER",
                "DROP USER",
            ];

            let query_upper = query.to_uppercase();
            for keyword in &dangerous_keywords {
                if query_upper.contains(keyword) {
                    return Err(ToolError::PermissionDenied(
                        format!("Dangerous SQL keyword detected: {}", keyword)
                    ));
                }
            }

            // For demonstration, simulate database query results
            // In production, this would connect to the actual database
            let rows = if query_upper.contains("SELECT") {
                vec![
                    serde_json::json!({"id": 1, "name": "Example 1", "value": 100}),
                    serde_json::json!({"id": 2, "name": "Example 2", "value": 200}),
                ]
            } else {
                vec![]
            };

            Ok(ToolResult {
                success: true,
                data: Some(serde_json::json!({
                    "rows": rows,
                    "row_count": rows.len(),
                    "query": query,
                    "executed_by": context.agent_id
                })),
                error: None,
                execution_time_ms: 0,
                tool_name: "database_query".to_string(),
            })
        }

        fn validate_input(&self, input: &serde_json::Value) -> Result<(), ToolError> {
            if !input.is_object() || !input.get("query").and_then(|q| q.as_str()).is_some() {
                return Err(ToolError::InvalidInput("Missing 'query' parameter".to_string()));
            }
            Ok(())
        }

        fn requires_permissions(&self) -> Vec<String> {
            vec!["database.read".to_string()]
        }
    }

}
