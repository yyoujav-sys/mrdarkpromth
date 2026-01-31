// MR.DarkPromth Tool Plugin Manager - Agent 6: MasterToolExecutor & Tool System Engineer
// Phase 1: Plugin Architecture Design

use crate::tool_system::{Tool, ToolPlugin, ToolError, ToolMetadata};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PluginConfig {
    pub name: String,
    pub version: String,
    pub enabled: bool,
    pub auto_load: bool,
    pub dependencies: Vec<String>,
}

#[derive(Clone)]
pub struct PluginManager {
    plugins: Arc<RwLock<HashMap<String, Box<dyn ToolPlugin>>>>,
    tools: Arc<RwLock<HashMap<String, Arc<dyn Tool>>>>,
    tool_metadata: Arc<RwLock<HashMap<String, ToolMetadata>>>,
    plugin_configs: Arc<RwLock<HashMap<String, PluginConfig>>>,
}

impl std::fmt::Debug for PluginManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginManager")
            .field("plugins_count", &self.plugins.read().unwrap().len())
            .field("tools_count", &self.tools.read().unwrap().len())
            .field("plugin_configs", &self.plugin_configs)
            .finish()
    }
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            tools: Arc::new(RwLock::new(HashMap::new())),
            tool_metadata: Arc::new(RwLock::new(HashMap::new())),
            plugin_configs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn register_plugin(&mut self, mut plugin: Box<dyn ToolPlugin>) -> Result<(), ToolError> {
        let plugin_name = plugin.name().to_string();
        
        // Initialize plugin
        plugin.initialize()?;
        
        // Register plugin tools
        let tools = plugin.tools();
        for tool in tools {
            let tool_name = tool.name().to_string();
            let schema = tool.schema();
            let tool_arc = Arc::from(tool);
            
            let metadata = ToolMetadata {
                id: Uuid::new_v4(),
                name: tool_name.clone(),
                version: plugin.version().to_string(),
                schema,
                enabled: true,
            };
            
            self.tools.write().unwrap().insert(tool_name.clone(), tool_arc);
            self.tool_metadata.write().unwrap().insert(tool_name, metadata);
        }
        
        self.plugins.write().unwrap().insert(plugin_name, plugin);
        Ok(())
    }

    pub fn unregister_plugin(&mut self, plugin_name: &str) -> Result<(), ToolError> {
        let mut plugins = self.plugins.write().unwrap();
        let mut tools = self.tools.write().unwrap();
        let mut metadata = self.tool_metadata.write().unwrap();
        
        if let Some(mut plugin) = plugins.remove(plugin_name) {
            // Remove all tools from this plugin
            let tool_names: Vec<String> = tools.keys()
                .filter(|&name| {
                    metadata.get(name)
                        .map(|m| m.version == plugin.version())
                        .unwrap_or(false)
                })
                .cloned()
                .collect();
            
            for tool_name in tool_names {
                tools.remove(&tool_name);
                metadata.remove(&tool_name);
            }
            
            plugin.shutdown()?;
        }
        
        Ok(())
    }

    pub fn get_tool(&self, tool_name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.read().unwrap().get(tool_name).cloned()
    }

    pub fn list_tools(&self) -> Vec<String> {
        self.tools.read().unwrap().keys().cloned().collect()
    }

    pub fn get_tool_metadata(&self, tool_name: &str) -> Option<ToolMetadata> {
        self.tool_metadata.read().unwrap().get(tool_name).cloned()
    }

    pub fn list_plugins(&self) -> Vec<String> {
        self.plugins.read().unwrap().keys().cloned().collect()
    }

    pub fn enable_tool(&mut self, tool_name: &str) -> Result<(), ToolError> {
        let mut metadata = self.tool_metadata.write().unwrap();
        if let Some(tool_meta) = metadata.get_mut(tool_name) {
            tool_meta.enabled = true;
            Ok(())
        } else {
            Err(ToolError::ToolNotFound(tool_name.to_string()))
        }
    }

    pub fn disable_tool(&mut self, tool_name: &str) -> Result<(), ToolError> {
        let mut metadata = self.tool_metadata.write().unwrap();
        if let Some(tool_meta) = metadata.get_mut(tool_name) {
            tool_meta.enabled = false;
            Ok(())
        } else {
            Err(ToolError::ToolNotFound(tool_name.to_string()))
        }
    }

    pub fn is_tool_enabled(&self, tool_name: &str) -> bool {
        self.tool_metadata.read().unwrap()
            .get(tool_name)
            .map(|m| m.enabled)
            .unwrap_or(false)
    }

    pub fn shutdown(&mut self) -> Result<(), ToolError> {
        let mut plugins = self.plugins.write().unwrap();
        for (_, plugin) in plugins.iter_mut() {
            plugin.shutdown()?;
        }
        plugins.clear();
        
        self.tools.write().unwrap().clear();
        self.tool_metadata.write().unwrap().clear();
        
        Ok(())
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

// Built-in plugin for basic file operations
#[derive(Debug)]
pub struct BuiltinFilePlugin {
    initialized: bool,
    db: Option<sqlx::PgPool>,
}

impl BuiltinFilePlugin {
    pub fn new() -> Self {
        Self {
            initialized: false,
            db: None,
        }
    }

    pub fn with_db(mut self, pool: sqlx::PgPool) -> Self {
        self.db = Some(pool);
        self
    }
}

impl ToolPlugin for BuiltinFilePlugin {
    fn name(&self) -> &str {
        "builtin_file_operations"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn tools(&self) -> Vec<Box<dyn Tool>> {
        vec![
            Box::new(crate::tool_system::builtin::FileReadTool),
            Box::new(crate::tool_system::builtin::FileWriteTool),
            Box::new(crate::tool_system::builtin::FileListTool),
            Box::new(crate::tool_system::builtin::WebScrapeTool),
            Box::new(crate::tool_system::builtin::CodeExecuteTool),
            Box::new(crate::tool_system::builtin::DatabaseQueryTool { db: self.db.clone() }),
        ]
    }

    fn initialize(&mut self) -> Result<(), ToolError> {
        if self.initialized {
            return Ok(());
        }
        
        // Initialize file system access, check permissions, etc.
        self.initialized = true;
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), ToolError> {
        if !self.initialized {
            return Ok(());
        }
        
        // Cleanup resources
        self.initialized = false;
        Ok(())
    }
}

// Plugin discovery system for future dynamic loading
#[derive(Debug, Clone)]
pub struct PluginDiscovery {
    plugin_paths: Vec<String>,
}

impl PluginDiscovery {
    pub fn new() -> Self {
        Self {
            plugin_paths: vec![
                "./plugins".to_string(),
                "./tools".to_string(),
            ],
        }
    }

    pub fn add_plugin_path(&mut self, path: String) {
        self.plugin_paths.push(path);
    }

    pub fn discover_plugins(&self) -> Result<Vec<String>, ToolError> {
        let mut discovered = Vec::new();
        
        // For now, return built-in plugins
        // In future phases, this will scan directories for dynamic libraries
        discovered.push("builtin_file_operations".to_string());
        
        Ok(discovered)
    }
}

impl Default for PluginDiscovery {
    fn default() -> Self {
        Self::new()
    }
}
