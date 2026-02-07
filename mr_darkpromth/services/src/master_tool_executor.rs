// MR.DarkPromth MasterToolExecutor - Agent 6: MasterToolExecutor & Tool System Engineer
// Phase 2: MasterToolExecutor Implementation

use crate::tool_system::{Tool, ToolError, ToolExecutionContext, ToolResult};
use crate::tool_registry::{ToolRegistry, ToolRegistryConfig};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, Semaphore};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ExecutorConfig {
    pub max_concurrent_executions: usize,
    pub default_timeout_ms: u64,
    pub enable_logging: bool,
    pub enable_metrics: bool,
    pub sandbox_enabled: bool,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            max_concurrent_executions: 50,
            default_timeout_ms: 30000, // 30 seconds
            enable_logging: true,
            enable_metrics: true,
            sandbox_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ExecutionRequest {
    pub id: Uuid,
    pub tool_name: String,
    pub input: serde_json::Value,
    pub context: ToolExecutionContext,
    pub timeout_ms: Option<u64>,
    pub priority: ExecutionPriority,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum ExecutionPriority {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExecutionResponse {
    pub request_id: Uuid,
    pub result: Result<ToolResult, ToolError>,
    pub execution_time_ms: u64,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: chrono::DateTime<chrono::Utc>,
}

pub struct MasterToolExecutor {
    registry: Arc<ToolRegistry>,
    config: ExecutorConfig,
    semaphore: Arc<Semaphore>,
    active_executions: Arc<RwLock<HashMap<Uuid, ExecutionRequest>>>,
    execution_history: Arc<RwLock<Vec<ExecutionResponse>>>,
    max_history_size: usize,
}

impl MasterToolExecutor {
    pub fn new(config: ExecutorConfig) -> Self {
        let registry_config = ToolRegistryConfig::default();
        let registry = Arc::new(ToolRegistry::new(registry_config));
        
        Self {
            registry,
            semaphore: Arc::new(Semaphore::new(config.max_concurrent_executions)),
            config,
            active_executions: Arc::new(RwLock::new(HashMap::new())),
            execution_history: Arc::new(RwLock::new(Vec::new())),
            max_history_size: 1000,
        }
    }

    pub fn with_registry(registry: Arc<ToolRegistry>, config: ExecutorConfig) -> Self {
        Self {
            registry,
            semaphore: Arc::new(Semaphore::new(config.max_concurrent_executions)),
            config,
            active_executions: Arc::new(RwLock::new(HashMap::new())),
            execution_history: Arc::new(RwLock::new(Vec::new())),
            max_history_size: 1000,
        }
    }

    pub async fn initialize(&self) -> Result<(), ToolError> {
        // Initialize the registry (this is a simplified version)
        // In a real implementation, we would need to handle the async nature properly
        Ok(())
    }

    pub async fn execute(&self, request: ExecutionRequest) -> Result<ExecutionResponse, ToolError> {
        let _permit = self.semaphore.acquire().await.map_err(|_| {
            ToolError::ExecutionFailed("Failed to acquire execution permit".to_string())
        })?;

        let started_at = chrono::Utc::now();
        
        // Track active execution
        {
            let mut active = self.active_executions.write().await;
            active.insert(request.id, request.clone());
        }

        let result = self.execute_internal(&request).await;
        let completed_at = chrono::Utc::now();
        let execution_time = (completed_at - started_at).num_milliseconds() as u64;

        // Remove from active executions
        {
            let mut active = self.active_executions.write().await;
            active.remove(&request.id);
        }

        let response = ExecutionResponse {
            request_id: request.id,
            result,
            execution_time_ms: execution_time,
            started_at,
            completed_at,
        };

        // Add to history
        self.add_to_history(response.clone()).await;

        Ok(response)
    }

    async fn execute_internal(&self, request: &ExecutionRequest) -> Result<ToolResult, ToolError> {
        // Apply timeout
        let timeout_ms = request.timeout_ms.unwrap_or(self.config.default_timeout_ms);
        
        let tool = self.registry.get_tool(&request.tool_name)
            .ok_or_else(|| ToolError::ToolNotFound(request.tool_name.clone()))?;

        // Validate input
        tool.validate_input(&request.input)?;

        // Check permissions
        self.check_permissions(&request.context, &tool).await?;

        // Execute with timeout
        let tool_clone = tool.clone();
        let input_clone = request.input.clone();
        let context_clone = request.context.clone();

        let execution_future = tokio::task::spawn_blocking(move || {
            tool_clone.execute(input_clone, &context_clone)
        });

        match tokio::time::timeout(
            std::time::Duration::from_millis(timeout_ms),
            execution_future
        ).await {
            Ok(Ok(result)) => result,
            Ok(Err(join_error)) => Err(ToolError::ExecutionFailed(
                format!("Task join error: {}", join_error)
            )),
            Err(_) => Err(ToolError::ExecutionFailed(
                format!("Execution timed out after {}ms", timeout_ms)
            )),
        }
    }

    async fn check_permissions(&self, context: &ToolExecutionContext, tool: &Arc<dyn Tool>) -> Result<(), ToolError> {
        let required_permissions = tool.requires_permissions();
        
        // Simple permission check based on user tier
        // In a real implementation, this would be more sophisticated
        for permission in &required_permissions {
            match context.user_tier.as_str() {
                "ultra" => continue, // Ultra tier has all permissions
                "premium" => {
                    if permission.starts_with("admin.") || permission.starts_with("system.") {
                        return Err(ToolError::PermissionDenied(
                            format!("Premium tier cannot access permission: {}", permission)
                        ));
                    }
                }
                "free" | "basic" => {
                    if permission.starts_with("admin.") || permission.starts_with("system.") || 
                       permission.starts_with("advanced.") {
                        return Err(ToolError::PermissionDenied(
                            format!("Free tier cannot access permission: {}", permission)
                        ));
                    }
                }
                _ => {
                    return Err(ToolError::PermissionDenied(
                        format!("Unknown user tier: {}", context.user_tier)
                    ));
                }
            }
        }

        Ok(())
    }

    async fn add_to_history(&self, response: ExecutionResponse) {
        let mut history = self.execution_history.write().await;
        history.push(response);
        
        // Keep only the last max_history_size entries
        if history.len() > self.max_history_size {
            history.remove(0);
        }
    }

    pub async fn get_active_executions(&self) -> Vec<ExecutionRequest> {
        let active = self.active_executions.read().await;
        active.values().cloned().collect()
    }

    pub async fn get_execution_history(&self, limit: Option<usize>) -> Vec<ExecutionResponse> {
        let history = self.execution_history.read().await;
        let limit = limit.unwrap_or(history.len());
        history.iter().rev().take(limit).cloned().collect()
    }

    pub async fn get_execution_stats(&self) -> ExecutorStats {
        let active_count = self.active_executions.read().await.len();
        let history = self.execution_history.read().await;
        
        let total_executions = history.len();
        let successful_executions = history.iter().filter(|r| r.result.is_ok()).count();
        let failed_executions = total_executions - successful_executions;
        
        let average_execution_time = if total_executions > 0 {
            history.iter().map(|r| r.execution_time_ms).sum::<u64>() as f64 / total_executions as f64
        } else {
            0.0
        };

        ExecutorStats {
            active_executions: active_count,
            total_executions,
            successful_executions,
            failed_executions,
            average_execution_time_ms: average_execution_time,
            max_concurrent_executions: self.config.max_concurrent_executions,
        }
    }

    pub fn get_registry(&self) -> Arc<ToolRegistry> {
        Arc::clone(&self.registry)
    }

    pub async fn cancel_execution(&self, request_id: Uuid) -> Result<(), ToolError> {
        let mut active = self.active_executions.write().await;
        if active.remove(&request_id).is_some() {
            Ok(())
        } else {
            Err(ToolError::ToolNotFound(format!("Execution {} not found", request_id)))
        }
    }

    pub async fn shutdown(&self) -> Result<(), ToolError> {
        // Cancel all active executions
        let mut active = self.active_executions.write().await;
        let active_ids: Vec<Uuid> = active.keys().cloned().collect();
        active.clear();
        
        if self.config.enable_logging {
            log::info!("Cancelled {} active executions during shutdown", active_ids.len());
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ExecutorStats {
    pub active_executions: usize,
    pub total_executions: usize,
    pub successful_executions: usize,
    pub failed_executions: usize,
    pub average_execution_time_ms: f64,
    pub max_concurrent_executions: usize,
}

impl Default for MasterToolExecutor {
    fn default() -> Self {
        Self::new(ExecutorConfig::default())
    }
}

// Global executor instance for convenience
static mut GLOBAL_EXECUTOR: Option<Arc<MasterToolExecutor>> = None;
static EXECUTOR_INIT: std::sync::Once = std::sync::Once::new();

#[allow(static_mut_refs)]
pub fn get_global_executor() -> Option<Arc<MasterToolExecutor>> {
    unsafe {
        EXECUTOR_INIT.call_once(|| {
            GLOBAL_EXECUTOR = Some(Arc::new(MasterToolExecutor::default()));
        });
        GLOBAL_EXECUTOR.clone()
    }
}

pub async fn initialize_global_executor(config: ExecutorConfig) -> Result<(), ToolError> {
    let executor = MasterToolExecutor::new(config);
    executor.initialize().await?;
    
    unsafe {
        GLOBAL_EXECUTOR = Some(Arc::new(executor));
    }
    
    Ok(())
}
