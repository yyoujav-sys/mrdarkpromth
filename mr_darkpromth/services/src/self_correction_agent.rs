use crate::agent_framework::{Agent, AgentError, AgentMessage, AgentResult, AgentState, AgentType};
use crate::correction_validator::CorrectionValidator;
use crate::error_detector::{ErrorCategory, ErrorDetection, ErrorDetector, ErrorSeverity, ErrorSource};
use crate::fix_generator::{FixGenerator, GeneratedFix};
use crate::learning_system::LearningSystem;
use crate::redis_coordination::RedisCoordinator;
use cerebras_client::CerebrasClient;
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct SelfCorrectionAgent {
    id: String,
    state: AgentState,
    redis_coordinator: Arc<RedisCoordinator>,
    error_detector: ErrorDetector,
    fix_generator: FixGenerator,
    correction_validator: CorrectionValidator,
    learning_system: Arc<Mutex<LearningSystem>>,
    project_root: String,
    #[allow(dead_code)]
    confidence_threshold: f64,
}

impl SelfCorrectionAgent {
    pub fn new(
        redis_url: &str,
        cerebras_client: CerebrasClient,
        db_pool: PgPool,
        project_root: String,
        confidence_threshold: f64,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let id = format!("self-correction-{}", uuid::Uuid::new_v4());
        Ok(Self {
            id: id.clone(),
            state: AgentState::Idle,
            redis_coordinator: Arc::new(RedisCoordinator::new(redis_url, "self-correction".to_string())?),
            error_detector: ErrorDetector::new()
                .map_err(|e| AgentError::MessageProcessingFailed(e.to_string()))?,
            fix_generator: FixGenerator::new(cerebras_client, confidence_threshold),
            correction_validator: CorrectionValidator::new(project_root.clone()),
            learning_system: Arc::new(Mutex::new(LearningSystem::new(db_pool))),
            project_root,
            confidence_threshold,
        })
    }

    pub async fn start(&mut self) -> AgentResult<()> {
        log::info!("Self-Correction Agent starting up");
        self.state = AgentState::Idle;
        
        self.redis_coordinator.subscribe("self-correction").await
            .map_err(|e| AgentError::CommunicationError(e.to_string()))?;
        
        self.publish_status("ready").await?;
        
        Ok(())
    }

    async fn publish_status(&self, status: &str) -> AgentResult<()> {
        let message = serde_json::json!({
            "agent_id": self.id,
            "agent_type": "self_correction",
            "status": status,
            "timestamp": chrono::Utc::now(),
        });
        
        self.redis_coordinator.publish("agent-status", &message.to_string()).await
            .map_err(|e| AgentError::CommunicationError(e.to_string()))?;
        
        Ok(())
    }

    async fn handle_error_detection(&mut self, log_message: &str, source: ErrorSource) -> AgentResult<()> {
        log::info!("Detecting errors from log: {}", log_message);
        
        let detections = self.error_detector.detect_from_log(log_message, &source);
        
        for detection in &detections {
            self.process_error_detection(detection).await?;
        }
        
        Ok(())
    }

    async fn process_error_detection(&mut self, detection: &ErrorDetection) -> AgentResult<()> {
        log::info!("Processing error detection: {:?}", detection.category);
        
        if detection.severity == ErrorSeverity::Critical {
            log::warn!("Critical error detected, requiring immediate attention: {}", detection.message);
            self.notify_critical_error(detection).await?;
        }
        
        if self.should_auto_correct(detection) {
            self.attempt_auto_correction(detection).await?;
        }
        
        Ok(())
    }

    fn should_auto_correct(&self, detection: &ErrorDetection) -> bool {
        match detection.severity {
            ErrorSeverity::Low => true,
            ErrorSeverity::Medium => true,
            ErrorSeverity::High => detection.category == ErrorCategory::SyntaxError,
            ErrorSeverity::Critical => false,
        }
    }

