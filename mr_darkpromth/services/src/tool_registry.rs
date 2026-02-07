// MR.DarkPromth Tool Registry - Agent 6: MasterToolExecutor & Tool System Engineer
// Phase 2: Tool Registry Implementation

use crate::tool_system::{Tool, ToolError, ToolMetadata, ToolExecutionContext, ToolResult, ToolPlugin};
use crate::tool_plugin_manager::{PluginManager, PluginDiscovery, BuiltinFilePlugin};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ToolRegistryConfig {
    pub auto_discover: bool,
    pub enable_builtin_tools: bool,
    pub plugin_paths: Vec<String>,
    pub max_concurrent_executions: usize,
}

impl Default for ToolRegistryConfig {
    fn default() -> Self {
        Self {
            auto_discover: true,
            enable_builtin_tools: true,
            plugin_paths: vec!["./plugins".to_string(), "./tools".to_string()],
            max_concurrent_executions: 100,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ToolRegistry {
    plugin_manager: PluginManager,
    discovery: PluginDiscovery,
    config: ToolRegistryConfig,
    execution_stats: Arc<RwLock<HashMap<String, ExecutionStats>>>,
    event_sender: broadcast::Sender<RegistryEvent>,
    db_pool: Option<sqlx::PgPool>,
}

#[derive(Debug, Clone)]
pub struct ExecutionStats {
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub average_execution_time_ms: f64,
    pub last_execution: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone)]
pub enum RegistryEvent {
    ToolRegistered { tool_name: String, tool_id: Uuid },
    ToolUnregistered { tool_name: String },
    ToolExecuted { tool_name: String, success: bool, duration_ms: u64 },
    PluginLoaded { plugin_name: String },
    PluginUnloaded { plugin_name: String },
}

impl ToolRegistry {
    pub fn new(config: ToolRegistryConfig) -> Self {
        let (event_sender, _) = broadcast::channel(1000);
        
        Self {
            plugin_manager: PluginManager::new(),
            discovery: PluginDiscovery::new(),
            config,
            execution_stats: Arc::new(RwLock::new(HashMap::new())),
            event_sender,
            db_pool: None,
        }
    }

    pub fn set_db_pool(&mut self, pool: sqlx::PgPool) {
        self.db_pool = Some(pool);
    }

    pub async fn initialize(&mut self) -> Result<(), ToolError> {
        // Load built-in tools if enabled
        if self.config.enable_builtin_tools {
            let mut builtin_plugin = BuiltinFilePlugin::new();
            if let Some(pool) = &self.db_pool {
                builtin_plugin = builtin_plugin.with_db(pool.clone());
            }
            
            self.plugin_manager.register_plugin(Box::new(builtin_plugin))?;
            
            let _ = self.event_sender.send(RegistryEvent::PluginLoaded {
                plugin_name: "builtin_file_operations".to_string(),
            });
        }

        // Auto-discover plugins if enabled
        if self.config.auto_discover {
            let discovered_plugins = self.discovery.discover_plugins()?;
            for plugin_name in discovered_plugins {
                // In future phases, this will load dynamic plugins
                log::info!("Discovered plugin: {}", plugin_name);
            }
        }

        Ok(())
    }

    pub fn get_tool(&self, tool_name: &str) -> Option<Arc<dyn Tool>> {
        self.plugin_manager.get_tool(tool_name)
    }

    pub fn list_tools(&self) -> Vec<String> {
        self.plugin_manager.list_tools()
    }

    pub fn get_tool_metadata(&self, tool_name: &str) -> Option<ToolMetadata> {
        self.plugin_manager.get_tool_metadata(tool_name)
    }

    pub fn execute_tool(
        &self,
        tool_name: &str,
        input: serde_json::Value,
        context: ToolExecutionContext,
    ) -> Result<ToolResult, ToolError> {
        let start_time = std::time::Instant::now();
        
        // Get tool
        let tool = self.plugin_manager.get_tool(tool_name)
            .ok_or_else(|| ToolError::ToolNotFound(tool_name.to_string()))?;

        // Check if tool is enabled
        if !self.plugin_manager.is_tool_enabled(tool_name) {
            return Err(ToolError::PermissionDenied(format!("Tool {} is disabled", tool_name)));
        }

        // Validate input
        tool.validate_input(&input)?;

        // Check permissions
        let _required_permissions = tool.requires_permissions();
        // TODO: Implement proper permission checking based on agent role and user tier
        // For now, we'll assume all permissions are granted

        // Execute tool
        let result = tool.execute(input, &context);
        let execution_time = start_time.elapsed().as_millis() as u64;

        // Update statistics
        self.update_execution_stats(tool_name, &result, execution_time);

        // Send event
        let _ = self.event_sender.send(RegistryEvent::ToolExecuted {
            tool_name: tool_name.to_string(),
            success: result.is_ok(),
            duration_ms: execution_time,
        });

        result
    }

    pub fn register_plugin(&mut self, plugin: Box<dyn ToolPlugin>) -> Result<(), ToolError> {
        let plugin_name = plugin.name().to_string();
        self.plugin_manager.register_plugin(plugin)?;
        
        let _ = self.event_sender.send(RegistryEvent::PluginLoaded {
            plugin_name: plugin_name.clone(),
        });
        
        Ok(())
    }

    pub fn unregister_plugin(&mut self, plugin_name: &str) -> Result<(), ToolError> {
        self.plugin_manager.unregister_plugin(plugin_name)?;
        
        let _ = self.event_sender.send(RegistryEvent::PluginUnloaded {
            plugin_name: plugin_name.to_string(),
        });
        
        Ok(())
    }

    pub fn enable_tool(&mut self, tool_name: &str) -> Result<(), ToolError> {
        self.plugin_manager.enable_tool(tool_name)
    }

    pub fn disable_tool(&mut self, tool_name: &str) -> Result<(), ToolError> {
        self.plugin_manager.disable_tool(tool_name)
    }

    pub fn get_execution_stats(&self, tool_name: &str) -> Option<ExecutionStats> {
        self.execution_stats.read().unwrap().get(tool_name).cloned()
    }

    pub fn list_plugins(&self) -> Vec<String> {
        self.plugin_manager.list_plugins()
    }

    pub fn subscribe_to_events(&self) -> broadcast::Receiver<RegistryEvent> {
        self.event_sender.subscribe()
    }

    pub fn search_tools(&self, query: &str) -> Vec<String> {
        let tools = self.plugin_manager.list_tools();
        tools.into_iter()
            .filter(|tool_name| {
                if let Some(metadata) = self.plugin_manager.get_tool_metadata(tool_name) {
                    metadata.name.contains(query)
                        || metadata.schema.description.contains(query)
                        || metadata.schema.category.contains(query)
                } else {
                    false
                }
            })
            .collect()
    }

    pub fn get_tools_by_category(&self, category: &str) -> Vec<String> {
        let tools = self.plugin_manager.list_tools();
        tools.into_iter()
            .filter(|tool_name| {
                if let Some(metadata) = self.plugin_manager.get_tool_metadata(tool_name) {
                    metadata.schema.category == category
                } else {
                    false
                }
            })
            .collect()
    }

    fn update_execution_stats(&self, tool_name: &str, result: &Result<ToolResult, ToolError>, execution_time: u64) {
        let mut stats = self.execution_stats.write().unwrap();
        let tool_stats = stats.entry(tool_name.to_string()).or_insert(ExecutionStats {
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            average_execution_time_ms: 0.0,
            last_execution: None,
        });

        tool_stats.total_executions += 1;
        tool_stats.last_execution = Some(chrono::Utc::now());

        match result {
            Ok(_) => {
                tool_stats.successful_executions += 1;
            }
            Err(_) => {
                tool_stats.failed_executions += 1;
            }
        }

        // Update average execution time
        let total_time = tool_stats.average_execution_time_ms * (tool_stats.total_executions - 1) as f64 + execution_time as f64;
        tool_stats.average_execution_time_ms = total_time / tool_stats.total_executions as f64;
    }

    pub async fn shutdown(&mut self) -> Result<(), ToolError> {
        self.plugin_manager.shutdown()?;
        Ok(())
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new(ToolRegistryConfig::default())
    }
}

// Global registry instance for convenience
static mut GLOBAL_REGISTRY: Option<Arc<ToolRegistry>> = None;
static REGISTRY_INIT: std::sync::Once = std::sync::Once::new();

#[allow(static_mut_refs)]
pub fn get_global_registry() -> Option<Arc<ToolRegistry>> {
    unsafe {
        REGISTRY_INIT.call_once(|| {
            GLOBAL_REGISTRY = Some(Arc::new(ToolRegistry::default()));
        });
        GLOBAL_REGISTRY.clone()
    }
}

pub async fn initialize_global_registry(config: ToolRegistryConfig) -> Result<(), ToolError> {
    let mut registry = ToolRegistry::new(config);
    registry.initialize().await?;
    
    unsafe {
        GLOBAL_REGISTRY = Some(Arc::new(registry));
    }
    
    Ok(())
}
