// MR.DarkPromth Tool Logging System - Agent 6: MasterToolExecutor & Tool System Engineer
// Phase 4: Tool Logging Implementation

use crate::tool_system::{ToolResult, ToolError, ToolExecutionContext};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionLog {
    pub log_id: Uuid,
    pub tool_name: String,
    pub agent_id: String,
    pub user_id: String,
    pub user_tier: String,
    pub request_id: String,
    pub input_hash: String,
    pub output_hash: String,
    pub success: bool,
    pub execution_time_ms: u64,
    pub error_message: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUsageStats {
    pub tool_name: String,
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub average_execution_time_ms: f64,
    pub last_execution: Option<DateTime<Utc>>,
    pub unique_users: u64,
}

#[derive(Debug, Clone)]
pub struct LoggingConfig {
    pub log_file_path: PathBuf,
    pub max_log_file_size_mb: u64,
    pub max_log_files: u32,
    pub enable_console_logging: bool,
    pub enable_file_logging: bool,
    pub log_level: LogLevel,
    pub retention_days: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            log_file_path: PathBuf::from("./logs/tool_executions.log"),
            max_log_file_size_mb: 100,
            max_log_files: 10,
            enable_console_logging: true,
            enable_file_logging: true,
            log_level: LogLevel::Info,
            retention_days: 30,
        }
    }
}

pub struct ToolLogger {
    config: LoggingConfig,
    execution_logs: Arc<RwLock<Vec<ToolExecutionLog>>>,
    usage_stats: Arc<RwLock<HashMap<String, ToolUsageStats>>>,
    current_log_file: Arc<RwLock<Option<File>>>,
}

impl ToolLogger {
    pub fn new(config: LoggingConfig) -> Self {
        let logger = Self {
            config,
            execution_logs: Arc::new(RwLock::new(Vec::new())),
            usage_stats: Arc::new(RwLock::new(HashMap::new())),
            current_log_file: Arc::new(RwLock::new(None)),
        };

        // Initialize logging
        logger.initialize_logging();

        logger
    }

    fn initialize_logging(&self) {
        if self.config.enable_file_logging {
            if let Some(parent) = self.config.log_file_path.parent() {
                if !parent.exists() {
                    let _ = std::fs::create_dir_all(parent);
                }
            }
        }
    }

    pub fn log_execution(
        &self,
        tool_name: &str,
        context: &ToolExecutionContext,
        result: &Result<ToolResult, ToolError>,
        execution_time_ms: u64,
    ) -> Result<(), ToolError> {
        let log_entry = self.create_log_entry(tool_name, context, result, execution_time_ms);

        // Store in memory
        self.execution_logs.write().unwrap().push(log_entry.clone());

        // Update usage statistics
        self.update_usage_stats(tool_name, &log_entry);

        // Write to file if enabled
        if self.config.enable_file_logging {
            self.write_to_file(&log_entry)?;
        }

        // Log to console if enabled
        if self.config.enable_console_logging {
            self.log_to_console(&log_entry);
        }

        Ok(())
    }

    fn create_log_entry(
        &self,
        tool_name: &str,
        context: &ToolExecutionContext,
        result: &Result<ToolResult, ToolError>,
        execution_time_ms: u64,
    ) -> ToolExecutionLog {
        let (success, error_message) = match result {
            Ok(tool_result) => (tool_result.success, tool_result.error.clone()),
            Err(e) => (false, Some(e.to_string())),
        };

        let input_hash = self.compute_hash(&serde_json::json!({}));
        let output_hash = self.compute_hash(result);

        ToolExecutionLog {
            log_id: Uuid::new_v4(),
            tool_name: tool_name.to_string(),
            agent_id: context.agent_id.clone(),
            user_id: context.metadata.get("user_id").cloned().unwrap_or_default(),
            user_tier: context.user_tier.clone(),
            request_id: context.request_id.clone(),
            input_hash,
            output_hash,
            success,
            execution_time_ms,
            error_message,
            timestamp: Utc::now(),
            metadata: context.metadata.clone(),
        }
    }

    fn update_usage_stats(&self, tool_name: &str, log_entry: &ToolExecutionLog) {
        let mut stats = self.usage_stats.write().unwrap();
        let tool_stats = stats.entry(tool_name.to_string()).or_insert(ToolUsageStats {
            tool_name: tool_name.to_string(),
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            average_execution_time_ms: 0.0,
            last_execution: None,
            unique_users: 0,
        });

        tool_stats.total_executions += 1;
        if log_entry.success {
            tool_stats.successful_executions += 1;
        } else {
            tool_stats.failed_executions += 1;
        }

        tool_stats.last_execution = Some(log_entry.timestamp);

        // Update average execution time
        let total_time = tool_stats.average_execution_time_ms * (tool_stats.total_executions - 1) as f64
            + log_entry.execution_time_ms as f64;
        tool_stats.average_execution_time_ms = total_time / tool_stats.total_executions as f64;
    }

