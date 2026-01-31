use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FilterError {
    #[error("Content contains blocked patterns: {0}")]
    BlockedContent(String),
    #[error("Content exceeds maximum length: {current} > {max}")]
    ContentTooLong { current: usize, max: usize },
    #[error("Regex compilation failed: {0}")]
    RegexError(#[from] regex::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterConfig {
    pub max_content_length: usize,
    pub blocked_patterns: Vec<String>,
    pub allowed_domains: Vec<String>,
    pub sensitive_keywords: Vec<String>,
    pub content_types: HashSet<String>,
}

impl Default for FilterConfig {
    fn default() -> Self {
        let mut content_types = HashSet::new();
        content_types.insert("text/plain".to_string());
        content_types.insert("application/json".to_string());
        content_types.insert("text/markdown".to_string());

        Self {
            max_content_length: 100_000,
            blocked_patterns: vec![
                r"(?i)(password|secret|token|key)\s*[:=]\s*\S+".to_string(),
                r"(?i)\b\d{4}[-\s]?\d{4}[-\s]?\d{4}[-\s]?\d{4}\b".to_string(),
                r"(?i)\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b".to_string(),
            ],
            allowed_domains: vec![
                "github.com".to_string(),
                "stackoverflow.com".to_string(),
                "docs.rs".to_string(),
            ],
            sensitive_keywords: vec![
                "credit_card".to_string(),
                "ssn".to_string(),
                "api_key".to_string(),
                "private_key".to_string(),
            ],
            content_types,
        }
    }
}

#[derive(Debug, Clone)]
pub struct OutputFilter {
    config: FilterConfig,
    blocked_regexes: Vec<Regex>,
    sensitive_regex: Regex,
}

impl OutputFilter {
    pub fn new(config: FilterConfig) -> Result<Self, FilterError> {
        let mut blocked_regexes = Vec::new();
        for pattern in &config.blocked_patterns {
            blocked_regexes.push(Regex::new(pattern)?);
        }

        let sensitive_pattern = config.sensitive_keywords.join("|");
        let sensitive_regex = Regex::new(&format!(r"(?i)\b({})\b", sensitive_pattern))?;

        Ok(Self {
            config,
            blocked_regexes,
            sensitive_regex,
        })
    }

    pub fn filter_content(&self, content: &str, user_tier: &mr_darkpromth_db::UserTier) -> Result<String, FilterError> {
        // Ultra Tier Bypass: Skip all content filtering for Ultra users
        if matches!(user_tier, mr_darkpromth_db::UserTier::Ultra) {
            return Ok(content.to_string());
        }
        if content.len() > self.config.max_content_length {
            return Err(FilterError::ContentTooLong {
                current: content.len(),
                max: self.config.max_content_length,
            });
        }

        let mut filtered_content = content.to_string();

        for regex in &self.blocked_regexes {
            if regex.is_match(content) {
                let matches: Vec<_> = regex.find_iter(content).collect();
                for m in matches {
                    let replacement = "*".repeat(m.as_str().len());
                    filtered_content = filtered_content.replace(m.as_str(), &replacement);
                }
                return Err(FilterError::BlockedContent(
                    "Sensitive information detected and redacted".to_string()
                ));
            }
        }

        if self.sensitive_regex.is_match(content) {
            log::warn!("Sensitive keywords detected in content");
        }

        Ok(filtered_content)
    }

    pub fn sanitize_urls(&self, content: &str) -> String {
        let url_regex = Regex::new(r"https?://[^\s]+").unwrap();
        url_regex.replace_all(content, |caps: &regex::Captures| {
            let url = caps.get(0).unwrap().as_str();
            if let Some(domain) = url.split("://").nth(1) {
                let base_domain = domain.split('/').next().unwrap_or(domain);
                if self.config.allowed_domains.contains(&base_domain.to_string()) {
                    url.to_string()
                } else {
                    "[URL_FILTERED]".to_string()
                }
            } else {
                "[URL_FILTERED]".to_string()
            }
        }).to_string()
    }

    pub fn validate_content_type(&self, content_type: &str) -> bool {
        self.config.content_types.contains(content_type)
    }

    pub fn get_stats(&self, content: &str) -> FilterStats {
        let mut blocked_matches = 0;
        let mut sensitive_matches = 0;

        for regex in &self.blocked_regexes {
            blocked_matches += regex.find_iter(content).count();
        }

        sensitive_matches += self.sensitive_regex.find_iter(content).count();

        FilterStats {
            content_length: content.len(),
            blocked_matches,
            sensitive_matches,
            is_safe: blocked_matches == 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterStats {
    pub content_length: usize,
    pub blocked_matches: usize,
    pub sensitive_matches: usize,
    pub is_safe: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_sensitive_info() {
        let config = FilterConfig::default();
        let filter = OutputFilter::new(config).unwrap();

        let content = "My password is secret123 and my email is test@example.com";
        let result = filter.filter_content(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_url_filtering() {
        let config = FilterConfig::default();
        let filter = OutputFilter::new(config).unwrap();

        let content = "Visit https://github.com and https://malicious-site.com";
        let filtered = filter.sanitize_urls(content);
        assert!(filtered.contains("github.com"));
        assert!(!filtered.contains("malicious-site.com"));
        assert!(filtered.contains("[URL_FILTERED]"));
    }

    #[test]
    fn test_content_length_validation() {
        let mut config = FilterConfig::default();
        config.max_content_length = 10;
        let filter = OutputFilter::new(config).unwrap();

        let content = "This is a very long content that exceeds the limit";
        let result = filter.filter_content(content);
        assert!(result.is_err());
    }
}