    async fn attempt_auto_correction(&mut self, detection: &ErrorDetection) -> AgentResult<()> {
        log::info!("Attempting auto-correction for error: {}", detection.message);
        
        let code_context = self.get_code_context(detection).await?;
        
        let fixes = self.fix_generator.generate_multiple_fixes(detection, &code_context, 3).await;
        
        if let Some(best_fix) = self.fix_generator.select_best_fix(&fixes) {
            if self.fix_generator.should_apply_fix(best_fix) {
                self.apply_fix(best_fix, detection).await?;
            }
        }
        
        Ok(())
    }

    async fn get_code_context(&self, detection: &ErrorDetection) -> AgentResult<String> {
        if let Some(file_path) = &detection.context.file_path {
            let full_path = std::path::Path::new(&self.project_root).join(file_path);
            
            if full_path.exists() {
                return std::fs::read_to_string(&full_path)
                    .map_err(|e| AgentError::MessageProcessingFailed(e.to_string()));
            }
        }
        
        Ok(String::new())
    }

    async fn apply_fix(&mut self, fix: &GeneratedFix, detection: &ErrorDetection) -> AgentResult<()> {
        log::info!("Applying fix: {}", fix.description);
        
        match self.correction_validator.validate_fix(fix, detection).await {
            Ok(attempt) => {
                let mut learning = self.learning_system.lock().await;
                learning.record_correction(&attempt, fix, detection).await
                    .map_err(|e| AgentError::MessageProcessingFailed(e.to_string()))?;
                
                match attempt.status {
                    crate::correction_validator::CorrectionStatus::Validated => {
                        log::info!("Fix successfully validated and applied");
                        self.notify_fix_success(fix, detection).await?;
                    }
                    crate::correction_validator::CorrectionStatus::RolledBack => {
                        log::warn!("Fix was rolled back due to validation failure");
                        self.notify_fix_rolled_back(fix, detection).await?;
                    }
                    _ => {
                        log::warn!("Fix application failed");
                    }
                }
                
                Ok(())
            }
            Err(e) => {
                log::error!("Failed to apply fix: {}", e);
                self.notify_fix_failed(fix, detection, &e.to_string()).await?;
                Err(AgentError::ActionExecutionFailed(e.to_string()))
            }
        }
    }

    async fn notify_critical_error(&self, detection: &ErrorDetection) -> AgentResult<()> {
        let message = json!({
            "type": "critical_error",
            "error_id": detection.id,
            "category": format!("{:?}", detection.category),
            "message": detection.message,
            "severity": format!("{:?}", detection.severity),
            "timestamp": detection.timestamp,
        });
        
        self.redis_coordinator.publish("critical-alerts", &message.to_string()).await
            .map_err(|e| AgentError::CommunicationError(e.to_string()))?;
        
        Ok(())
    }

    async fn notify_fix_success(&self, fix: &GeneratedFix, detection: &ErrorDetection) -> AgentResult<()> {
        let message = json!({
            "type": "fix_applied",
            "fix_id": fix.id,
            "error_id": detection.id,
            "description": fix.description,
            "confidence": fix.confidence_score,
            "timestamp": chrono::Utc::now(),
        });
        
        self.redis_coordinator.publish("correction-updates", &message.to_string()).await
            .map_err(|e| AgentError::CommunicationError(e.to_string()))?;
        
        Ok(())
    }

    async fn notify_fix_rolled_back(&self, fix: &GeneratedFix, detection: &ErrorDetection) -> AgentResult<()> {
        let message = json!({
            "type": "fix_rolled_back",
            "fix_id": fix.id,
            "error_id": detection.id,
            "description": fix.description,
            "reason": "validation_failed",
            "timestamp": chrono::Utc::now(),
        });
        
        self.redis_coordinator.publish("correction-updates", &message.to_string()).await
            .map_err(|e| AgentError::CommunicationError(e.to_string()))?;
        
        Ok(())
    }

    async fn notify_fix_failed(&self, fix: &GeneratedFix, detection: &ErrorDetection, error: &str) -> AgentResult<()> {
        let message = json!({
            "type": "fix_failed",
            "fix_id": fix.id,
            "error_id": detection.id,
            "description": fix.description,
            "error": error,
            "timestamp": chrono::Utc::now(),
        });
        
        self.redis_coordinator.publish("correction-updates", &message.to_string()).await
            .map_err(|e| AgentError::CommunicationError(e.to_string()))?;
        
        Ok(())
    }

