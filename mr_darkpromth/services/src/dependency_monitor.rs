// MR.DarkPromth Dependency Monitor
// Agent 4: Jailbreak & Ultra Tier Engineer
// Active Monitoring & Dependency Integration Phase

use crate::{RedisCoordinator, CoordinationEvent, EventType};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::io::Write;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyStatus {
    pub agent_id: String,
    pub name: String,
    pub status: DependencyState,
    pub last_check: DateTime<Utc>,
    pub last_heartbeat: Option<DateTime<Utc>>,
    pub tasks_completed: u32,
    pub current_phase: String,
    pub blocking_dependencies: Vec<String>,
    pub ready_for_integration: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DependencyState {
    Unknown,
    Starting,
    InProgress,
    Ready,
    Error,
    Completed,
}

pub struct DependencyMonitor {
    redis_coordinator: std::sync::Arc<std::sync::Mutex<RedisCoordinator>>,
    dependencies: HashMap<String, DependencyStatus>,
    monitoring_start: Instant,
    check_interval: Duration,
}

impl DependencyMonitor {
    pub fn new(redis_coordinator: RedisCoordinator) -> Self {
        let mut monitor = Self {
            redis_coordinator: std::sync::Arc::new(std::sync::Mutex::new(redis_coordinator)),
            dependencies: HashMap::new(),
            monitoring_start: Instant::now(),
            check_interval: Duration::from_secs(30),
        };
        
        // Initialize dependency tracking
        monitor.initialize_dependencies();
        monitor
    }

    fn initialize_dependencies(&mut self) {
        // Agent 3: Cerebras.ai Integration Specialist
        self.dependencies.insert("agent3".to_string(), DependencyStatus {
            agent_id: "agent3".to_string(),
            name: "Cerebras.ai Integration Specialist".to_string(),
            status: DependencyState::Unknown,
            last_check: Utc::now(),
            last_heartbeat: None,
            tasks_completed: 0,
            current_phase: "Phase 1: Cerebras.ai Client Operational".to_string(),
            blocking_dependencies: vec![],
            ready_for_integration: false,
        });

        // Agent 5: User Management & Authentication Engineer
        self.dependencies.insert("agent5".to_string(), DependencyStatus {
            agent_id: "agent5".to_string(),
            name: "User Management & Authentication Engineer".to_string(),
            status: DependencyState::Unknown,
            last_check: Utc::now(),
            last_heartbeat: None,
            tasks_completed: 0,
            current_phase: "Phase 1: User Authentication and Authorization".to_string(),
            blocking_dependencies: vec![],
            ready_for_integration: false,
        });
    }

    pub async fn start_monitoring(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔍 Agent 4: Starting dependency monitoring...");
        
        // Subscribe to global events
        {
            let mut coordinator = self.redis_coordinator.lock().unwrap();
            coordinator.subscribe_to_events(&EventType::TaskCompletion)?;
            coordinator.subscribe_to_events(&EventType::ResourceReady)?;
            coordinator.subscribe_to_events(&EventType::Heartbeat)?;
        }

        // Initial status check
        self.check_all_dependencies().await?;
        
        // Start monitoring loop
        self.monitoring_loop().await
    }

    async fn monitoring_loop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            // Check for new events
            self.check_for_events().await?;
            
            // Update dependency status
            self.check_all_dependencies().await?;
            
            // Log current status
            self.log_dependency_status();
            
            // Check if all dependencies are ready
            if self.all_dependencies_ready() {
                println!("🎉 All dependencies are ready! Agent 4 can proceed with full integration.");
                self.log_memory("DEPENDENCIES_READY", "All dependencies ready for integration", "SUCCESS", None);
                break;
            }
            
            // Wait for next check
            tokio::time::sleep(self.check_interval).await;
        }
        
        Ok(())
    }

    async fn check_for_events(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let (task_events, resource_events, heartbeat_events) = {
            let coordinator = self.redis_coordinator.lock().unwrap();
            let task_events = coordinator.read_events(&EventType::TaskCompletion, Some(1000)).unwrap_or_default();
            let resource_events = coordinator.read_events(&EventType::ResourceReady, Some(1000)).unwrap_or_default();
            let heartbeat_events = coordinator.read_events(&EventType::Heartbeat, Some(1000)).unwrap_or_default();
            (task_events, resource_events, heartbeat_events)
        };

        // Handle events first
        for stream_event in &task_events {
            self.handle_task_completion_event(&stream_event.event);
        }

        for stream_event in &resource_events {
            self.handle_resource_ready_event(&stream_event.event);
        }

        for stream_event in &heartbeat_events {
            self.handle_heartbeat_event(&stream_event.event);
        }

        // Acknowledge events in separate scope
        {
            let coordinator = self.redis_coordinator.lock().unwrap();
            for stream_event in task_events {
                let _ = coordinator.acknowledge_event(&EventType::TaskCompletion, &stream_event.stream_id);
            }
            for stream_event in resource_events {
                let _ = coordinator.acknowledge_event(&EventType::ResourceReady, &stream_event.stream_id);
            }
            for stream_event in heartbeat_events {
                let _ = coordinator.acknowledge_event(&EventType::Heartbeat, &stream_event.stream_id);
            }
        }

        Ok(())
    }

    fn handle_task_completion_event(&mut self, event: &CoordinationEvent) {
        if let Some(agent_id) = event.payload.get("agent_id").and_then(|a| a.as_str()) {
            if let Some(dependency) = self.dependencies.get_mut(agent_id) {
                dependency.tasks_completed += 1;
                dependency.last_check = event.timestamp;
                
                // Update status based on task
                if let Some(task) = event.payload.get("task").and_then(|t| t.as_str()) {
                    match task {
                        "cerebras_client_operational" => {
                            dependency.status = DependencyState::Ready;
                            dependency.ready_for_integration = true;
                            dependency.current_phase = "Phase 1 Complete".to_string();
                        }
                        "user_authentication_system" => {
                            dependency.status = DependencyState::Ready;
                            dependency.ready_for_integration = true;
                            dependency.current_phase = "Phase 1 Complete".to_string();
                        }
                        _ => {
                            dependency.status = DependencyState::InProgress;
                        }
                    }
                }
                
                println!("📊 Agent 4: Task completion from {}: {:?}", agent_id, event.payload.get("task"));
            }
        }
    }

    fn handle_resource_ready_event(&mut self, event: &CoordinationEvent) {
        if let Some(agent_id) = event.payload.get("agent_id").and_then(|a| a.as_str()) {
            if let Some(dependency) = self.dependencies.get_mut(agent_id) {
                dependency.status = DependencyState::Ready;
                dependency.ready_for_integration = true;
                dependency.last_check = event.timestamp;
                
                println!("🚀 Agent 4: Resource ready from {}: {:?}", agent_id, event.payload.get("resource"));
            }
        }
    }

    fn handle_heartbeat_event(&mut self, event: &CoordinationEvent) {
        if let Some(agent_id) = event.payload.get("agent_id").and_then(|a| a.as_str()) {
            if let Some(dependency) = self.dependencies.get_mut(agent_id) {
                dependency.last_heartbeat = Some(event.timestamp);
                dependency.last_check = event.timestamp;
                
                if dependency.status == DependencyState::Unknown {
                    dependency.status = DependencyState::InProgress;
                }
            }
        }
    }

    async fn check_all_dependencies(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let agent_ids: Vec<String> = self.dependencies.keys().cloned().collect();

        for agent_id in agent_ids {
            // Query agent status
            let correlation_id = self.query_agent_status(&agent_id).await?;

            // Wait for response (with timeout)
            let response = self.wait_for_status_response(&correlation_id, 5000).await;

            if let Some(dependency) = self.dependencies.get_mut(&agent_id) {
                if let Some(response) = response {
                    Self::update_dependency_from_response(dependency, &response);
                } else {
                    // No response, mark as unknown
                    dependency.status = DependencyState::Unknown;
                    dependency.last_check = Utc::now();
                }
            }
        }

        Ok(())
    }

    async fn query_agent_status(&self, agent_id: &str) -> Result<String, Box<dyn std::error::Error>> {
        let mut coordinator = self.redis_coordinator.lock().unwrap();
        
        let query_payload = serde_json::json!({
            "query": "status_check",
            "requester": "agent4",
            "timestamp": Utc::now()
        });
        
        let correlation_id = coordinator.publish_query_event(agent_id, &query_payload.to_string())?;
        Ok(correlation_id)
    }

    async fn wait_for_status_response(&self, correlation_id: &str, timeout_ms: u64) -> Option<serde_json::Value> {
        let start_time = std::time::Instant::now();
        
        while start_time.elapsed().as_millis() < timeout_ms as u128 {
            if let Ok(coordinator) = self.redis_coordinator.try_lock() {
                if let Ok(events) = coordinator.read_events(&EventType::ResponseEvent, Some(100)) {
                    for stream_event in events {
                        if let Some(event_correlation_id) = &stream_event.event.correlation_id {
                            if event_correlation_id == correlation_id {
                                let _ = coordinator.acknowledge_event(&EventType::ResponseEvent, &stream_event.stream_id);
                                return Some(stream_event.event.payload);
                            }
                        }
                    }
                }
            }
            
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        
        None
    }

    fn update_dependency_from_response(dependency: &mut DependencyStatus, response: &serde_json::Value) {
        dependency.last_check = Utc::now();
        
        if let Some(status) = response.get("status").and_then(|s| s.as_str()) {
            match status {
                "ready" => {
                    dependency.status = DependencyState::Ready;
                    dependency.ready_for_integration = true;
                }
                "in_progress" => {
                    dependency.status = DependencyState::InProgress;
                }
                "error" => {
                    dependency.status = DependencyState::Error;
                }
                "completed" => {
                    dependency.status = DependencyState::Completed;
                    dependency.ready_for_integration = true;
                }
                _ => {
                    dependency.status = DependencyState::Unknown;
                }
            }
        }
        
        if let Some(phase) = response.get("current_phase").and_then(|p| p.as_str()) {
            dependency.current_phase = phase.to_string();
        }
        
        if let Some(tasks) = response.get("tasks_completed").and_then(|t| t.as_u64()) {
            dependency.tasks_completed = tasks as u32;
        }
    }

    fn log_dependency_status(&self) {
        let monitoring_duration = self.monitoring_start.elapsed();
        
        println!("\n📋 Agent 4 Dependency Status Report");
        println!("=====================================");
        println!("Monitoring Duration: {:?}", monitoring_duration);
        println!("Last Update: {}", Utc::now().format("%Y-%m-%d %H:%M:%S UTC"));
        println!();
        
        for (agent_id, dependency) in &self.dependencies {
            println!("🤖 {}: {}", dependency.name, agent_id);
            println!("   Status: {:?}", dependency.status);
            println!("   Phase: {}", dependency.current_phase);
            println!("   Tasks Completed: {}", dependency.tasks_completed);
            println!("   Ready for Integration: {}", dependency.ready_for_integration);
            
            if let Some(heartbeat) = dependency.last_heartbeat {
                let heartbeat_age = Utc::now().signed_duration_since(heartbeat);
                println!("   Last Heartbeat: {} ({})", 
                    heartbeat.format("%H:%M:%S"), 
                    heartbeat_age.num_seconds()
                );
            } else {
                println!("   Last Heartbeat: Never");
            }
            
            println!();
        }
        
        let ready_count = self.dependencies.values().filter(|d| d.ready_for_integration).count();
        let total_count = self.dependencies.len();
        
        println!("📊 Summary: {}/{} dependencies ready for integration", ready_count, total_count);
        
        if ready_count == total_count {
            println!("🎉 All dependencies are ready!");
        } else {
            println!("⏳ Waiting for {} more dependencies...", total_count - ready_count);
        }
        
        println!("=====================================\n");
    }

    pub fn all_dependencies_ready(&self) -> bool {
        self.dependencies.values().all(|d| d.ready_for_integration)
    }

    pub fn get_dependency_status(&self, agent_id: &str) -> Option<&DependencyStatus> {
        self.dependencies.get(agent_id)
    }

    pub fn get_all_dependencies(&self) -> &HashMap<String, DependencyStatus> {
        &self.dependencies
    }

    pub fn is_agent3_ready(&self) -> bool {
        self.dependencies
            .get("agent3")
            .map(|d| d.ready_for_integration)
            .unwrap_or(false)
    }

    pub fn is_agent5_ready(&self) -> bool {
        self.dependencies
            .get("agent5")
            .map(|d| d.ready_for_integration)
            .unwrap_or(false)
    }

    pub fn get_monitoring_summary(&self) -> serde_json::Value {
        let dependencies: Vec<serde_json::Value> = self.dependencies
            .values()
            .map(|d| {
                serde_json::json!({
                    "agent_id": d.agent_id,
                    "name": d.name,
                    "status": format!("{:?}", d.status),
                    "current_phase": d.current_phase,
                    "tasks_completed": d.tasks_completed,
                    "ready_for_integration": d.ready_for_integration,
                    "last_check": d.last_check,
                    "last_heartbeat": d.last_heartbeat
                })
            })
            .collect();

        serde_json::json!({
            "monitoring_duration_seconds": self.monitoring_start.elapsed().as_secs(),
            "total_dependencies": self.dependencies.len(),
            "ready_dependencies": self.dependencies.values().filter(|d| d.ready_for_integration).count(),
            "all_ready": self.all_dependencies_ready(),
            "dependencies": dependencies,
            "last_update": Utc::now()
        })
    }

    fn log_memory(&self, action: &str, details: &str, status: &str, metadata: Option<serde_json::Value>) {
        let timestamp = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ");
        let log_entry = format!("[{}] [{}] [{}] [{}]", timestamp, action, details, status);
        
        if let Some(meta) = metadata {
            let meta_str = serde_json::to_string(&meta).unwrap_or_default();
            let full_entry = format!("{} {}", log_entry, meta_str);
            
            // Append to memory log file
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("./memory/agent4_jailbreak_ultra.log") {
                let _ = writeln!(file, "{}", full_entry);
            }
        } else {
            // Append to memory log file
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("./memory/agent4_jailbreak_ultra.log") {
                let _ = writeln!(file, "{}", log_entry);
            }
        }
    }

    pub fn set_check_interval(&mut self, interval: Duration) {
        self.check_interval = interval;
    }

    pub fn get_check_interval(&self) -> Duration {
        self.check_interval
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_monitor_initialization() {
        // This test would require Redis instance
        // For now, test the structure
        let dependencies: HashMap<String, DependencyStatus> = HashMap::new();
        assert!(dependencies.is_empty());
    }
}
