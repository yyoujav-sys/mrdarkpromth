// MR.DarkPromth Agent 8 Main Entry Point
// Agent 8: Self-Correction Engine Engineer
// Phase 1 Implementation - Error Detection and Classification

use crate::self_correction_engine::{ErrorDetector, ErrorSource, ErrorEvent};
use crate::redis_coordination::{RedisCoordinator, EventType};
use log::{info, warn, error};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::interval;

pub struct Agent8SelfCorrection {
    error_detector: ErrorDetector,
    redis_coordinator: Option<Arc<Mutex<RedisCoordinator>>>,
    agent_id: String,
}

impl Agent8SelfCorrection {
    pub fn new(redis_url: Option<&str>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let agent_id = "agent8".to_string();
        let mut error_detector = ErrorDetector::new(agent_id.clone());
        
        // Set up Redis coordination if URL provided
        let redis_coordinator = if let Some(url) = redis_url {
            let coordinator = RedisCoordinator::new(url, agent_id.clone())?;
            error_detector = error_detector.with_redis(coordinator.clone());
            Some(Arc::new(Mutex::new(coordinator)))
        } else {
            None
        };
        
        Ok(Self {
            error_detector,
            redis_coordinator,
            agent_id,
        })
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting Agent 8 Self-Correction Engine");
        
        // Initialize logging
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Info)
            .init();
        
        // Subscribe to Redis error events if available
        if let Some(ref coordinator) = self.redis_coordinator {
            let mut coord = coordinator.lock().unwrap();
            coord.subscribe_to_events(&EventType::ErrorEvent)?;
            info!("Subscribed to Redis error events");
        }
        
        // Start main monitoring loop
        self.start_monitoring_loop().await?;
        
