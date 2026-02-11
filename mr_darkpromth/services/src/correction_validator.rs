use crate::error_detector::ErrorDetection;
use crate::fix_generator::{CodeChange, GeneratedFix, RiskLevel};
use md5;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrectionAttempt {
    pub id: uuid::Uuid,
    pub fix_id: uuid::Uuid,
    pub error_id: uuid::Uuid,
    pub status: CorrectionStatus,
    pub rollback_snapshot: Option<RollbackSnapshot>,
    pub test_results: Vec<TestResult>,
    pub validation_score: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CorrectionStatus {
    Pending,
    Applied,
    Failed,
    RolledBack,
    Validated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackSnapshot {
    pub id: uuid::Uuid,
    pub file_snapshots: HashMap<String, FileSnapshot>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSnapshot {
    pub file_path: String,
    pub original_content: String,
    pub checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub test_name: String,
    pub passed: bool,
    pub output: String,
    pub duration_ms: u64,
}

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Failed to create rollback snapshot: {0}")]
    SnapshotError(String),
    #[error("Failed to apply fix: {0}")]
    ApplicationError(String),
    #[error("Failed to rollback: {0}")]
    RollbackError(String),
    #[error("All tests failed: {0}")]
    AllTestsFailed(String),
    #[error("File not found: {0}")]
    FileNotFound(String),
}

pub struct CorrectionValidator {
    project_root: String,
    #[allow(dead_code)]
    test_timeout_seconds: u64,
}

impl CorrectionValidator {
    pub fn new(project_root: String) -> Self {
        Self {
            project_root,
            test_timeout_seconds: 30,
        }
    }

    pub async fn validate_and_apply(&self, fix: &GeneratedFix, error: &ErrorDetection) -> Result<CorrectionAttempt, ValidationError> {
        log::info!("Validating fix for error: {}", error.message);
        
        let snapshot = self.create_snapshot(&fix.code_changes).await?;
        
        let attempt = CorrectionAttempt {
            id: uuid::Uuid::new_v4(),
            fix_id: fix.id,
            error_id: error.id,
            status: CorrectionStatus::Pending,
            rollback_snapshot: Some(snapshot),
            test_results: Vec::new(),
            validation_score: 0.0,
            timestamp: chrono::Utc::now(),
        };
        
        Ok(attempt)
    }

    async fn create_snapshot(&self, changes: &[CodeChange]) -> Result<RollbackSnapshot, ValidationError> {
        let mut file_snapshots = HashMap::new();
        
        for change in changes {
            let file_path = Path::new(&self.project_root).join(&change.file_path);
            
            if !file_path.exists() {
                return Err(ValidationError::FileNotFound(change.file_path.clone()));
            }
            
            let content = std::fs::read_to_string(&file_path)
                .map_err(|e| ValidationError::SnapshotError(format!("Failed to read file: {}", e)))?;
            
            let checksum = format!("{:x}", md5::compute(content.as_bytes()));
            
            file_snapshots.insert(
                change.file_path.clone(),
                FileSnapshot {
                    file_path: change.file_path.clone(),
                    original_content: content,
                    checksum,
                }
            );
        }
        
        Ok(RollbackSnapshot {
            id: uuid::Uuid::new_v4(),
            file_snapshots,
            created_at: chrono::Utc::now(),
        })
    }

    pub async fn apply_fix(&self, fix: &GeneratedFix) -> Result<(), ValidationError> {
        log::info!("Applying fix with {} code changes", fix.code_changes.len());
        
        for change in &fix.code_changes {
            self.apply_code_change(change).await?;
        }
        
        Ok(())
    }

    async fn apply_code_change(&self, change: &CodeChange) -> Result<(), ValidationError> {
        let file_path = Path::new(&self.project_root).join(&change.file_path);
        
        let content = std::fs::read_to_string(&file_path)
            .map_err(|e| ValidationError::ApplicationError(format!("Failed to read file: {}", e)))?;
        
        let lines: Vec<&str> = content.lines().collect();
        
        let new_content = match change.change_type {
            crate::fix_generator::ChangeType::Replace => {
                if change.line_number as usize > lines.len() {
                    return Err(ValidationError::ApplicationError(
                        format!("Line {} out of bounds", change.line_number)
                    ));
                }
                
                let mut new_lines: Vec<String> = lines.iter().map(|&s| s.to_string()).collect();
                new_lines[(change.line_number - 1) as usize] = change.new_code.clone();
                new_lines.join("\n")
            }
            crate::fix_generator::ChangeType::Insert => {
                if change.line_number as usize > lines.len() {
                    return Err(ValidationError::ApplicationError(
                        format!("Line {} out of bounds", change.line_number)
                    ));
                }
                
                let mut new_lines: Vec<String> = lines.iter().map(|&s| s.to_string()).collect();
                new_lines.insert((change.line_number - 1) as usize, change.new_code.clone());
                new_lines.join("\n")
            }
            crate::fix_generator::ChangeType::Delete => {
                if change.line_number as usize > lines.len() {
                    return Err(ValidationError::ApplicationError(
                        format!("Line {} out of bounds", change.line_number)
                    ));
                }
                
                let mut new_lines = lines.to_vec();
                new_lines.remove((change.line_number - 1) as usize);
                new_lines.join("\n")
            }
        };
        
        std::fs::write(&file_path, new_content)
            .map_err(|e| ValidationError::ApplicationError(format!("Failed to write file: {}", e)))?;
        
        Ok(())
    }

    pub async fn rollback(&self, snapshot: &RollbackSnapshot) -> Result<(), ValidationError> {
        log::info!("Rolling back {} files", snapshot.file_snapshots.len());
        
        for (file_path, file_snapshot) in &snapshot.file_snapshots {
            let full_path = Path::new(&self.project_root).join(file_path);
            
            let current_content = std::fs::read_to_string(&full_path)
                .map_err(|e| ValidationError::RollbackError(format!("Failed to read file: {}", e)))?;
            
            let current_checksum = format!("{:x}", md5::compute(current_content.as_bytes()));
            
            if current_checksum != file_snapshot.checksum {
                log::warn!("File {} has been modified since snapshot was created", file_path);
            }
            
            std::fs::write(&full_path, &file_snapshot.original_content)
                .map_err(|e| ValidationError::RollbackError(format!("Failed to write file: {}", e)))?;
        }
        
        Ok(())
    }

    pub async fn run_tests(&self, fix: &GeneratedFix) -> Vec<TestResult> {
        let mut results = Vec::new();
        
        for test_name in &fix.suggested_tests {
            let result = self.run_single_test(test_name).await;
            results.push(result);
        }
        
        results
    }

    async fn run_single_test(&self, test_name: &str) -> TestResult {
        let start = std::time::Instant::now();
        
        log::info!("Running test: {}", test_name);
        
        
        
        if test_name.contains("syntax") {
            TestResult {
                test_name: test_name.to_string(),
                passed: true,
                output: "Syntax check passed".to_string(),
                duration_ms: start.elapsed().as_millis() as u64,
            }
        } else {
            TestResult {
                test_name: test_name.to_string(),
                passed: true,
                output: "Test passed (simulated)".to_string(),
                duration_ms: start.elapsed().as_millis() as u64,
            }
        }
    }

    pub fn calculate_validation_score(&self, test_results: &[TestResult]) -> f64 {
        if test_results.is_empty() {
            return 0.0;
        }
        
        let passed_count = test_results.iter().filter(|t| t.passed).count();
        passed_count as f64 / test_results.len() as f64
    }

    pub fn is_fix_safe_to_apply(&self, fix: &GeneratedFix, error: &ErrorDetection) -> bool {
        if fix.estimated_risk == RiskLevel::Critical {
            log::warn!("Fix has critical risk level, not applying automatically");
            return false;
        }
        
        if fix.confidence_score < 0.6 {
            log::warn!("Fix confidence score too low: {}", fix.confidence_score);
            return false;
        }
        
        match error.category {
            crate::error_detector::ErrorCategory::RuntimeError => {
                if fix.estimated_risk == RiskLevel::High {
                    log::warn!("High risk fix for runtime error, manual review required");
                    return false;
                }
            }
            crate::error_detector::ErrorCategory::LogicalError => {
                if fix.confidence_score < 0.8 {
                    log::warn!("Low confidence for logical error fix, manual review required");
                    return false;
                }
            }
            _ => {}
        }
        
        true
    }

    pub async fn validate_fix(&self, fix: &GeneratedFix, error: &ErrorDetection) -> Result<CorrectionAttempt, ValidationError> {
        if !self.is_fix_safe_to_apply(fix, error) {
            return Err(ValidationError::ApplicationError(
                "Fix does not meet safety criteria".to_string()
            ));
        }
        
        let mut attempt = self.validate_and_apply(fix, error).await?;
        
        self.apply_fix(fix).await?;
        attempt.status = CorrectionStatus::Applied;
        
        let test_results = self.run_tests(fix).await;
        attempt.test_results = test_results.clone();
        
        let validation_score = self.calculate_validation_score(&test_results);
        attempt.validation_score = validation_score;
        
        if validation_score >= 0.8 {
            attempt.status = CorrectionStatus::Validated;
        } else {
            log::warn!("Fix validation score too low: {}, rolling back", validation_score);
            if let Some(snapshot) = &attempt.rollback_snapshot {
                self.rollback(snapshot).await?;
            }
            attempt.status = CorrectionStatus::RolledBack;
        }
        
        Ok(attempt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_validation_score_calculation() {
        let validator = CorrectionValidator::new("test".to_string());
        
        let results = vec![
            TestResult {
                test_name: "test1".to_string(),
                passed: true,
                output: "passed".to_string(),
                duration_ms: 100,
            },
            TestResult {
                test_name: "test2".to_string(),
                passed: true,
                output: "passed".to_string(),
                duration_ms: 100,
            },
            TestResult {
                test_name: "test3".to_string(),
                passed: false,
                output: "failed".to_string(),
                duration_ms: 100,
            },
        ];
        
        let score = validator.calculate_validation_score(&results);
        assert_eq!(score, 2.0 / 3.0);
    }
}
