// MR.DarkPromth Self-Correction Engine
// Agent 8: Self-Correction Engine Engineer
// Phase 1 Implementation - Error Detection and Classification

use crate::redis_coordination::{RedisCoordinator, CoordinationEvent, EventType};
use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;
use thiserror::Error;
use uuid::Uuid;
use log::info;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEvent {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub source: ErrorSource,
    pub error_type: ErrorType,
    pub severity: ErrorSeverity,
    pub message: String,
    pub file_path: Option<PathBuf>,
    pub line_number: Option<u32>,
    pub stack_trace: Option<String>,
    pub context: Value,
    pub agent_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorSource {
    ApplicationLog,
    ToolExecution,
    AgentFeedback,
    SystemMonitor,
    RedisEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorType {
    SyntaxError,
    RuntimeError,
    LogicalError,
    CompilationError,
    NetworkError,
    DatabaseError,
    ConfigurationError,
    SecurityError,
    PerformanceError,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPattern {
    pub id: String,
    pub name: String,
    pub pattern: String,
    pub error_type: ErrorType,
    pub severity: ErrorSeverity,
    pub description: String,
}

pub struct ErrorDetector {
    patterns: Vec<ErrorPattern>,
    redis_coordinator: Option<RedisCoordinator>,
    agent_id: String,
}

impl ErrorDetector {
    pub fn new(agent_id: String) -> Self {
        let mut detector = Self {
            patterns: Vec::new(),
            redis_coordinator: None,
            agent_id,
        };
        
        detector.initialize_default_patterns();
        detector
    }

    pub fn with_redis(mut self, redis_coordinator: RedisCoordinator) -> Self {
        self.redis_coordinator = Some(redis_coordinator);
        self
    }

    fn initialize_default_patterns(&mut self) {
        // Rust compilation errors
        self.add_pattern(ErrorPattern {
            id: "rust_compilation_error".to_string(),
            name: "Rust Compilation Error".to_string(),
            pattern: r"error\[E\d+\]:.*".to_string(),
            error_type: ErrorType::CompilationError,
            severity: ErrorSeverity::High,
            description: "Rust compiler error".to_string(),
        });

        // Python syntax errors
        self.add_pattern(ErrorPattern {
            id: "python_syntax_error".to_string(),
            name: "Python Syntax Error".to_string(),
            pattern: r"SyntaxError:.*".to_string(),
            error_type: ErrorType::SyntaxError,
            severity: ErrorSeverity::High,
            description: "Python syntax error".to_string(),
        });

        // Runtime exceptions
        self.add_pattern(ErrorPattern {
            id: "runtime_exception".to_string(),
            name: "Runtime Exception".to_string(),
            pattern: r"(Exception|Error|panic!|assertion failed):.*".to_string(),
            error_type: ErrorType::RuntimeError,
            severity: ErrorSeverity::High,
            description: "Runtime exception or panic".to_string(),
        });

        // Network errors
        self.add_pattern(ErrorPattern {
            id: "network_error".to_string(),
            name: "Network Error".to_string(),
            pattern: r"(Connection refused|Timeout|Network unreachable|DNS resolution failed).*".to_string(),
            error_type: ErrorType::NetworkError,
            severity: ErrorSeverity::Medium,
            description: "Network connectivity error".to_string(),
        });

        // Database errors
        self.add_pattern(ErrorPattern {
            id: "database_error".to_string(),
            name: "Database Error".to_string(),
            pattern: r"(SQL error|Database connection failed|Connection pool exhausted).*".to_string(),
            error_type: ErrorType::DatabaseError,
            severity: ErrorSeverity::High,
            description: "Database operation error".to_string(),
        });

        // Redis errors
        self.add_pattern(ErrorPattern {
            id: "redis_error".to_string(),
            name: "Redis Error".to_string(),
            pattern: r"(Redis connection failed|MOVED|ASK|NOAUTH).*".to_string(),
            error_type: ErrorType::NetworkError,
            severity: ErrorSeverity::Medium,
            description: "Redis operation error".to_string(),
        });

        // Security errors
        self.add_pattern(ErrorPattern {
            id: "security_error".to_string(),
            name: "Security Error".to_string(),
            pattern: r"(Authentication failed|Access denied|Unauthorized|Security violation).*".to_string(),
            error_type: ErrorType::SecurityError,
            severity: ErrorSeverity::Critical,
            description: "Security or authentication error".to_string(),
        });

        // Performance issues
        self.add_pattern(ErrorPattern {
            id: "performance_error".to_string(),
            name: "Performance Issue".to_string(),
            pattern: r"(Timeout|Slow query|High memory usage|Out of memory).*".to_string(),
            error_type: ErrorType::PerformanceError,
            severity: ErrorSeverity::Medium,
            description: "Performance degradation".to_string(),
        });
    }

    pub fn add_pattern(&mut self, pattern: ErrorPattern) {
        self.patterns.push(pattern);
    }

    pub fn detect_from_log(&self, log_line: &str, source: ErrorSource) -> Result<Vec<ErrorEvent>> {
        let mut errors = Vec::new();
        
        for pattern in &self.patterns {
            if let Ok(regex) = Regex::new(&pattern.pattern) {
                if let Some(captures) = regex.captures(log_line) {
                    let error_event = ErrorEvent {
                        id: Uuid::new_v4().to_string(),
                        timestamp: Utc::now(),
                        source: source.clone(),
                        error_type: pattern.error_type.clone(),
                        severity: pattern.severity.clone(),
                        message: captures.get(0).unwrap().as_str().to_string(),
                        file_path: self.extract_file_path(log_line),
                        line_number: self.extract_line_number(log_line),
                        stack_trace: self.extract_stack_trace(log_line),
                        context: json!({
                            "pattern_id": pattern.id,
                            "pattern_name": pattern.name,
                            "original_log": log_line,
                            "captures": captures.iter().filter_map(|m| m.map(|mat| mat.as_str())).collect::<Vec<_>>()
                        }),
                        agent_id: Some(self.agent_id.clone()),
                    };
                    
                    errors.push(error_event);
                }
            }
        }
        
        Ok(errors)
    }

    pub fn detect_from_tool_result(&self, tool_name: &str, result: &str, exit_code: i32) -> Result<Option<ErrorEvent>> {
        if exit_code == 0 {
            return Ok(None);
        }

        let error_type = self.classify_tool_error(tool_name, result);
        let severity = self.determine_severity_from_exit_code(exit_code);

        let error_event = ErrorEvent {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            source: ErrorSource::ToolExecution,
            error_type,
            severity,
            message: format!("Tool '{}' failed with exit code {}: {}", tool_name, exit_code, result),
            file_path: None,
            line_number: None,
            stack_trace: Some(result.to_string()),
            context: json!({
                "tool_name": tool_name,
                "exit_code": exit_code,
                "result": result
            }),
            agent_id: Some(self.agent_id.clone()),
        };

        Ok(Some(error_event))
    }

    pub fn detect_from_agent_feedback(&self, agent_id: &str, feedback: &str) -> Result<Vec<ErrorEvent>> {
        let mut errors = Vec::new();
        
        // Look for error indicators in agent feedback
        let error_indicators = vec![
            "error", "failed", "exception", "panic", "crash", "timeout",
            "unable to", "cannot", "invalid", "malformed", "corrupted"
        ];
        
        for indicator in error_indicators {
            if feedback.to_lowercase().contains(indicator) {
                let error_event = ErrorEvent {
                    id: Uuid::new_v4().to_string(),
                    timestamp: Utc::now(),
                    source: ErrorSource::AgentFeedback,
                    error_type: ErrorType::RuntimeError,
                    severity: ErrorSeverity::Medium,
                    message: format!("Agent {} reported issue: {}", agent_id, feedback),
                    file_path: None,
                    line_number: None,
                    stack_trace: None,
                    context: json!({
                        "agent_id": agent_id,
                        "feedback": feedback,
                        "indicator": indicator
                    }),
                    agent_id: Some(self.agent_id.clone()),
                };
                
                errors.push(error_event);
            }
        }
        
        Ok(errors)
    }

    pub async fn process_redis_error_event(&mut self, event: &CoordinationEvent) -> Result<()> {
        if event.event_type != EventType::ErrorEvent {
            return Ok(());
        }

        let error_event = ErrorEvent {
            id: event.event_id.clone(),
            timestamp: event.timestamp,
            source: ErrorSource::RedisEvent,
            error_type: self.extract_error_type_from_payload(&event.payload),
            severity: self.extract_severity_from_payload(&event.payload),
            message: self.extract_message_from_payload(&event.payload),
            file_path: event.payload.get("file").and_then(|f| f.as_str()).map(|s| PathBuf::from(s)),
            line_number: event.payload.get("line").and_then(|l| l.as_u64()).map(|l| l as u32),
            stack_trace: None,
            context: event.payload.clone(),
            agent_id: Some(event.agent_id.clone()),
        };

        self.handle_detected_error(error_event).await
    }

    async fn handle_detected_error(&mut self, error: ErrorEvent) -> Result<()> {
        info!("Detected error: {:?} - {}", error.error_type, error.message);
        
        // Store error for analysis
        // TODO: Implement error storage in Phase 4
        
        // Publish to Redis if available
        if let Some(ref mut coordinator) = self.redis_coordinator {
            let _ = coordinator.publish_error_event(
                &format!("{:?}", error.error_type),
                &error.message,
                error.file_path.as_ref().map(|p| p.to_str().unwrap()),
                error.line_number
            );
        }
        
        // Trigger classification and fix generation
        // TODO: Implement in Phase 2
        
        Ok(())
    }

    fn classify_tool_error(&self, tool_name: &str, result: &str) -> ErrorType {
        match tool_name {
            "cargo" | "rustc" => ErrorType::CompilationError,
            "python" | "pip" => ErrorType::RuntimeError,
            "redis-cli" => ErrorType::NetworkError,
            "mysql" | "psql" => ErrorType::DatabaseError,
            _ if result.contains("syntax") => ErrorType::SyntaxError,
            _ if result.contains("timeout") => ErrorType::PerformanceError,
            _ if result.contains("permission") || result.contains("access") => ErrorType::SecurityError,
            _ => ErrorType::RuntimeError,
        }
    }

    fn determine_severity_from_exit_code(&self, exit_code: i32) -> ErrorSeverity {
        match exit_code {
            1 => ErrorSeverity::Low,
            2..=5 => ErrorSeverity::Medium,
            6..=10 => ErrorSeverity::High,
            _ => ErrorSeverity::Critical,
        }
    }

    fn extract_file_path(&self, log_line: &str) -> Option<PathBuf> {
        let re = Regex::new(r"--> ([^:]+):\d+").unwrap();
        re.captures(log_line)
            .and_then(|c| c.get(1))
            .map(|m| PathBuf::from(m.as_str()))
    }

    fn extract_line_number(&self, log_line: &str) -> Option<u32> {
        let re = Regex::new(r":(\d+):").unwrap();
        re.captures(log_line)
            .and_then(|c| c.get(1))
            .and_then(|m| m.as_str().parse().ok())
    }

    fn extract_stack_trace(&self, log_line: &str) -> Option<String> {
        if log_line.contains("stack backtrace:") || log_line.contains("Traceback") {
            Some(log_line.to_string())
        } else {
            None
        }
    }

    fn extract_error_type_from_payload(&self, payload: &Value) -> ErrorType {
        payload.get("error_type")
            .and_then(|t| t.as_str())
            .and_then(|t| match t {
                "syntax" => Some(ErrorType::SyntaxError),
                "runtime" => Some(ErrorType::RuntimeError),
                "logical" => Some(ErrorType::LogicalError),
                "compilation" => Some(ErrorType::CompilationError),
                "network" => Some(ErrorType::NetworkError),
                "database" => Some(ErrorType::DatabaseError),
                "configuration" => Some(ErrorType::ConfigurationError),
                "security" => Some(ErrorType::SecurityError),
                "performance" => Some(ErrorType::PerformanceError),
                _ => None,
            })
            .unwrap_or(ErrorType::Unknown)
    }

    fn extract_severity_from_payload(&self, payload: &Value) -> ErrorSeverity {
        payload.get("severity")
            .and_then(|s| s.as_str())
            .and_then(|s| match s {
                "low" => Some(ErrorSeverity::Low),
                "medium" => Some(ErrorSeverity::Medium),
                "high" => Some(ErrorSeverity::High),
                "critical" => Some(ErrorSeverity::Critical),
                _ => None,
            })
            .unwrap_or(ErrorSeverity::Medium)
    }

    fn extract_message_from_payload(&self, payload: &Value) -> String {
        payload.get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("Unknown error")
            .to_string()
    }
}

#[derive(Error, Debug)]
pub enum SelfCorrectionError {
    #[error("Pattern compilation error: {0}")]
    PatternError(#[from] regex::Error),
    
    #[error("Redis coordination error: {0}")]
    RedisError(String),
    
    #[error("JSON serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_error_detection() {
        let detector = ErrorDetector::new("test_agent".to_string());
        let log_line = "error[E0277]: cannot multiply f64 by f32";
        let errors = detector.detect_from_log(log_line, ErrorSource::ApplicationLog).unwrap();
        
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0].error_type, ErrorType::CompilationError));
        assert_eq!(errors[0].severity, ErrorSeverity::High);
    }

    #[test]
    fn test_tool_error_detection() {
        let detector = ErrorDetector::new("test_agent".to_string());
        let result = "error: cannot find Cargo.toml";
        let error = detector.detect_from_tool_result("cargo", result, 101).unwrap();
        
        assert!(error.is_some());
        assert!(matches!(error.unwrap().error_type, ErrorType::CompilationError));
    }

    #[test]
    fn test_agent_feedback_detection() {
        let detector = ErrorDetector::new("test_agent".to_string());
        let feedback = "Agent 3 failed to process request due to timeout";
        let errors = detector.detect_from_agent_feedback("agent3", feedback).unwrap();
        
        // The feedback may match multiple error indicators
        assert!(errors.len() >= 1, "Should detect at least one error");
        assert!(matches!(errors[0].error_type, ErrorType::RuntimeError));
    }
}