    fn write_to_file(&self, log_entry: &ToolExecutionLog) -> Result<(), ToolError> {
        let log_line = serde_json::to_string(log_entry)
            .map_err(|e| ToolError::SerializationError(e.to_string()))?;

        let mut file = self.current_log_file.write().unwrap();
        
        if file.is_none() {
            *file = Some(OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.config.log_file_path)
                .map_err(|e| ToolError::ExecutionFailed(format!("Failed to open log file: {}", e)))?);
        }

        if let Some(ref mut f) = *file {
            writeln!(f, "{}", log_line)
                .map_err(|e| ToolError::ExecutionFailed(format!("Failed to write to log file: {}", e)))?;
        }

        Ok(())
    }

    fn log_to_console(&self, log_entry: &ToolExecutionLog) {
        let level = if log_entry.success {
            LogLevel::Info
        } else {
            LogLevel::Error
        };

        let level_str = level_to_string(level);
        if level as u8 <= self.config.log_level as u8 {
            let status = if log_entry.success { "SUCCESS" } else { "FAILED" };
            println!(
                "[{}] {} - Tool: {}, Agent: {}, Time: {}ms, Status: {}",
                log_entry.timestamp.format("%Y-%m-%d %H:%M:%S"),
                level_str,
                log_entry.tool_name,
                log_entry.agent_id,
                log_entry.execution_time_ms,
                status
            );

            if let Some(ref error) = log_entry.error_message {
                println!("  Error: {}", error);
            }
        }
    }

    pub fn get_execution_logs(
        &self,
        tool_name: Option<&str>,
        limit: Option<usize>,
    ) -> Vec<ToolExecutionLog> {
        let logs = self.execution_logs.read().unwrap();
        let filtered: Vec<ToolExecutionLog> = if let Some(name) = tool_name {
            logs.iter()
                .filter(|log| log.tool_name == name)
                .cloned()
                .collect()
        } else {
            logs.clone()
        };

        let limit = limit.unwrap_or(filtered.len());
        filtered.into_iter()
            .rev()
            .take(limit)
            .collect()
    }

    pub fn get_usage_stats(&self, tool_name: Option<&str>) -> Vec<ToolUsageStats> {
        let stats = self.usage_stats.read().unwrap();
        if let Some(name) = tool_name {
            stats.get(name)
                .cloned()
                .into_iter()
                .collect()
        } else {
            stats.values().cloned().collect()
        }
    }

    pub fn get_error_logs(&self, limit: Option<usize>) -> Vec<ToolExecutionLog> {
        let logs = self.execution_logs.read().unwrap();
        let errors: Vec<ToolExecutionLog> = logs.iter()
            .filter(|log| !log.success)
            .cloned()
            .collect();

        let limit = limit.unwrap_or(errors.len());
        errors.into_iter()
            .rev()
            .take(limit)
            .collect()
    }

    pub fn cleanup_old_logs(&self) {
        let mut logs = self.execution_logs.write().unwrap();
        let cutoff = Utc::now() - chrono::Duration::days(self.config.retention_days as i64);
        logs.retain(|log| log.timestamp > cutoff);
    }

    pub fn export_logs(&self, output_path: &PathBuf) -> Result<(), ToolError> {
        let logs = self.execution_logs.read().unwrap();
        let json = serde_json::to_string_pretty(&*logs)
            .map_err(|e| ToolError::SerializationError(e.to_string()))?;

        std::fs::write(output_path, json)
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to export logs: {}", e)))?;

        Ok(())
    }

    fn compute_hash(&self, value: &impl serde::Serialize) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let json = serde_json::to_string(value).unwrap_or_default();
        let mut hasher = DefaultHasher::new();
        json.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    pub fn get_log_summary(&self) -> LogSummary {
        let logs = self.execution_logs.read().unwrap();
        let stats = self.usage_stats.read().unwrap();

        LogSummary {
            total_executions: logs.len(),
            successful_executions: logs.iter().filter(|l| l.success).count(),
            failed_executions: logs.iter().filter(|l| !l.success).count(),
            unique_tools: stats.len(),
            average_execution_time_ms: logs.iter()
                .map(|l| l.execution_time_ms)
                .sum::<u64>() as f64 / logs.len() as f64,
            last_log_time: logs.last().map(|l| l.timestamp),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSummary {
    pub total_executions: usize,
    pub successful_executions: usize,
    pub failed_executions: usize,
    pub unique_tools: usize,
    pub average_execution_time_ms: f64,
    pub last_log_time: Option<DateTime<Utc>>,
}

fn level_to_string(level: LogLevel) -> &'static str {
    match level {
        LogLevel::Debug => "DEBUG",
        LogLevel::Info => "INFO",
        LogLevel::Warning => "WARN",
        LogLevel::Error => "ERROR",
        LogLevel::Critical => "CRITICAL",
    }
}

impl Default for ToolLogger {
    fn default() -> Self {
        Self::new(LoggingConfig::default())
    }
}
