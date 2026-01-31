use crate::error_detector::{ErrorCategory, ErrorDetection};
use cerebras_client::CerebrasClient;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedFix {
    pub id: uuid::Uuid,
    pub error_id: uuid::Uuid,
    pub description: String,
    pub code_changes: Vec<CodeChange>,
    pub confidence_score: f64,
    pub reasoning: String,
    pub estimated_risk: RiskLevel,
    pub suggested_tests: Vec<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeChange {
    pub file_path: String,
    pub line_number: u32,
    pub old_code: String,
    pub new_code: String,
    pub change_type: ChangeType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    Insert,
    Replace,
    Delete,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Error, Debug)]
pub enum FixGenerationError {
    #[error("Failed to generate fix: {0}")]
    GenerationFailed(String),
    #[error("Failed to parse AI response: {0}")]
    ParseError(String),
    #[error("Cerebras API error: {0}")]
    ApiError(String),
}

pub struct FixGenerator {
    cerebras_client: CerebrasClient,
    confidence_threshold: f64,
}

impl FixGenerator {
    pub fn new(cerebras_client: CerebrasClient, confidence_threshold: f64) -> Self {
        Self {
            cerebras_client,
            confidence_threshold,
        }
    }

    fn extract_response_text(response: &cerebras_client::ChatCompletionResponse) -> String {
        response
            .choices
            .first()
            .map(|choice| choice.message.content.clone())
            .unwrap_or_default()
    }

    pub async fn generate_fix(&self, error: &ErrorDetection, code_context: &str) -> Result<GeneratedFix, FixGenerationError> {
        let prompt = self.build_fix_prompt(error, code_context);
        
        let request = cerebras_client::ChatRequest {
            messages: vec![
                cerebras_client::ChatMessage {
                    role: cerebras_client::Role::User,
                    content: prompt,
                }
            ],
            model: Some(cerebras_client::Model::Llama3_3_70b),
            temperature: Some(0.7f32),
            max_tokens: Some(2000),
            top_p: Some(1.0f32),
            stream: Some(false),
            intent: Some(cerebras_client::RequestIntent::Code),
        };
        
        let response = self.cerebras_client.chat_completion(request).await
            .map_err(|e: cerebras_client::CerebrasClientError| FixGenerationError::ApiError(e.to_string()))?;
        
        let response_text = Self::extract_response_text(&response);
        
        self.parse_fix_response(response_text, error)
    }

    pub async fn generate_multiple_fixes(&self, error: &ErrorDetection, code_context: &str, count: usize) -> Vec<GeneratedFix> {
        let mut fixes = Vec::new();
        
        for i in 0..count {
            let prompt = format!(
                "{}\n\nGenerate fix variant #{} of {}. Provide a different approach than previous fixes.",
                self.build_fix_prompt(error, code_context),
                i + 1,
                count
            );
            
            let request = cerebras_client::ChatRequest {
                messages: vec![
                    cerebras_client::ChatMessage {
                        role: cerebras_client::Role::User,
                        content: prompt,
                    }
                ],
                model: Some(cerebras_client::Model::Llama3_3_70b),
                max_tokens: Some(2000),
                temperature: Some(0.7f32),
                top_p: Some(1.0f32),
                stream: Some(false),
                intent: Some(cerebras_client::RequestIntent::Code),
            };
            
            if let Ok(response) = self.cerebras_client.chat_completion(request).await {
                let response_text = Self::extract_response_text(&response);
                if let Ok(fix) = self.parse_fix_response(response_text, error) {
                    fixes.push(fix);
                }
            }
        }
        
        fixes.sort_by(|a, b| b.confidence_score.partial_cmp(&a.confidence_score).unwrap());
        fixes
    }

    fn build_fix_prompt(&self, error: &ErrorDetection, code_context: &str) -> String {
        format!(
            "You are an expert software engineer specializing in debugging and error correction.\n\n\
            Error Details:\n\
            - Category: {:?}\n\
            - Severity: {:?}\n\
            - Message: {}\n\
            - Source: {:?}\n\n\
            Code Context:\n\
            {}\n\n\
            Task: Analyze this error and provide a fix. Your response must be valid JSON with this structure:\n\
            {{\n\
              \"description\": \"Brief description of the fix\",\n\
              \"code_changes\": [\n\
                {{\n\
                  \"file_path\": \"path/to/file\",\n\
                  \"line_number\": 42,\n\
                  \"old_code\": \"original code\",\n\
                  \"new_code\": \"fixed code\",\n\
                  \"change_type\": \"replace\"\n\
                }}\n\
              ],\n\
              \"reasoning\": \"Explanation of why this fix works\",\n\
              \"estimated_risk\": \"low|medium|high|critical\",\n\
              \"suggested_tests\": [\"test1\", \"test2\"]\n\
            }}\n\n\
            Guidelines:\n\
            1. Provide minimal, focused changes\n\
            2. Explain your reasoning clearly\n\
            3. Estimate the risk level honestly\n\
            4. Suggest tests to validate the fix\n\
            5. Consider edge cases and side effects",
            error.category,
            error.severity,
            error.message,
            error.source,
            code_context
        )
    }

    fn parse_fix_response(&self, response: String, error: &ErrorDetection) -> Result<GeneratedFix, FixGenerationError> {
        let json_str = self.extract_json(&response)
            .ok_or_else(|| FixGenerationError::ParseError("No JSON found in response".to_string()))?;
        
        let fix_data: serde_json::Value = serde_json::from_str(&json_str)
            .map_err(|e| FixGenerationError::ParseError(e.to_string()))?;
        
        let confidence_score = self.calculate_confidence(&fix_data, error);
        
        Ok(GeneratedFix {
            id: uuid::Uuid::new_v4(),
            error_id: error.id,
            description: fix_data["description"].as_str().unwrap_or("No description").to_string(),
            code_changes: self.parse_code_changes(&fix_data["code_changes"])?,
            confidence_score,
            reasoning: fix_data["reasoning"].as_str().unwrap_or("No reasoning provided").to_string(),
            estimated_risk: self.parse_risk_level(fix_data["estimated_risk"].as_str()),
            suggested_tests: self.parse_suggested_tests(&fix_data["suggested_tests"]),
            timestamp: chrono::Utc::now(),
        })
    }

