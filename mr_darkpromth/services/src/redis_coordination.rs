// MR.DarkPromth Redis Coordination System
// Agent 4: Jailbreak & Ultra Tier Engineer
// Phase 1 Implementation - Redis Event Bus Integration

use redis::{Client, Connection, Commands, RedisResult, RedisError};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationEvent {
    pub event_id: String,
    pub agent_id: String,
    pub event_type: EventType,
    pub timestamp: DateTime<Utc>,
    pub correlation_id: Option<String>,
    pub payload: Value,
}

#[derive(Debug, Clone)]
pub struct RedisStreamEvent {
    pub stream_id: String,
    pub event: CoordinationEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EventType {
    TaskCompletion,
    ResourceReady,
    ErrorEvent,
    QueryEvent,
    ResponseEvent,
    Heartbeat,
    Shutdown,
}

pub struct RedisCoordinator {
    client: Client,
    agent_id: String,
    subscriptions: HashMap<String, String>,
    pubsub_channel: Arc<Mutex<Option<String>>>,
}

impl RedisCoordinator {
    pub fn new(redis_url: &str, agent_id: String) -> Result<Self, RedisError> {
        let client = Client::open(redis_url)?;
        
        Ok(Self {
            client,
            agent_id,
            subscriptions: HashMap::new(),
            pubsub_channel: Arc::new(Mutex::new(None)),
        })
    }

    pub fn get_connection(&self) -> RedisResult<Connection> {
        self.client.get_connection()
    }

    pub fn publish_event(&mut self, event: CoordinationEvent) -> RedisResult<()> {
        let mut conn = self.get_connection()?;
        let stream_name = self.get_publish_stream_name(&event);
        let event_json = serde_json::to_string(&event).unwrap();
        
        // Use Redis Streams for guaranteed delivery
        let _: String = conn.xadd(&stream_name, "*", &[
            ("event_data", &event_json)
        ])?;
        
        Ok(())
    }

    pub async fn publish(&self, channel: &str, message: &str) -> RedisResult<()> {
        let mut conn = self.get_connection()?;
        let _: i64 = redis::cmd("RPUSH")
            .arg(channel)
            .arg(message)
            .query(&mut conn)?;
        Ok(())
    }

    pub async fn subscribe(&self, channel: &str) -> RedisResult<()> {
        let mut guard = self
            .pubsub_channel
            .lock()
            .map_err(|_| RedisError::from((redis::ErrorKind::IoError, "pubsub lock poisoned")))?;
        *guard = Some(channel.to_string());
        Ok(())
    }

    pub async fn get_message(&self) -> RedisResult<String> {
        let channel = self
            .pubsub_channel
            .lock()
            .map_err(|_| RedisError::from((redis::ErrorKind::IoError, "pubsub lock poisoned")))?
            .clone();

        let channel = channel.ok_or_else(|| {
            RedisError::from((redis::ErrorKind::IoError, "No channel subscribed"))
        })?;

        let mut conn = self.get_connection()?;
        let result: Option<(String, String)> = redis::cmd("BLPOP")
            .arg(&channel)
            .arg(1)
            .query(&mut conn)?;

        if let Some((_key, message)) = result {
            Ok(message)
        } else {
            Err(RedisError::from((redis::ErrorKind::IoError, "No message")))
        }
    }

    pub fn subscribe_to_events(&mut self, event_type: &EventType) -> RedisResult<String> {
        let stream_name = self.get_stream_name(event_type);
        let consumer_group = format!("{}_group", self.agent_id);
        let consumer_name = self.agent_id.clone();
        
        let mut conn = self.get_connection()?;
        
        // Create consumer group if it doesn't exist
        let _: RedisResult<()> = conn.xgroup_create_mkstream(&stream_name, &consumer_group, "0");
        
        self.subscriptions.insert(format!("{:?}", event_type), stream_name.clone());
        
        Ok(stream_name)
    }