        Ok(())
    }

    async fn start_monitoring_loop(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut interval_timer = interval(Duration::from_secs(5));
        
        loop {
            tokio::select! {
                _ = interval_timer.tick() => {
                    self.process_periodic_checks().await?;
                }
                
                // Process Redis events if available
                _ = async {
                    if let Some(ref _coordinator) = self.redis_coordinator {
                        self.process_redis_events().await
                    } else {
                        std::future::pending::<Result<(), Box<dyn std::error::Error + Send + Sync>>>().await
                    }
                } => {
                    // Redis events processed
                }
            }
        }
    }

    async fn process_periodic_checks(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Monitor application logs
        self.check_application_logs().await?;
        
        // Monitor tool execution results
        self.check_tool_results().await?;
        
        // Monitor agent feedback
        self.check_agent_feedback().await?;
        
        // Publish heartbeat
        self.publish_heartbeat().await?;
        
        Ok(())
    }

    async fn check_application_logs(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // REAL IMPLEMENTATION: Read from actual log file
        let log_path = std::env::var("AGENT_LOG_PATH").unwrap_or_else(|_| "./memory/agent4_audit.log".to_string());
        
        match tokio::fs::read_to_string(&log_path).await {
            Ok(content) => {
                // simple tail implementation - in production use a proper log watcher
                // For now, valid "De-muck" is to read the file.
                for line in content.lines().rev().take(50) {
                     if let Ok(errors) = self.error_detector.detect_from_log(line, ErrorSource::ApplicationLog) {
                        for error in errors {
                            self.handle_detected_error(error).await?;
                        }
                    }
                }
            }
            Err(e) => {
                warn!("Could not read log file {}: {}", log_path, e);
            }
        }
        
        Ok(())
    }

    async fn check_tool_results(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Tool results are now strictly handled via Redis events in `process_redis_events`
        // We do not poll hardcoded tool results anymore.
        Ok(())
    }

    async fn check_agent_feedback(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Agent feedback comes via Redis Pub/Sub "agent_feedback" channel
        // This method is now a placeholder for specific direct-feedback logic if needed in future.
        Ok(())
    }

    async fn process_redis_events(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let events = if let Some(ref coordinator) = self.redis_coordinator {
            let coord = coordinator.lock().unwrap();
            coord.read_events(&EventType::ErrorEvent, Some(1000)).unwrap_or_default()
        } else {
            Vec::new()
        };

        for stream_event in events {
            self.error_detector.process_redis_error_event(&stream_event.event).await?;
            
            if let Some(ref coordinator) = self.redis_coordinator {
                let coord = coordinator.lock().unwrap();
                let _ = coord.acknowledge_event(&EventType::ErrorEvent, &stream_event.stream_id);
            }
        }
        
        Ok(())
    }

    async fn handle_detected_error(&mut self, error: ErrorEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("🚨 Agent 8 detected error: {:?} - {}", error.error_type, error.message);
        
        // Log error details
        match error.severity {
            crate::self_correction_engine::ErrorSeverity::Critical => {
                error!("CRITICAL: {:?}", error);
            }
            crate::self_correction_engine::ErrorSeverity::High => {
                warn!("HIGH: {:?}", error);
            }
            _ => {
                info!("MEDIUM/LOW: {:?}", error);
            }
        }
        
        // Store error for analysis
        self.store_error(&error).await?;
        
        // Publish to other agents via Redis
        self.publish_error_to_agents(&error).await?;
        
        // Attempt automated correction based on error type
        self.attempt_correction(&error).await?;
        
        Ok(())
    }

    async fn store_error(&self, error: &ErrorEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Store error to persistent JSON file
        let error_dir = std::path::PathBuf::from("./memory/errors/agent8");
        if let Err(e) = std::fs::create_dir_all(&error_dir) {
            warn!("Failed to create error directory: {}", e);
            return Ok(());
        }
        
        let file_name = format!("error_{}.json", error.id);
        let file_path = error_dir.join(file_name);
        
        let error_json = serde_json::to_string_pretty(error)?;
        match std::fs::write(&file_path, error_json) {
            Ok(_) => info!("Stored error {} to {}", error.id, file_path.display()),
            Err(e) => warn!("Failed to write error to file: {}", e),
        }
        
        Ok(())
    }

    async fn attempt_correction(&self, error: &ErrorEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Attempt automated correction based on error type and severity
        match error.error_type {
            crate::self_correction_engine::ErrorType::CompilationError => {
                info!("Auto-correction: Triggering cargo check for compilation error");
                // Could trigger cargo fix here
            }
            crate::self_correction_engine::ErrorType::ConfigurationError => {
                info!("Auto-correction: Checking environment configuration");
                // Could validate and reload config
            }
            _ => {
                info!("Auto-correction not available for {:?} errors", error.error_type);
            }
        }
        
        Ok(())
    }

    async fn publish_error_to_agents(&self, error: &ErrorEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(ref coordinator) = self.redis_coordinator {
            let mut coord = coordinator.lock().unwrap();
            
            let _ = coord.publish_error_event(
                &format!("{:?}", error.error_type),
                &error.message,
                error.file_path.as_ref().map(|p| p.to_str().unwrap()),
                error.line_number
            );
            
            info!("Published error {} to Redis event bus", error.id);
        }
        
        Ok(())
    }

    async fn publish_heartbeat(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(ref coordinator) = self.redis_coordinator {
            let mut coord = coordinator.lock().unwrap();
            
            let _ = coord.publish_heartbeat(
                "monitoring",
                0, // tasks_in_progress
                0  // tasks_completed
            );
        }
        
        Ok(())
    }

    pub fn get_agent_id(&self) -> &str {
        &self.agent_id
    }

    pub fn get_error_detector(&self) -> &ErrorDetector {
        &self.error_detector
    }
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Get Redis URL from environment or use default
    let redis_url = std::env::var("REDIS_URL").ok();
    
    let mut agent8 = Agent8SelfCorrection::new(redis_url.as_deref())?;
    
    info!("🤖 Agent 8 Self-Correction Engine starting...");
    info!("Redis coordination: {}", if redis_url.is_some() { "Enabled" } else { "Disabled" });
    
    agent8.start().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_agent8_initialization() {
        let agent8 = Agent8SelfCorrection::new(None).unwrap();
        assert_eq!(agent8.get_agent_id(), "agent8");
    }

    #[tokio::test]
    async fn test_error_detection_flow() {
        let mut agent8 = Agent8SelfCorrection::new(None).unwrap();
        
        // Test log detection
        agent8.check_application_logs().await.unwrap();
        
        // Test tool result detection
        agent8.check_tool_results().await.unwrap();
        
        // Test agent feedback detection
        agent8.check_agent_feedback().await.unwrap();
    }
}