    pub async fn get_insights(&self) -> AgentResult<String> {
        let learning = self.learning_system.lock().await;
        learning.get_insights().await
            .map_err(|e| AgentError::MessageProcessingFailed(e.to_string()))
    }

    pub async fn get_metrics(&self) -> AgentResult<serde_json::Value> {
        let learning = self.learning_system.lock().await;
        let metrics = learning.get_metrics();
        
        Ok(json!({
            "total_corrections": metrics.total_corrections,
            "successful_corrections": metrics.successful_corrections,
            "failed_corrections": metrics.failed_corrections,
            "success_rate": if metrics.total_corrections > 0 {
                (metrics.successful_corrections as f64 / metrics.total_corrections as f64) * 100.0
            } else {
                0.0
            },
            "average_confidence": metrics.average_confidence,
            "average_validation_score": metrics.average_validation_score,
            "most_common_errors": metrics.most_common_errors.iter()
                .map(|(cat, count)| json!({
                    "category": format!("{:?}", cat),
                    "count": count
                }))
                .collect::<Vec<_>>(),
        }))
    }
}

impl Agent for SelfCorrectionAgent {
    fn agent_id(&self) -> &str {
        &self.id
    }

    fn agent_type(&self) -> AgentType {
        AgentType::Coordinator
    }

    fn state(&self) -> AgentState {
        self.state.clone()
    }

    fn handle_message(&mut self, message: AgentMessage) -> AgentResult<()> {
        log::info!("Self-Correction Agent received message: {}", message.message_type);
        
        match message.message_type.as_str() {
            "error_detected" => {
                if let (Some(log_message), Some(source_str)) = (
                    message.content["log_message"].as_str(),
                    message.content["source"].as_str()
                ) {
                    let source = match source_str {
                        "application_log" => ErrorSource::ApplicationLog,
                        "tool_execution" => ErrorSource::ToolExecution,
                        "agent_feedback" => ErrorSource::AgentFeedback,
                        "system_monitor" => ErrorSource::SystemMonitor,
                        _ => ErrorSource::ApplicationLog,
                    };
                    
                    let rt = tokio::runtime::Runtime::new()
                        .map_err(|e| AgentError::MessageProcessingFailed(e.to_string()))?;
                    
                    rt.block_on(self.handle_error_detection(log_message, source))?;
                }
            }
            "request_insights" => {
                let rt = tokio::runtime::Runtime::new()
                    .map_err(|e| AgentError::MessageProcessingFailed(e.to_string()))?;
                
                let insights = rt.block_on(self.get_insights())?;
                log::info!("Self-Correction Insights:\n{}", insights);
            }
            "request_metrics" => {
                let rt = tokio::runtime::Runtime::new()
                    .map_err(|e| AgentError::MessageProcessingFailed(e.to_string()))?;
                
                if let Ok(metrics) = rt.block_on(self.get_metrics()) {
                    log::info!("Self-Correction Metrics: {}", metrics);
                }
            }
            _ => {
                log::warn!("Unknown message type: {}", message.message_type);
            }
        }
        
        Ok(())
    }

    fn process(&mut self) -> AgentResult<()> {
        log::info!("Self-Correction Agent processing");
        Ok(())
    }

    fn shutdown(&mut self) -> AgentResult<()> {
        log::info!("Self-Correction Agent shutting down");
        self.state = AgentState::Shutdown;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_db_utils::create_test_pool;

    #[tokio::test]
    async fn test_agent_creation() {
        let redis_url = "redis://localhost:6379";
        let config = cerebras_client::CerebrasConfig::new(vec!["test_key".to_string()])
            .expect("config");
        let cerebras_client = CerebrasClient::new(config);
        let db_pool = create_test_pool().await.unwrap();
        let project_root = "/tmp/test".to_string();
        
        let agent = SelfCorrectionAgent::new(
            redis_url,
            cerebras_client,
            db_pool,
            project_root,
            0.7
        );
        
        assert!(agent.is_ok());
    }
}