    pub fn read_events(&self, event_type: &EventType, block_ms: Option<i64>) -> RedisResult<Vec<RedisStreamEvent>> {
        let stream_name = self.get_stream_name(event_type);
        let consumer_group = format!("{}_group", self.agent_id);
        let consumer_name = self.agent_id.clone();
        
        let mut conn = self.get_connection()?;

        let mut cmd = redis::cmd("XREADGROUP");
        cmd.arg("GROUP").arg(&consumer_group).arg(&consumer_name);
        if let Some(ms) = block_ms {
            cmd.arg("BLOCK").arg(ms);
        }
        cmd.arg("STREAMS").arg(&stream_name).arg(">");

        let result: Option<Vec<(String, Vec<(String, Vec<(String, String)>)>)>> = cmd.query(&mut conn)?;
        let result = result.unwrap_or_default();

        let mut events = Vec::new();

        for (_, messages) in result {
            for (stream_id, fields) in messages {
                if let Some(event_data) = fields.iter().find(|(k, _)| k == "event_data") {
                    if let Ok(event) = serde_json::from_str::<CoordinationEvent>(&event_data.1) {
                        events.push(RedisStreamEvent { stream_id, event });
                    }
                }
            }
        }

        Ok(events)
    }

    pub fn acknowledge_event(&self, event_type: &EventType, stream_id: &str) -> RedisResult<()> {
        let stream_name = self.get_stream_name(event_type);
        let consumer_group = format!("{}_group", self.agent_id);
        
        let mut conn = self.get_connection()?;
        let _: () = conn.xack(&stream_name, &consumer_group, &[stream_id])?;
        
        Ok(())
    }

    pub fn publish_task_completion(&mut self, task: &str, artifacts: Vec<String>, metadata: Value) -> RedisResult<()> {
        let event = CoordinationEvent {
            event_id: Uuid::new_v4().to_string(),
            agent_id: self.agent_id.clone(),
            event_type: EventType::TaskCompletion,
            timestamp: Utc::now(),
            correlation_id: None,
            payload: json!({
                "task": task,
                "status": "completed",
                "artifacts": artifacts,
                "metadata": metadata
            }),
        };
        
        self.publish_event(event)
    }

    pub fn publish_resource_ready(&mut self, resource: &str, artifacts: Vec<String>) -> RedisResult<()> {
        let event = CoordinationEvent {
            event_id: Uuid::new_v4().to_string(),
            agent_id: self.agent_id.clone(),
            event_type: EventType::ResourceReady,
            timestamp: Utc::now(),
            correlation_id: None,
            payload: json!({
                "resource": resource,
                "artifacts": artifacts,
                "status": "ready"
            }),
        };
        
        self.publish_event(event)
    }

    pub fn publish_error_event(&mut self, error_type: &str, message: &str, file: Option<&str>, line: Option<u32>) -> RedisResult<()> {
        let event = CoordinationEvent {
            event_id: Uuid::new_v4().to_string(),
            agent_id: self.agent_id.clone(),
            event_type: EventType::ErrorEvent,
            timestamp: Utc::now(),
            correlation_id: None,
            payload: json!({
                "error_type": error_type,
                "message": message,
                "file": file,
                "line": line
            }),
        };
        
        self.publish_event(event)
    }

    pub fn publish_query_event(&mut self, target_agent: &str, query: &str) -> RedisResult<String> {
        let correlation_id = Uuid::new_v4().to_string();
        
        let event = CoordinationEvent {
            event_id: Uuid::new_v4().to_string(),
            agent_id: self.agent_id.clone(),
            event_type: EventType::QueryEvent,
            timestamp: Utc::now(),
            correlation_id: Some(correlation_id.clone()),
            payload: json!({
                "query": query,
                "target_agent": target_agent
            }),
        };
        
        self.publish_event(event)?;
        Ok(correlation_id)
    }

    pub fn publish_response_event(&mut self, correlation_id: &str, response: &str, target_agent: &str) -> RedisResult<()> {
        let event = CoordinationEvent {
            event_id: Uuid::new_v4().to_string(),
            agent_id: self.agent_id.clone(),
            event_type: EventType::ResponseEvent,
            timestamp: Utc::now(),
            correlation_id: Some(correlation_id.to_string()),
            payload: json!({
                "response": response,
                "target_agent": target_agent
            }),
        };
        
        self.publish_event(event)
    }

