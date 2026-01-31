use crate::correction_validator::CorrectionAttempt;
use crate::error_detector::{ErrorCategory, ErrorDetection};
use crate::fix_generator::{GeneratedFix, RiskLevel};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrectionHistory {
    pub id: uuid::Uuid,
    pub error_id: uuid::Uuid,
    pub fix_id: uuid::Uuid,
    pub error_category: ErrorCategory,
    pub error_message: String,
    pub fix_description: String,
    pub confidence_score: f64,
    pub risk_level: RiskLevel,
    pub validation_score: f64,
    pub outcome: CorrectionOutcome,
    pub lessons_learned: Vec<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CorrectionOutcome {
    Success,
    PartialSuccess,
    Failure,
    RolledBack,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPattern {
    pub id: uuid::Uuid,
    pub pattern: String,
    pub category: ErrorCategory,
    pub frequency: u32,
    pub success_rate: f64,
    pub recommended_fix_approach: String,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningMetrics {
    pub total_corrections: u64,
    pub successful_corrections: u64,
    pub failed_corrections: u64,
    pub average_confidence: f64,
    pub average_validation_score: f64,
    pub most_common_errors: Vec<(ErrorCategory, u32)>,
}

#[derive(Error, Debug)]
pub enum LearningError {
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Failed to analyze pattern: {0}")]
    PatternAnalysisError(String),
    #[error("Failed to generate insights: {0}")]
    InsightGenerationError(String),
}

pub struct LearningSystem {
    db_pool: PgPool,
    pattern_cache: HashMap<String, ErrorPattern>,
    metrics: LearningMetrics,
}

impl LearningSystem {
    pub fn new(db_pool: PgPool) -> Self {
        Self {
            db_pool,
            pattern_cache: HashMap::new(),
            metrics: LearningMetrics {
                total_corrections: 0,
                successful_corrections: 0,
                failed_corrections: 0,
                average_confidence: 0.0,
                average_validation_score: 0.0,
                most_common_errors: Vec::new(),
            },
        }
    }

    pub async fn record_correction(&mut self, attempt: &CorrectionAttempt, fix: &GeneratedFix, error: &ErrorDetection) -> Result<(), LearningError> {
        let outcome = match attempt.status {
            crate::correction_validator::CorrectionStatus::Validated => CorrectionOutcome::Success,
            crate::correction_validator::CorrectionStatus::RolledBack => CorrectionOutcome::RolledBack,
            crate::correction_validator::CorrectionStatus::Failed => CorrectionOutcome::Failure,
            _ => CorrectionOutcome::PartialSuccess,
        };

        let history = CorrectionHistory {
            id: uuid::Uuid::new_v4(),
            error_id: error.id,
            fix_id: fix.id,
            error_category: error.category.clone(),
            error_message: error.message.clone(),
            fix_description: fix.description.clone(),
            confidence_score: fix.confidence_score,
            risk_level: fix.estimated_risk.clone(),
            validation_score: attempt.validation_score,
            outcome: outcome.clone(),
            lessons_learned: self.extract_lessons_learned(attempt, fix, error),
            timestamp: chrono::Utc::now(),
        };

        sqlx::query(
            r#"
            INSERT INTO correction_history (
                id, error_id, fix_id, error_category, error_message, fix_description,
                confidence_score, risk_level, validation_score, outcome, lessons_learned, timestamp
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#
        )
        .bind(history.id)
        .bind(history.error_id)
        .bind(history.fix_id)
        .bind(format!("{:?}", history.error_category))
        .bind(&history.error_message)
        .bind(&history.fix_description)
        .bind(history.confidence_score)
        .bind(format!("{:?}", history.risk_level))
        .bind(history.validation_score)
        .bind(format!("{:?}", history.outcome))
        .bind(&history.lessons_learned)
        .bind(history.timestamp)
        .execute(&self.db_pool)
        .await
        .map_err(|e| LearningError::DatabaseError(e.to_string()))?;

        self.update_metrics(&history);
        self.analyze_error_patterns(error).await?;

        Ok(())
    }

    fn extract_lessons_learned(&self, attempt: &CorrectionAttempt, fix: &GeneratedFix, error: &ErrorDetection) -> Vec<String> {
        let mut lessons = Vec::new();

        if attempt.validation_score >= 0.9 {
            lessons.push(format!(
                "High-confidence fix ({}%) for {:?} errors works well",
                (fix.confidence_score * 100.0) as u32,
                error.category
            ));
        }

        if fix.estimated_risk == RiskLevel::Low && attempt.validation_score >= 0.8 {
            lessons.push("Low-risk fixes with good reasoning are reliable".to_string());
        }

        if fix.suggested_tests.len() >= 2 && attempt.validation_score >= 0.8 {
            lessons.push("Multiple test suggestions correlate with successful fixes".to_string());
        }

        if attempt.validation_score < 0.5 {
            lessons.push(format!(
                "Fixes with {}% validation score often need manual review",
                (attempt.validation_score * 100.0) as u32
            ));
        }

        lessons
    }

    async fn analyze_error_patterns(&mut self, error: &ErrorDetection) -> Result<(), LearningError> {
        let pattern_key = self.extract_pattern_key(error);

        if let Some(existing_pattern) = self.pattern_cache.get_mut(&pattern_key) {
            existing_pattern.frequency += 1;
            existing_pattern.last_updated = chrono::Utc::now();
        } else {
            let pattern = ErrorPattern {
                id: uuid::Uuid::new_v4(),
                pattern: pattern_key.clone(),
                category: error.category.clone(),
                frequency: 1,
                success_rate: 0.0,
                recommended_fix_approach: self.determine_fix_approach(error),
                last_updated: chrono::Utc::now(),
            };
            self.pattern_cache.insert(pattern_key, pattern);
        }

        Ok(())
    }

    fn extract_pattern_key(&self, error: &ErrorDetection) -> String {
        let mut key = format!("{:?}", error.category);
        
        if let Some(function) = &error.context.function_name {
            key.push_str(&format!("::{}", function));
        }
        
        key
    }

    fn determine_fix_approach(&self, error: &ErrorDetection) -> String {
        match error.category {
            ErrorCategory::SyntaxError => "Review syntax and structure, check for typos".to_string(),
            ErrorCategory::RuntimeError => "Check for null references, bounds, and error handling".to_string(),
            ErrorCategory::LogicalError => "Review logic flow and assertions".to_string(),
            ErrorCategory::NetworkError => "Check connectivity and timeout configurations".to_string(),
            ErrorCategory::DatabaseError => "Review queries and connection handling".to_string(),
            ErrorCategory::AuthenticationError => "Check credentials and token validation".to_string(),
            ErrorCategory::ConfigurationError => "Review configuration files and environment variables".to_string(),
            ErrorCategory::Unknown => "Investigate error context and logs".to_string(),
        }
    }

    fn update_metrics(&mut self, history: &CorrectionHistory) {
        self.metrics.total_corrections += 1;

        match history.outcome {
            CorrectionOutcome::Success => self.metrics.successful_corrections += 1,
            CorrectionOutcome::PartialSuccess => self.metrics.successful_corrections += 1,
            CorrectionOutcome::Failure | CorrectionOutcome::RolledBack => self.metrics.failed_corrections += 1,
        }

        let total = self.metrics.total_corrections as f64;
        self.metrics.average_confidence = (self.metrics.average_confidence * (total - 1.0) + history.confidence_score) / total;
        self.metrics.average_validation_score = (self.metrics.average_validation_score * (total - 1.0) + history.validation_score) / total;

        self.update_most_common_errors(&history.error_category);
    }

    fn update_most_common_errors(&mut self, category: &ErrorCategory) {
        let category_str = format!("{:?}", category);
        
        if let Some(index) = self.metrics.most_common_errors.iter().position(|(cat, _)| {
            format!("{:?}", cat) == category_str
        }) {
            self.metrics.most_common_errors[index].1 += 1;
        } else {
            self.metrics.most_common_errors.push((category.clone(), 1));
        }

        self.metrics.most_common_errors.sort_by(|a, b| b.1.cmp(&a.1));
        self.metrics.most_common_errors.truncate(5);
    }

    pub async fn get_insights(&self) -> Result<String, LearningError> {
        let mut insights = Vec::new();

        insights.push("=== Self-Correction Engine Insights ===".to_string());
        insights.push(format!("Total Corrections: {}", self.metrics.total_corrections));
        insights.push(format!("Success Rate: {:.1}%", 
            (self.metrics.successful_corrections as f64 / self.metrics.total_corrections as f64) * 100.0
        ));
        insights.push(format!("Average Confidence: {:.1}%", self.metrics.average_confidence * 100.0));
        insights.push(format!("Average Validation Score: {:.1}%", self.metrics.average_validation_score * 100.0));

        insights.push("\n=== Most Common Error Categories ===".to_string());
        for (category, count) in &self.metrics.most_common_errors {
            insights.push(format!("{:?}: {} occurrences", category, count));
        }

        insights.push("\n=== Top Error Patterns ===".to_string());
        let mut patterns: Vec<_> = self.pattern_cache.values().collect();
        patterns.sort_by(|a, b| b.frequency.cmp(&a.frequency));
        
        for pattern in patterns.iter().take(5) {
            insights.push(format!(
                "{}: {} occurrences - Recommended: {}",
                pattern.pattern,
                pattern.frequency,
                pattern.recommended_fix_approach
            ));
        }

        if self.metrics.total_corrections > 10 {
            insights.push("\n=== Recommendations ===".to_string());
            
            if self.metrics.average_validation_score > 0.8 {
                insights.push("System is performing well. Continue current approach.".to_string());
            } else {
                insights.push("Consider increasing confidence threshold for automated fixes.".to_string());
            }

            if self.metrics.failed_corrections > self.metrics.successful_corrections / 2 {
                insights.push("High failure rate detected. Review fix quality criteria.".to_string());
            }
        }

        Ok(insights.join("\n"))
    }

    pub fn get_correction_history(&self, _limit: usize) -> Result<Vec<CorrectionHistory>, LearningError> {
        Ok(Vec::new())
    }

    pub async fn improve_fix_generation(&self, _error: &ErrorDetection, similar_errors: Vec<CorrectionHistory>) -> Vec<String> {
        let mut suggestions = Vec::new();

        for history in similar_errors {
            if history.outcome == CorrectionOutcome::Success && history.validation_score >= 0.8 {
                suggestions.push(format!(
                    "Similar error resolved with: {} (confidence: {}%)",
                    history.fix_description,
                    (history.confidence_score * 100.0) as u32
                ));
            }
        }

        suggestions
    }

    pub async fn find_similar_errors(&self, _error: &ErrorDetection) -> Vec<CorrectionHistory> {
        Vec::new()
    }

    pub fn get_metrics(&self) -> &LearningMetrics {
        &self.metrics
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_extraction() {
        let system = LearningSystem::new(sqlx::PgPool::connect_lazy("postgresql://localhost/test").unwrap());
        
        let error = ErrorDetection {
            id: uuid::Uuid::new_v4(),
            category: ErrorCategory::SyntaxError,
            severity: crate::error_detector::ErrorSeverity::High,
            message: "syntax error".to_string(),
            source: crate::error_detector::ErrorSource::ApplicationLog,
            context: crate::error_detector::ErrorContext {
                file_path: None,
                line_number: None,
                function_name: Some("test_function".to_string()),
                request_id: None,
                user_id: None,
                additional_metadata: HashMap::new(),
            },
            timestamp: chrono::Utc::now(),
            stack_trace: None,
        };

        let pattern_key = system.extract_pattern_key(&error);
        assert!(pattern_key.contains("SyntaxError"));
        assert!(pattern_key.contains("test_function"));
    }

    #[test]
    fn test_metrics_update() {
        let mut system = LearningSystem::new(sqlx::PgPool::connect_lazy("postgresql://localhost/test").unwrap());
        
        let history = CorrectionHistory {
            id: uuid::Uuid::new_v4(),
            error_id: uuid::Uuid::new_v4(),
            fix_id: uuid::Uuid::new_v4(),
            error_category: ErrorCategory::SyntaxError,
            error_message: "test".to_string(),
            fix_description: "test fix".to_string(),
            confidence_score: 0.85,
            risk_level: RiskLevel::Low,
            validation_score: 0.9,
            outcome: CorrectionOutcome::Success,
            lessons_learned: vec!["lesson1".to_string()],
            timestamp: chrono::Utc::now(),
        };

        system.update_metrics(&history);
        
        assert_eq!(system.metrics.total_corrections, 1);
        assert_eq!(system.metrics.successful_corrections, 1);
    }
}
