use serde::{Deserialize, Serialize};
use regex::Regex;
use std::collections::HashSet;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Input contains malicious content: {0}")]
    MaliciousContent(String),
    #[error("Input format is invalid: {0}")]
    InvalidFormat(String),
    #[error("Input exceeds maximum length: {current} > {max}")]
    InputTooLong { current: usize, max: usize },
    #[error("Input contains forbidden characters: {0}")]
    ForbiddenCharacters(String),
    #[error("Regex compilation failed: {0}")]
    RegexError(#[from] regex::Error),
    #[error("URL validation failed: {0}")]
    InvalidUrl(String),
    #[error("Email validation failed: {0}")]
    InvalidEmail(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    pub max_input_length: usize,
    pub allowed_characters: Option<String>,
    pub forbidden_patterns: Vec<String>,
    pub required_patterns: Vec<String>,
    pub sanitize_html: bool,
    pub validate_urls: bool,
    pub validate_emails: bool,
    pub allowed_domains: Vec<String>,
    pub strict_mode: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            max_input_length: 100_000,
            allowed_characters: None,
            forbidden_patterns: vec![
                r"(?i)<script[^>]*>.*?</script>".to_string(),
                r"(?i)javascript:".to_string(),
                r"(?i)vbscript:".to_string(),
                r"(?i)onload\s*=".to_string(),
                r"(?i)onerror\s*=".to_string(),
                r"(?i)onclick\s*=".to_string(),
                r"(?i)onmouseover\s*=".to_string(),
                r"(?i)eval\s*\(".to_string(),
                r"(?i)document\s*\.".to_string(),
                r"(?i)window\s*\.".to_string(),
            ],
            required_patterns: vec![],
            sanitize_html: true,
            validate_urls: true,
            validate_emails: true,
            allowed_domains: vec![
                "github.com".to_string(),
                "stackoverflow.com".to_string(),
                "docs.rs".to_string(),
                "crates.io".to_string(),
            ],
            strict_mode: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub sanitized_input: String,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub detected_threats: Vec<String>,
    pub metadata: ValidationMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationMetadata {
    pub original_length: usize,
    pub sanitized_length: usize,
    pub patterns_matched: Vec<String>,
    pub urls_found: Vec<String>,
    pub emails_found: Vec<String>,
    pub processing_time_ms: u64,
}

#[derive(Debug, Clone)]
pub struct InputValidator {
    config: ValidationConfig,
    forbidden_regexes: Vec<Regex>,
    required_regexes: Vec<Regex>,
    url_regex: Regex,
    email_regex: Regex,
    html_tag_regex: Regex,
    script_regex: Regex,
}

impl InputValidator {
    pub fn new(config: ValidationConfig) -> Result<Self, ValidationError> {
        let mut forbidden_regexes = Vec::new();
        for pattern in &config.forbidden_patterns {
            forbidden_regexes.push(Regex::new(pattern)?);
        }

        let mut required_regexes = Vec::new();
        for pattern in &config.required_patterns {
            required_regexes.push(Regex::new(pattern)?);
        }

        Ok(Self {
            forbidden_regexes,
            required_regexes,
            url_regex: Regex::new(r"https?://[^\s<>"{}|\\^`\[\]]+")?,
            email_regex: Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b")?,
            html_tag_regex: Regex::new(r"<[^>]*>")?,
            script_regex: Regex::new(r"(?i)<script[^>]*>.*?</script>")?,
            config,
        })
    }

    pub fn validate_and_sanitize(&self, input: &str) -> Result<ValidationResult, ValidationError> {
        let start_time = std::time::Instant::now();
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut detected_threats = Vec::new();
        let mut sanitized_input = input.to_string();

        // Check input length
        if input.len() > self.config.max_input_length {
            return Err(ValidationError::InputTooLong {
                current: input.len(),
                max: self.config.max_input_length,
            });
        }

        // Check allowed characters
        if let Some(allowed_chars) = &self.config.allowed_characters {
            let invalid_chars: Vec<char> = input.chars()
                .filter(|c| !allowed_chars.contains(*c))
                .collect();
            
            if !invalid_chars.is_empty() {
                let invalid_str: String = invalid_chars.into_iter().collect();
                errors.push(format!("Forbidden characters detected: {}", invalid_str));
                if self.config.strict_mode {
                    return Err(ValidationError::ForbiddenCharacters(invalid_str));
                }
            }
        }

        // Check forbidden patterns
        for regex in &self.forbidden_regexes {
            if regex.is_match(input) {
                let matches: Vec<_> = regex.find_iter(input).collect();
                for m in matches {
                    detected_threats.push(m.as_str().to_string());
                    if self.config.strict_mode {
                        errors.push(format!("Malicious pattern detected: {}", m.as_str()));
                    }
                }
            }
        }

        // Check required patterns
        for regex in &self.required_regexes {
            if !regex.is_match(input) {
                warnings.push("Required pattern not found".to_string());
                if self.config.strict_mode {
                    errors.push("Required pattern missing".to_string());
                }
            }
        }

        // Sanitize HTML if enabled
        if self.config.sanitize_html {
            sanitized_input = self.sanitize_html(&sanitized_input);
            
            // Check for script tags specifically
            if self.script_regex.is_match(input) {
                detected_threats.push("Script tag detected".to_string());
                errors.push("Script tags are not allowed".to_string());
            }
        }

        // Validate URLs if enabled
        let mut urls_found = Vec::new();
        if self.config.validate_urls {
            for cap in self.url_regex.find_iter(&sanitized_input) {
                let url = cap.as_str().to_string();
                urls_found.push(url.clone());
                
                if let Err(e) = self.validate_url(&url) {
                    warnings.push(format!("Invalid URL: {}", e));
                    if self.config.strict_mode {
                        errors.push(format!("Invalid URL detected: {}", e));
                    }
                }
            }
        }

        // Validate emails if enabled
        let mut emails_found = Vec::new();
        if self.config.validate_emails {
            for cap in self.email_regex.find_iter(&sanitized_input) {
                let email = cap.as_str().to_string();
                emails_found.push(email.clone());
                
                if let Err(e) = self.validate_email(&email) {
                    warnings.push(format!("Invalid email: {}", e));
                    if self.config.strict_mode {
                        errors.push(format!("Invalid email detected: {}", e));
                    }
                }
            }
        }

        let processing_time = start_time.elapsed().as_millis() as u64;
        let is_valid = errors.is_empty();

        let result = ValidationResult {
            is_valid,
            sanitized_input,
            warnings,
            errors,
            detected_threats,
            metadata: ValidationMetadata {
                original_length: input.len(),
                sanitized_length: input.len(),
                patterns_matched: detected_threats.clone(),
                urls_found,
                emails_found,
                processing_time_ms: processing_time,
            },
        };

        if !is_valid && self.config.strict_mode {
            return Err(ValidationError::MaliciousContent(
                format!("Validation failed: {:?}", errors)
            ));
        }

        Ok(result)
    }

    fn sanitize_html(&self, input: &str) -> String {
        let sanitized = self.html_tag_regex.replace_all(input, "");
        
        // Additional HTML entity encoding
        sanitized
            .replace("&", "&amp;")
            .replace("<", "&lt;")
            .replace(">", "&gt;")
            .replace("\"", "&quot;")
            .replace("'", "&#x27;")
            .to_string()
    }

    fn validate_url(&self, url: &str) -> Result<(), ValidationError> {
        // Basic URL format validation
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(ValidationError::InvalidUrl("URL must start with http:// or https://".to_string()));
        }

        // Check against allowed domains if specified
        if !self.config.allowed_domains.is_empty() {
            let domain_match = self.config.allowed_domains.iter()
                .any(|allowed| url.contains(allowed));
            
            if !domain_match {
                return Err(ValidationError::InvalidUrl(
                    format!("URL domain not in allowed list: {}", url)
                ));
            }
        }

        Ok(())
    }

    fn validate_email(&self, email: &str) -> Result<(), ValidationError> {
        // Basic email format validation
        if !email.contains('@') || !email.contains('.') {
            return Err(ValidationError::InvalidEmail(
                "Invalid email format".to_string()
            ));
        }

        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err(ValidationError::InvalidEmail(
                "Invalid email structure".to_string()
            ));
        }

        if !parts[1].contains('.') {
            return Err(ValidationError::InvalidEmail(
                "Email domain must contain a dot".to_string()
            ));
        }

        Ok(())
    }

    pub fn quick_validate(&self, input: &str) -> bool {
        // Fast validation for high-throughput scenarios
        if input.len() > self.config.max_input_length {
            return false;
        }

        for regex in &self.forbidden_regexes {
            if regex.is_match(input) {
                return false;
            }
        }

        true
    }

    pub fn batch_validate(&self, inputs: &[String]) -> Vec<ValidationResult> {
        inputs.iter()
            .map(|input| self.validate_and_sanitize(input).unwrap_or_else(|e| ValidationResult {
                is_valid: false,
                sanitized_input: input.clone(),
                warnings: vec![],
                errors: vec![e.to_string()],
                detected_threats: vec![],
                metadata: ValidationMetadata {
                    original_length: input.len(),
                    sanitized_length: input.len(),
                    patterns_matched: vec![],
                    urls_found: vec![],
                    emails_found: vec![],
                    processing_time_ms: 0,
                },
            }))
            .collect()
    }

    pub fn add_forbidden_pattern(&mut self, pattern: String) -> Result<(), ValidationError> {
        let regex = Regex::new(&pattern)?;
        self.forbidden_regexes.push(regex);
        self.config.forbidden_patterns.push(pattern);
        Ok(())
    }

    pub fn remove_forbidden_pattern(&mut self, index: usize) {
        if index < self.config.forbidden_patterns.len() {
            self.config.forbidden_patterns.remove(index);
            self.forbidden_regexes.remove(index);
        }
    }

    pub fn add_allowed_domain(&mut self, domain: String) {
        self.config.allowed_domains.push(domain);
    }

    pub fn set_strict_mode(&mut self, strict: bool) {
        self.config.strict_mode = strict;
    }

    pub fn get_config(&self) -> &ValidationConfig {
        &self.config
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationStats {
    pub total_validations: u64,
    pub successful_validations: u64,
    pub failed_validations: u64,
    pub threats_detected: u64,
    pub average_processing_time_ms: f64,
    pub most_common_threats: Vec<(String, u64)>,
}

pub struct ValidationMetrics {
    total_validations: u64,
    successful_validations: u64,
    failed_validations: u64,
    threats_detected: u64,
    total_processing_time_ms: u64,
    threat_counts: std::collections::HashMap<String, u64>,
}

impl ValidationMetrics {
    pub fn new() -> Self {
        Self {
            total_validations: 0,
            successful_validations: 0,
            failed_validations: 0,
            threats_detected: 0,
            total_processing_time_ms: 0,
            threat_counts: std::collections::HashMap::new(),
        }
    }

    pub fn record_validation(&mut self, result: &ValidationResult) {
        self.total_validations += 1;
        self.total_processing_time_ms += result.metadata.processing_time_ms;

        if result.is_valid {
            self.successful_validations += 1;
        } else {
            self.failed_validations += 1;
        }

        self.threats_detected += result.detected_threats.len() as u64;

        for threat in &result.detected_threats {
            *self.threat_counts.entry(threat.clone()).or_insert(0) += 1;
        }
    }

    pub fn get_stats(&self) -> ValidationStats {
        let average_processing_time = if self.total_validations > 0 {
            self.total_processing_time_ms as f64 / self.total_validations as f64
        } else {
            0.0
        };

        let mut most_common_threats: Vec<_> = self.threat_counts.iter().collect();
        most_common_threats.sort_by(|a, b| b.1.cmp(a.1));
        most_common_threats.truncate(10);
        most_common_threats = most_common_threats.into_iter()
            .map(|(threat, count)| (threat.clone(), *count))
            .collect();

        ValidationStats {
            total_validations: self.total_validations,
            successful_validations: self.successful_validations,
            failed_validations: self.failed_validations,
            threats_detected: self.threats_detected,
            average_processing_time_ms: average_processing_time,
            most_common_threats,
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xss_detection() {
        let config = ValidationConfig::default();
        let validator = InputValidator::new(config).unwrap();

        let malicious_input = "<script>alert('xss')</script>";
        let result = validator.validate_and_sanitize(malicious_input);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ValidationError::MaliciousContent(_)));
    }

    #[test]
    fn test_html_sanitization() {
        let mut config = ValidationConfig::default();
        config.strict_mode = false;
        let validator = InputValidator::new(config).unwrap();

        let html_input = "Hello <b>world</b> <script>alert('bad')</script>";
        let result = validator.validate_and_sanitize(html_input).unwrap();
        
        assert!(result.sanitized_input.contains("Hello"));
        assert!(!result.sanitized_input.contains("<script>"));
        assert!(!result.sanitized_input.contains("<b>"));
    }

    #[test]
    fn test_url_validation() {
        let config = ValidationConfig::default();
        let validator = InputValidator::new(config).unwrap();

        let valid_url = "https://github.com/user/repo";
        let result = validator.validate_and_sanitize(valid_url);
        assert!(result.is_ok());
        assert!(result.unwrap().is_valid);

        let invalid_url = "https://malicious-site.com";
        let result = validator.validate_and_sanitize(invalid_url);
        assert!(result.is_ok());
        assert!(!result.unwrap().is_valid);
    }

    #[test]
    fn test_input_length_validation() {
        let mut config = ValidationConfig::default();
        config.max_input_length = 10;
        let validator = InputValidator::new(config).unwrap();

        let long_input = "This is a very long input that exceeds the limit";
        let result = validator.validate_and_sanitize(long_input);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ValidationError::InputTooLong { .. }));
    }

    #[test]
    fn test_batch_validation() {
        let config = ValidationConfig::default();
        let validator = InputValidator::new(config).unwrap();

        let inputs = vec![
            "Normal input".to_string(),
            "<script>alert('xss')</script>".to_string(),
            "Another normal input".to_string(),
        ];

        let results = validator.batch_validate(&inputs);
        assert_eq!(results.len(), 3);
        assert!(results[0].is_valid);
        assert!(!results[1].is_valid);
        assert!(results[2].is_valid);
    }

    #[test]
    fn test_quick_validate() {
        let config = ValidationConfig::default();
        let validator = InputValidator::new(config).unwrap();

        assert!(validator.quick_validate("Normal input"));
        assert!(!validator.quick_validate("<script>alert('xss')</script>"));
    }
}