    pub fn publish_heartbeat(&mut self, status: &str, tasks_in_progress: u32, tasks_completed: u32) -> RedisResult<()> {
        let event = CoordinationEvent {
            event_id: Uuid::new_v4().to_string(),
            agent_id: self.agent_id.clone(),
            event_type: EventType::Heartbeat,
            timestamp: Utc::now(),
            correlation_id: None,
            payload: json!({
                "status": status,
                "tasks_in_progress": tasks_in_progress,
                "tasks_completed": tasks_completed
            }),
        };
        
        self.publish_event(event)
    }

    pub fn wait_for_response(&self, correlation_id: &str, timeout_ms: u64) -> Option<CoordinationEvent> {
        let start_time = std::time::Instant::now();
        let correlation_id = correlation_id.to_string();
        
        while start_time.elapsed().as_millis() < timeout_ms as u128 {
            if let Ok(events) = self.read_events(&EventType::ResponseEvent, Some(1000)) {
                for stream_event in events {
                    if stream_event.event.correlation_id.as_ref() == Some(&correlation_id) {
                        let _ = self.acknowledge_event(&EventType::ResponseEvent, &stream_event.stream_id);
                        return Some(stream_event.event);
                    }
                }
            }
        }
        
        None
    }

    fn get_publish_stream_name(&self, event: &CoordinationEvent) -> String {
        match &event.event_type {
            EventType::QueryEvent => event
                .payload
                .get("target_agent")
                .and_then(|agent| agent.as_str())
                .map(|agent| format!("mr_darkpromth:{}:query_event", agent))
                .unwrap_or_else(|| self.get_stream_name(&event.event_type)),
            EventType::ResponseEvent => event
                .payload
                .get("target_agent")
                .and_then(|agent| agent.as_str())
                .map(|agent| format!("mr_darkpromth:{}:response_event", agent))
                .unwrap_or_else(|| self.get_stream_name(&event.event_type)),
            _ => self.get_stream_name(&event.event_type),
        }
    }

    fn get_stream_name(&self, event_type: &EventType) -> String {
        match event_type {
            EventType::TaskCompletion => "mr_darkpromth:global:task_completion".to_string(),
            EventType::ResourceReady => "mr_darkpromth:global:resource_ready".to_string(),
            EventType::ErrorEvent => "mr_darkpromth:global:error_event".to_string(),
            EventType::QueryEvent => format!("mr_darkpromth:{}:query_event", self.agent_id),
            EventType::ResponseEvent => format!("mr_darkpromth:{}:response_event", self.agent_id),
            EventType::Heartbeat => "mr_darkpromth:system:heartbeat".to_string(),
            EventType::Shutdown => "mr_darkpromth:global:shutdown".to_string(),
        }
    }

    pub fn start_heartbeat_loop(&mut self, interval_seconds: u64) -> std::thread::JoinHandle<()> {
        let agent_id = self.agent_id.clone();
        let mut coordinator = self.clone();
        
        std::thread::spawn(move || {
            loop {
                if let Err(e) = coordinator.publish_heartbeat("healthy", 0, 0) {
                    eprintln!("Failed to publish heartbeat: {}", e);
                }
                std::thread::sleep(std::time::Duration::from_secs(interval_seconds));
            }
        })
    }
}

impl Clone for RedisCoordinator {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            agent_id: self.agent_id.clone(),
            subscriptions: self.subscriptions.clone(),
            pubsub_channel: self.pubsub_channel.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_serialization() {
        let event = CoordinationEvent {
            event_id: Uuid::new_v4().to_string(),
            agent_id: "agent4".to_string(),
            event_type: EventType::TaskCompletion,
            timestamp: Utc::now(),
            correlation_id: None,
            payload: json!({"task": "test"}),
        };
        
        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: CoordinationEvent = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(event.agent_id, deserialized.agent_id);
    }
}
