use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ErrorCategory {
    SyntaxError,
    RuntimeError,
    LogicalError,
    NetworkError,
    DatabaseError,
    AuthenticationError,
    ConfigurationError,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetection {
    pub id: uuid::Uuid,
    pub category: ErrorCategory,
    pub severity: ErrorSeverity,
    pub message: String,
    pub source: ErrorSource,
    pub context: ErrorContext,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub stack_trace: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorSource {
    ApplicationLog,
    ToolExecution,
    AgentFeedback,
    SystemMonitor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    pub file_path: Option<String>,
    pub line_number: Option<u32>,
    pub function_name: Option<String>,
    pub request_id: Option<String>,
    pub user_id: Option<String>,
    pub additional_metadata: HashMap<String, String>,
}

#[derive(Error, Debug)]
pub enum DetectionError {
    #[error("Failed to parse error message: {0}")]
    ParseError(String),
    #[error("Failed to classify error: {0}")]
    ClassificationError(String),
    #[error("Invalid context provided: {0}")]
    InvalidContext(String),
}

pub struct ErrorDetector {
    syntax_patterns: Vec<Regex>,
    runtime_patterns: Vec<Regex>,
    logical_patterns: Vec<Regex>,
    network_patterns: Vec<Regex>,
    database_patterns: Vec<Regex>,
    auth_patterns: Vec<Regex>,
    config_patterns: Vec<Regex>,
}

impl ErrorDetector {
    pub fn new() -> Result<Self, DetectionError> {
        Ok(Self {
            syntax_patterns: vec![
                Regex::new(r"unexpected (?:token|end of input|character)").unwrap(),
                Regex::new(r"syntax error").unwrap(),
                Regex::new(r"expected .*,? found").unwrap(),
                Regex::new(r"invalid syntax").unwrap(),
                Regex::new(r"parse error").unwrap(),
            ],
            runtime_patterns: vec![
                Regex::new(r"panic:|exception:|error:").unwrap(),
                Regex::new(r"segmentation fault").unwrap(),
                Regex::new(r"null pointer").unwrap(),
                Regex::new(r"index out of bounds").unwrap(),
                Regex::new(r"division by zero").unwrap(),
                Regex::new(r"stack overflow").unwrap(),
            ],
            logical_patterns: vec![
                Regex::new(r"logic error").unwrap(),
                Regex::new(r"assertion failed").unwrap(),
                Regex::new(r"unexpected result").unwrap(),
                Regex::new(r"invariant violation").unwrap(),
            ],
            network_patterns: vec![
                Regex::new(r"connection (refused|reset|timeout)").unwrap(),
                Regex::new(r"network unreachable").unwrap(),
                Regex::new(r"dns resolution failed").unwrap(),
                Regex::new(r"timeout").unwrap(),
            ],
            database_patterns: vec![
                Regex::new(r"database error").unwrap(),
                Regex::new(r"sql.*error").unwrap(),
                Regex::new(r"connection.*failed").unwrap(),
                Regex::new(r"deadlock detected").unwrap(),
                Regex::new(r"constraint violation").unwrap(),
            ],
            auth_patterns: vec![
                Regex::new(r"authentication failed").unwrap(),
                Regex::new(r"unauthorized").unwrap(),
                Regex::new(r"access denied").unwrap(),
                Regex::new(r"token.*invalid").unwrap(),
            ],
            config_patterns: vec![
                Regex::new(r"configuration error").unwrap(),
                Regex::new(r"missing.*configuration").unwrap(),
                Regex::new(r"invalid.*setting").unwrap(),
            ],
        })
    }

    pub fn detect_from_log(&self, log_message: &str, source: &ErrorSource) -> Vec<ErrorDetection> {
        let mut detections = Vec::new();
        
        if self.contains_any_pattern(&self.syntax_patterns, log_message) {
            detections.push(self.create_detection(
                ErrorCategory::SyntaxError,
                log_message,
                source,
            ));
        }
        
        if self.contains_any_pattern(&self.runtime_patterns, log_message) {
            detections.push(self.create_detection(
                ErrorCategory::RuntimeError,
                log_message,
                source,
            ));
        }
        
        if self.contains_any_pattern(&self.logical_patterns, log_message) {
            detections.push(self.create_detection(
                ErrorCategory::LogicalError,
                log_message,
                source,
            ));
        }
        
        if self.contains_any_pattern(&self.network_patterns, log_message) {
            detections.push(self.create_detection(
                ErrorCategory::NetworkError,
                log_message,
                source,
            ));
        }
        
        if self.contains_any_pattern(&self.database_patterns, log_message) {
            detections.push(self.create_detection(
                ErrorCategory::DatabaseError,
                log_message,
                source,
            ));
        }
        
        if self.contains_any_pattern(&self.auth_patterns, log_message) {
            detections.push(self.create_detection(
                ErrorCategory::AuthenticationError,
                log_message,
                source,
            ));
        }
        
        if self.contains_any_pattern(&self.config_patterns, log_message) {
            detections.push(self.create_detection(
                ErrorCategory::ConfigurationError,
                log_message,
                source,
            ));
        }
        
        if detections.is_empty() {
            detections.push(self.create_detection(
                ErrorCategory::Unknown,
                log_message,
                source,
            ));
        }
        
        detections
    }

    pub fn detect_from_tool_result(&self, result: &str, exit_code: i32) -> Option<ErrorDetection> {
        if exit_code != 0 {
            Some(self.create_detection(
                ErrorCategory::RuntimeError,
                &format!("Tool failed with exit code {}: {}", exit_code, result),
                &ErrorSource::ToolExecution,
            ))
        } else {
            None
        }
    }

    pub fn detect_from_agent_feedback(&self, feedback: &str) -> Vec<ErrorDetection> {
        let mut detections = Vec::new();
        
        if feedback.contains("error") || feedback.contains("failed") {
            detections.push(self.create_detection(
                ErrorCategory::RuntimeError,
                feedback,
                &ErrorSource::AgentFeedback,
            ));
        }
        
        detections
    }

    fn contains_any_pattern(&self, patterns: &[Regex], text: &str) -> bool {
        patterns.iter().any(|pattern| pattern.is_match(text))
    }

    fn create_detection(&self, category: ErrorCategory, message: &str, source: &ErrorSource) -> ErrorDetection {
        let severity = self.determine_severity(&category, message);
        
        ErrorDetection {
            id: uuid::Uuid::new_v4(),
            category,
            severity,
            message: message.to_string(),
            source: source.clone(),
            context: ErrorContext {
                file_path: None,
                line_number: None,
                function_name: None,
                request_id: None,
                user_id: None,
                additional_metadata: HashMap::new(),
            },
            timestamp: chrono::Utc::now(),
            stack_trace: None,
        }
    }

    fn determine_severity(&self, category: &ErrorCategory, message: &str) -> ErrorSeverity {
        match category {
            ErrorCategory::SyntaxError => ErrorSeverity::High,
            ErrorCategory::RuntimeError => {
                if message.contains("panic") || message.contains("segmentation fault") {
                    ErrorSeverity::Critical
                } else {
                    ErrorSeverity::High
                }
            }
            ErrorCategory::LogicalError => ErrorSeverity::Medium,
            ErrorCategory::NetworkError => ErrorSeverity::Medium,
            ErrorCategory::DatabaseError => ErrorSeverity::High,
            ErrorCategory::AuthenticationError => ErrorSeverity::High,
            ErrorCategory::ConfigurationError => ErrorSeverity::Medium,
            ErrorCategory::Unknown => ErrorSeverity::Low,
        }
    }

    pub fn classify_error(&self, error_message: &str) -> ErrorCategory {
        if self.contains_any_pattern(&self.syntax_patterns, error_message) {
            ErrorCategory::SyntaxError
        } else if self.contains_any_pattern(&self.runtime_patterns, error_message) {
            ErrorCategory::RuntimeError
        } else if self.contains_any_pattern(&self.logical_patterns, error_message) {
            ErrorCategory::LogicalError
        } else if self.contains_any_pattern(&self.network_patterns, error_message) {
            ErrorCategory::NetworkError
        } else if self.contains_any_pattern(&self.database_patterns, error_message) {
            ErrorCategory::DatabaseError
        } else if self.contains_any_pattern(&self.auth_patterns, error_message) {
            ErrorCategory::AuthenticationError
        } else if self.contains_any_pattern(&self.config_patterns, error_message) {
            ErrorCategory::ConfigurationError
        } else {
            ErrorCategory::Unknown
        }
    }
}

impl Default for ErrorDetector {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syntax_error_detection() {
        let detector = ErrorDetector::new().unwrap();
        let detections = detector.detect_from_log(
            "unexpected token found at line 42",
            &ErrorSource::ApplicationLog,
        );
        
        assert!(!detections.is_empty());
        assert_eq!(detections[0].category, ErrorCategory::SyntaxError);
    }

    #[test]
    fn test_runtime_error_detection() {
        let detector = ErrorDetector::new().unwrap();
        let detections = detector.detect_from_log(
            "panic: index out of bounds",
            &ErrorSource::ApplicationLog,
        );
        
        assert!(!detections.is_empty());
        assert_eq!(detections[0].category, ErrorCategory::RuntimeError);
        assert_eq!(detections[0].severity, ErrorSeverity::Critical);
    }

    #[test]
    fn test_tool_error_detection() {
        let detector = ErrorDetector::new().unwrap();
        let detection = detector.detect_from_tool_result("Command failed", 1);
        
        assert!(detection.is_some());
        assert_eq!(detection.unwrap().category, ErrorCategory::RuntimeError);
    }

    #[test]
    fn test_no_error_detection() {
        let detector = ErrorDetector::new().unwrap();
        let detection = detector.detect_from_tool_result("Success", 0);
        
        assert!(detection.is_none());
    }
}