    fn extract_json(&self, text: &str) -> Option<String> {
        let start = text.find('{')?;
        let mut brace_count = 0;
        let mut end = start;
        
        for (i, c) in text[start..].chars().enumerate() {
            match c {
                '{' => brace_count += 1,
                '}' => {
                    brace_count -= 1;
                    if brace_count == 0 {
                        end = start + i;
                        break;
                    }
                }
                _ => {}
            }
        }
        
        if brace_count == 0 {
            Some(text[start..=end].to_string())
        } else {
            None
        }
    }

    fn parse_code_changes(&self, changes: &serde_json::Value) -> Result<Vec<CodeChange>, FixGenerationError> {
        changes.as_array()
            .ok_or_else(|| FixGenerationError::ParseError("code_changes is not an array".to_string()))?
            .iter()
            .map(|change| {
                Ok(CodeChange {
                    file_path: change["file_path"].as_str().unwrap_or("").to_string(),
                    line_number: change["line_number"].as_u64().unwrap_or(0) as u32,
                    old_code: change["old_code"].as_str().unwrap_or("").to_string(),
                    new_code: change["new_code"].as_str().unwrap_or("").to_string(),
                    change_type: self.parse_change_type(change["change_type"].as_str()),
                })
            })
            .collect()
    }

    fn parse_change_type(&self, type_str: Option<&str>) -> ChangeType {
        match type_str {
            Some("insert") => ChangeType::Insert,
            Some("replace") => ChangeType::Replace,
            Some("delete") => ChangeType::Delete,
            _ => ChangeType::Replace,
        }
    }

    fn parse_risk_level(&self, risk_str: Option<&str>) -> RiskLevel {
        match risk_str {
            Some("low") => RiskLevel::Low,
            Some("medium") => RiskLevel::Medium,
            Some("high") => RiskLevel::High,
            Some("critical") => RiskLevel::Critical,
            _ => RiskLevel::Medium,
        }
    }

    fn parse_suggested_tests(&self, tests: &serde_json::Value) -> Vec<String> {
        tests.as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|t| t.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default()
    }

    fn calculate_confidence(&self, fix_data: &serde_json::Value, error: &ErrorDetection) -> f64 {
        let mut confidence: f64 = 0.5;
        
        let reasoning = fix_data["reasoning"].as_str().unwrap_or("");
        if reasoning.len() > 100 {
            confidence += 0.15;
        }
        
        let empty_vec = vec![];
        let code_changes = fix_data["code_changes"].as_array().unwrap_or(&empty_vec);
        if code_changes.len() <= 3 {
            confidence += 0.15;
        }
        
        let suggested_tests = fix_data["suggested_tests"].as_array().unwrap_or(&empty_vec);
        if !suggested_tests.is_empty() {
            confidence += 0.1;
        }
        
        match error.category {
            ErrorCategory::SyntaxError => confidence += 0.2,
            ErrorCategory::RuntimeError => confidence += 0.1,
            ErrorCategory::LogicalError => confidence -= 0.1,
            _ => {}
        }
        
        confidence.min(0.99).max(0.1) as f64
    }

    pub fn should_apply_fix(&self, fix: &GeneratedFix) -> bool {
        fix.confidence_score >= self.confidence_threshold
            && fix.estimated_risk != RiskLevel::Critical
    }

    pub fn select_best_fix<'a>(&self, fixes: &'a [GeneratedFix]) -> Option<&'a GeneratedFix> {
        fixes.iter()
            .filter(|f| self.should_apply_fix(f))
            .max_by(|a, b| a.confidence_score.partial_cmp(&b.confidence_score).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_extraction() {
        let config = cerebras_client::CerebrasConfig::new(vec!["test_key".to_string()])
            .expect("config");
        let generator = FixGenerator::new(
            CerebrasClient::new(config),
            0.7
        );
        
        let text = r#"Some text here {"description": "test"} more text"#;
        let json = generator.extract_json(text);
        assert!(json.is_some());
    }

    #[test]
    fn test_confidence_calculation() {
        let config = cerebras_client::CerebrasConfig::new(vec!["test_key".to_string()])
            .expect("config");
        let generator = FixGenerator::new(
            CerebrasClient::new(config),
            0.7
        );
        
        let fix_data = serde_json::json!({
            "reasoning": "This is a very detailed explanation of why this fix works and what it does to solve the problem.",
            "code_changes": [{"file_path": "test.rs", "line_number": 42}],
            "suggested_tests": ["test1", "test2"]
        });
        
        let error = ErrorDetection {
            id: uuid::Uuid::new_v4(),
            category: ErrorCategory::SyntaxError,
            severity: crate::error_detector::ErrorSeverity::High,
            message: "syntax error".to_string(),
            source: crate::error_detector::ErrorSource::ApplicationLog,
            context: crate::error_detector::ErrorContext {
                file_path: None,
                line_number: None,
                function_name: None,
                request_id: None,
                user_id: None,
                additional_metadata: HashMap::new(),
            },
            timestamp: chrono::Utc::now(),
            stack_trace: None,
        };
        
        let confidence = generator.calculate_confidence(&fix_data, &error);
        assert!(confidence > 0.5);
        assert!(confidence < 1.0);
    }
}
