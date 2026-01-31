// MR.DarkPromth Agent 8 Main Entry Point
// Agent 8: Self-Correction Engine Engineer
// Phase 1 Implementation - Error Detection and Classification

use crate::self_correction_engine::{ErrorDetector, ErrorSource, ErrorEvent};
use crate::redis_coordination::{RedisCoordinator, EventType};
use chrono::Utc;
use log::{info, warn, error};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::interval;
use uuid::Uuid;

pub struct Agent8SelfCorrection {
    error_detector: ErrorDetector,
    redis_coordinator: Option<Arc<Mutex<RedisCoordinator>>>,
    agent_id: String,
}

impl Agent8SelfCorrection {
    pub fn new(redis_url: Option<&str>) -> Result<Self, Box<dyn std::error::Error>> {
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

    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
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

    async fn start_monitoring_loop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut interval_timer = interval(Duration::from_secs(5));
        
        loop {
            tokio::select! {
                _ = interval_timer.tick() => {
                    self.process_periodic_checks().await?;
                }
                
                // Process Redis events if available
                _ = async {
                    if let Some(ref coordinator) = self.redis_coordinator {
                        self.process_redis_events().await
                    } else {
                        std::future::pending::<Result<(), Box<dyn std::error::Error>>>().await
                    }
                } => {
                    // Redis events processed
                }
            }
        }
    }

    async fn process_periodic_checks(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Simulate checking application logs
        self.check_application_logs().await?;
        
        // Simulate checking tool execution results
        self.check_tool_results().await?;
        
        // Simulate checking agent feedback
        self.check_agent_feedback().await?;
        
        // Publish heartbeat
        self.publish_heartbeat().await?;
        
        Ok(())
    }

    async fn check_application_logs(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Simulate log entries (in real implementation, would read from log files)
        let sample_logs = vec![
            "error[E0277]: cannot multiply f64 by f32",
            "warning: unused variable: `x`",
            "info: Processing request completed successfully",
            "panic! at 'assertion failed: x > 0'",
        ];
        
        for log_line in sample_logs {
            if let Ok(errors) = self.error_detector.detect_from_log(log_line, ErrorSource::ApplicationLog) {
                for error in errors {
                    self.handle_detected_error(error).await?;
                }
            }
        }
        
        Ok(())
    }

    async fn check_tool_results(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Simulate tool execution results
        let sample_tools = vec![
            ("cargo", "error: cannot find Cargo.toml", 101),
            ("python", "SyntaxError: invalid syntax", 1),
            ("redis-cli", "Connection refused", 1),
            ("git", "fatal: not a git repository", 128),
        ];
        
        for (tool_name, result, exit_code) in sample_tools {
            if let Ok(Some(error)) = self.error_detector.detect_from_tool_result(tool_name, result, exit_code) {
                self.handle_detected_error(error).await?;
            }
        }
        
        Ok(())
    }

    async fn check_agent_feedback(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Simulate agent feedback
        let sample_feedback = vec![
            ("agent3", "Failed to process request due to timeout"),
            ("agent4", "Jailbreak system encountered an error"),
            ("agent5", "User authentication failed for user123"),
        ];
        
        for (agent_id, feedback) in sample_feedback {
            if let Ok(errors) = self.error_detector.detect_from_agent_feedback(agent_id, feedback) {
                for error in errors {
                    self.handle_detected_error(error).await?;
                }
            }
        }
        
        Ok(())
    }

    async fn process_redis_events(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref coordinator) = self.redis_coordinator {
            let mut coord = coordinator.lock().unwrap();
            
            if let Ok(events) = coord.read_events(&EventType::ErrorEvent, Some(1000)) {
                for stream_event in events {
                    self.error_detector.process_redis_error_event(&stream_event.event).await?;
                    let _ = coord.acknowledge_event(&EventType::ErrorEvent, &stream_event.stream_id);
                }
            }
        }
        
        Ok(())
    }

    async fn handle_detected_error(&mut self, error: ErrorEvent) -> Result<(), Box<dyn std::error::Error>> {
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
        
        // Store error for analysis (Phase 4)
        self.store_error(&error).await?;
        
        // Publish to other agents via Redis
        self.publish_error_to_agents(&error).await?;
        
        // TODO: Phase 2 - Generate automated fixes
        // TODO: Phase 3 - Apply and validate fixes
        // TODO: Phase 4 - Learn from corrections
        
        Ok(())
    }

    async fn store_error(&self, error: &ErrorEvent) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: Implement persistent storage in Phase 4
        info!("Storing error {} for analysis", error.id);
        Ok(())
    }

    async fn publish_error_to_agents(&self, error: &ErrorEvent) -> Result<(), Box<dyn std::error::Error>> {
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

    async fn publish_heartbeat(&self) -> Result<(), Box<dyn std::error::Error>> {
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
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
