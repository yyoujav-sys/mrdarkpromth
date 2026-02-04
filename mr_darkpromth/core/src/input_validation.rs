// MR.DarkPromth Input Validation and Sanitization System
// Phase 2: Safety and Security Implementation

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Input contains forbidden characters: {0}")]
    ForbiddenCharacters(String),
    #[error("Input exceeds maximum length: {current} > {max}")]
    InputTooLong { current: usize, max: usize },
    #[error("Input format is invalid: {0}")]
    InvalidFormat(String),
    #[error("Input contains potential XSS attack: {0}")]
    XSSAttempt(String),
    #[error("Input contains SQL injection attempt: {0}")]
    SQLInjectionAttempt(String),
    #[error("Input contains path traversal attempt: {0}")]
    PathTraversalAttempt(String),
    #[error("Input contains command injection attempt: {0}")]
    CommandInjectionAttempt(String),
    #[error("Regex compilation failed: {0}")]
    RegexError(#[from] regex::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    pub max_input_length: usize,
    pub allowed_html_tags: HashSet<String>,
    pub allowed_file_extensions: HashSet<String>,
    pub forbidden_patterns: Vec<String>,
    pub xss_patterns: Vec<String>,
    pub sql_injection_patterns: Vec<String>,
    pub path_traversal_patterns: Vec<String>,
    pub command_injection_patterns: Vec<String>,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        let mut allowed_html_tags = HashSet::new();
        allowed_html_tags.extend([
            "p", "br", "strong", "em", "u", "i", "b",
            "h1", "h2", "h3", "h4", "h5", "h6",
            "ul", "ol", "li", "blockquote", "code", "pre",
            "a", "img", "div", "span",
        ]
        .iter()
        .map(|tag| tag.to_string()));

        let mut allowed_file_extensions = HashSet::new();
        allowed_file_extensions.extend([
            "txt", "md", "json", "yaml", "yml", "csv", "tsv",
            "py", "js", "html", "css", "xml", "sql", "sh", "bat",
            "jpg", "jpeg", "png", "gif", "bmp", "svg", "ico",
            "mp3", "wav", "mp4", "avi", "mov", "mkv", "flv",
            "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx",
        ]
        .iter()
        .map(|ext| ext.to_string()));

        Self {
            max_input_length: 10_000,
            allowed_html_tags,
            allowed_file_extensions,
            forbidden_patterns: vec![
                r"(?i)<script[^>]*>.*?</script>".to_string(),
                r"(?i)javascript:".to_string(),
                r"(?i)vbscript:".to_string(),
                r"(?i)onload\s*=".to_string(),
                r"(?i)onerror\s*=".to_string(),
                r"(?i)onclick\s*=".to_string(),
            ],
            xss_patterns: vec![
                r"(?i)<script[^>]*>".to_string(),
                r"(?i)javascript:".to_string(),
                r"(?i)vbscript:".to_string(),
                r"(?i)on\w+\s*=".to_string(),
                r"(?i)<iframe[^>]*>".to_string(),
                r"(?i)<object[^>]*>".to_string(),
                r"(?i)<embed[^>]*>".to_string(),
                r"(?i)<link[^>]*>".to_string(),
                r"(?i)<meta[^>]*>".to_string(),
                r"(?i)expression\s*\(".to_string(),
            ],
            sql_injection_patterns: vec![
                r"(?i)(union|select|insert|update|delete|drop|create|alter|exec|execute)\s+".to_string(),
                r"(?i)'.*or.*'.*='".to_string(),
                r#"(?i)".*or.*".*=""#.to_string(),
                r"(?i)1\s*=\s*1".to_string(),
                r"(?i)1\s*=\s*1\s*--".to_string(),
                r"(?i)\bxp_cmdshell\b".to_string(),
                r"(?i)sp_executesql".to_string(),
                r"(?i)waitfor\s+delay".to_string(),
            ],
            path_traversal_patterns: vec![
                r"(?i)\.\./".to_string(),
                r"(?i)\.\.\\/".to_string(),
                r"(?i)%2e%2e%2f".to_string(),
                r"(?i)%2e%2e%5c".to_string(),
                r"(?i)\.\.\\\\".to_string(),
                r"(?i)/etc/passwd".to_string(),
                r"(?i)/etc/shadow".to_string(),
                r"(?i)C:\\Windows".to_string(),
            ],
            command_injection_patterns: vec![
                r"(?i);\s*rm\s+".to_string(),
                r"(?i);\s*dd\s+".to_string(),
                r"(?i);\s*cat\s+".to_string(),
                r"(?i);\s*ls\s+".to_string(),
                r"(?i);\s*ps\s+".to_string(),
                r"(?i);\s*netstat".to_string(),
                r"(?i);\s*wget\s+".to_string(),
                r"(?i);\s*curl\s+".to_string(),
                r"(?i);\s*nc\s+".to_string(),
                r"(?i);\s*nmap\s+".to_string(),
                r"(?i)`[^`]*`".to_string(),
                r"(?i)\$\(.*\)".to_string(),
                r"(?i)\|\s*sh".to_string(),
                r"(?i)\|\s*bash".to_string(),
                r"(?i)>\s*/dev/".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone)]
pub struct InputValidator {
    config: ValidationConfig,
    forbidden_regexes: Vec<Regex>,
    xss_regexes: Vec<Regex>,
    sql_injection_regexes: Vec<Regex>,
    path_traversal_regexes: Vec<Regex>,
    command_injection_regexes: Vec<Regex>,
}

impl InputValidator {
    pub fn new(config: ValidationConfig) -> Result<Self, ValidationError> {
        let mut validator = Self {
            forbidden_regexes: Vec::new(),
            xss_regexes: Vec::new(),
            sql_injection_regexes: Vec::new(),
            path_traversal_regexes: Vec::new(),
            command_injection_regexes: Vec::new(),
            config,
        };

        validator.compile_regexes()?;
        Ok(validator)
    }

    fn compile_regexes(&mut self) -> Result<(), ValidationError> {
        // Compile forbidden patterns
        for pattern in &self.config.forbidden_patterns {
            self.forbidden_regexes.push(Regex::new(pattern)?);
        }

        // Compile XSS patterns
        for pattern in &self.config.xss_patterns {
            self.xss_regexes.push(Regex::new(pattern)?);
        }

        // Compile SQL injection patterns
        for pattern in &self.config.sql_injection_patterns {
            self.sql_injection_regexes.push(Regex::new(pattern)?);
        }

        // Compile path traversal patterns
        for pattern in &self.config.path_traversal_patterns {
            self.path_traversal_regexes.push(Regex::new(pattern)?);
        }

        // Compile command injection patterns
        for pattern in &self.config.command_injection_patterns {
            self.command_injection_regexes.push(Regex::new(pattern)?);
        }

        Ok(())
    }

    pub fn validate_string(&self, input: &str) -> Result<String, ValidationError> {
        // Check length
        if input.len() > self.config.max_input_length {
            return Err(ValidationError::InputTooLong {
                current: input.len(),
                max: self.config.max_input_length,
            });
        }

        // Check for forbidden patterns
        for regex in &self.forbidden_regexes {
            if regex.is_match(input) {
                return Err(ValidationError::ForbiddenCharacters(
                    "Forbidden pattern detected".to_string()
                ));
            }
        }

        // Check for XSS attempts
        for regex in &self.xss_regexes {
            if regex.is_match(input) {
                return Err(ValidationError::XSSAttempt(
                    "XSS attempt detected".to_string()
                ));
            }
        }

        // Check for SQL injection attempts
        for regex in &self.sql_injection_regexes {
            if regex.is_match(input) {
                return Err(ValidationError::SQLInjectionAttempt(
                    "SQL injection attempt detected".to_string()
                ));
            }
        }

        // Check for path traversal attempts
        for regex in &self.path_traversal_regexes {
            if regex.is_match(input) {
                return Err(ValidationError::PathTraversalAttempt(
                    "Path traversal attempt detected".to_string()
                ));
            }
        }

        // Check for command injection attempts
        for regex in &self.command_injection_regexes {
            if regex.is_match(input) {
                return Err(ValidationError::CommandInjectionAttempt(
                    "Command injection attempt detected".to_string()
                ));
            }
        }

        Ok(input.to_string())
    }

    pub fn sanitize_html(&self, html: &str) -> Result<String, ValidationError> {
        // Basic HTML sanitization - remove dangerous tags and attributes
        let mut sanitized = html.to_string();

        // Remove script tags
        let script_regex = Regex::new(r"(?i)<script[^>]*>.*?</script>").unwrap();
        sanitized = script_regex.replace_all(&sanitized, "").to_string();

        // Remove dangerous event handlers
        let event_regex = Regex::new(r#"(?i)on\w+\s*=\s*["'][^"']*["']"#).unwrap();
        sanitized = event_regex.replace_all(&sanitized, "").to_string();

        // Remove javascript: URLs
        let js_regex = Regex::new(r"(?i)javascript:").unwrap();
        sanitized = js_regex.replace_all(&sanitized, "").to_string();

        Ok(sanitized)
    }

    pub fn validate_filename(&self, filename: &str) -> Result<String, ValidationError> {
        // Check for path traversal in filename
        if filename.contains("..") || filename.contains("/") || filename.contains("\\") {
            return Err(ValidationError::PathTraversalAttempt(
                "Path traversal in filename".to_string()
            ));
        }

        // Check file extension
        if let Some(extension) = std::path::Path::new(filename)
            .extension()
            .and_then(|ext| ext.to_str()) {
            if !self.config.allowed_file_extensions.contains(extension) {
                return Err(ValidationError::InvalidFormat(
                    format!("File extension '{}' is not allowed", extension)
                ));
            }
        } else {
            return Err(ValidationError::InvalidFormat(
                "Filename must have a valid extension".to_string()
            ));
        }

        // Validate the filename itself
        self.validate_string(filename)?;

        Ok(filename.to_string())
    }

    pub fn validate_email(&self, email: &str) -> Result<String, ValidationError> {
        let email_regex = Regex::new(
            r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$"
        ).unwrap();

        if !email_regex.is_match(email) {
            return Err(ValidationError::InvalidFormat(
                "Invalid email format".to_string()
            ));
        }

        self.validate_string(email)?;
        Ok(email.to_string())
    }

    pub fn validate_url(&self, url: &str) -> Result<String, ValidationError> {
        let url_regex = Regex::new(
            r"^https?://[a-zA-Z0-9.-]+(?:\.[a-zA-Z]{2,})?(?:/[^\s]*)?$"
        ).unwrap();

        if !url_regex.is_match(url) {
            return Err(ValidationError::InvalidFormat(
                "Invalid URL format".to_string()
            ));
        }

        self.validate_string(url)?;
        Ok(url.to_string())
    }

    pub fn validate_json(&self, json_str: &str) -> Result<serde_json::Value, ValidationError> {
        // First validate the string itself
        self.validate_string(json_str)?;

        // Then try to parse as JSON
        serde_json::from_str(json_str).map_err(|_| {
            ValidationError::InvalidFormat("Invalid JSON format".to_string())
        })
    }

    pub fn get_validation_stats(&self, input: &str) -> ValidationStats {
        let mut forbidden_matches = 0;
        let mut xss_matches = 0;
        let mut sql_injection_matches = 0;
        let mut path_traversal_matches = 0;
        let mut command_injection_matches = 0;

        for regex in &self.forbidden_regexes {
            forbidden_matches += regex.find_iter(input).count();
        }

        for regex in &self.xss_regexes {
            xss_matches += regex.find_iter(input).count();
        }

        for regex in &self.sql_injection_regexes {
            sql_injection_matches += regex.find_iter(input).count();
        }

        for regex in &self.path_traversal_regexes {
            path_traversal_matches += regex.find_iter(input).count();
        }

        for regex in &self.command_injection_regexes {
            command_injection_matches += regex.find_iter(input).count();
        }

        ValidationStats {
            input_length: input.len(),
            forbidden_matches,
            xss_matches,
            sql_injection_matches,
            path_traversal_matches,
            command_injection_matches,
            is_safe: forbidden_matches == 0 && xss_matches == 0 && 
                    sql_injection_matches == 0 && path_traversal_matches == 0 && 
                    command_injection_matches == 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationStats {
    pub input_length: usize,
    pub forbidden_matches: usize,
    pub xss_matches: usize,
    pub sql_injection_matches: usize,
    pub path_traversal_matches: usize,
    pub command_injection_matches: usize,
    pub is_safe: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xss_detection() {
        let config = ValidationConfig::default();
        let validator = InputValidator::new(config).unwrap();

        // Use a more specific XSS pattern that matches the regex
        let xss_input = "<script src='evil.js'>";
        let result = validator.validate_string(xss_input);
        assert!(result.is_err(), "Should detect XSS in: {}", xss_input);
        assert!(matches!(result.unwrap_err(), ValidationError::XSSAttempt(_)));

        // Also test with javascript: protocol
        let js_input = "javascript:alert('xss')";
        let result2 = validator.validate_string(js_input);
        assert!(result2.is_err(), "Should detect javascript: protocol");
    }

    #[test]
    fn test_sql_injection_detection() {
        let config = ValidationConfig::default();
        let validator = InputValidator::new(config).unwrap();

        let sql_input = "SELECT * FROM users WHERE '1'='1'";
        let result = validator.validate_string(sql_input);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ValidationError::SQLInjectionAttempt(_)));
    }

    #[test]
    fn test_path_traversal_detection() {
        let config = ValidationConfig::default();
        let validator = InputValidator::new(config).unwrap();

        let path_input = "../../../etc/passwd";
        let result = validator.validate_string(path_input);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ValidationError::PathTraversalAttempt(_)));
    }

    #[test]
    fn test_command_injection_detection() {
        let config = ValidationConfig::default();
        let validator = InputValidator::new(config).unwrap();

        let cmd_input = "ls; rm -rf /";
        let result = validator.validate_string(cmd_input);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ValidationError::CommandInjectionAttempt(_)));
    }

    #[test]
    fn test_safe_input_validation() {
        let config = ValidationConfig::default();
        let validator = InputValidator::new(config).unwrap();

        let safe_input = "Hello, World! This is a safe input.";
        let result = validator.validate_string(safe_input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_email_validation() {
        let config = ValidationConfig::default();
        let validator = InputValidator::new(config).unwrap();

        assert!(validator.validate_email("test@example.com").is_ok());
        assert!(validator.validate_email("invalid-email").is_err());
    }

    #[test]
    fn test_filename_validation() {
        let config = ValidationConfig::default();
        let validator = InputValidator::new(config).unwrap();

        assert!(validator.validate_filename("document.txt").is_ok());
        assert!(validator.validate_filename("../../../etc/passwd").is_err());
        assert!(validator.validate_filename("malicious.exe").is_err());
    }

    #[test]
    fn test_html_sanitization() {
        let config = ValidationConfig::default();
        let validator = InputValidator::new(config).unwrap();

        let html_input = "<p>Safe content</p><script>alert('xss')</script>";
        let sanitized = validator.sanitize_html(html_input).unwrap();
        
        assert!(sanitized.contains("<p>Safe content</p>"));
        assert!(!sanitized.contains("<script>"));
    }

    #[test]
    fn test_validation_stats() {
        let config = ValidationConfig::default();
        let validator = InputValidator::new(config).unwrap();

        let input = "This is <script>alert('xss')</script> and SELECT * FROM users";
        let stats = validator.get_validation_stats(input);
        
        assert!(!stats.is_safe);
        assert!(stats.xss_matches > 0);
        assert!(stats.sql_injection_matches > 0);
    }
}
