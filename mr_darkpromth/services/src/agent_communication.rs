use crate::agent_framework::{Agent, AgentMessage, AgentResult, AgentError};
use crate::redis_coordination::RedisCoordinator;
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

pub type MessageHandler = Box<dyn Fn(AgentMessage) -> AgentResult<()> + Send + Sync>;

pub struct AgentCommunicationManager {
    redis_coordinator: Arc<RedisCoordinator>,
    agent_subscriptions: Arc<Mutex<HashMap<String, Vec<String>>>>,
    message_handlers: Arc<Mutex<HashMap<String, MessageHandler>>>,
}

impl AgentCommunicationManager {
    pub fn new(redis_coordinator: Arc<RedisCoordinator>) -> Self {
        Self {
            redis_coordinator,
            agent_subscriptions: Arc::new(Mutex::new(HashMap::new())),
            message_handlers: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    pub async fn register_agent(&self, agent_id: &str, agent_type: &str) -> AgentResult<()> {
        log::info!("Registering agent: {} (type: {})", agent_id, agent_type);
        
        let channel = format!("agent:{}", agent_id);
        self.redis_coordinator.subscribe(&channel).await
            .map_err(|e| AgentError::CommunicationError(e.to_string()))?;
        
        let mut subscriptions = self.agent_subscriptions.lock().unwrap();
        subscriptions.entry(agent_id.to_string()).or_default();
        
        Ok(())
    }
    
    pub async fn send_message(&self, message: AgentMessage) -> AgentResult<()> {
        log::info!("Sending message from {} to {}: {}", 
            message.from_agent, message.to_agent, message.message_type);
        
        let channel = format!("agent:{}", message.to_agent);
        let message_json = serde_json::to_string(&message)
            .map_err(|e| AgentError::SerializationError(e.to_string()))?;
        
        self.redis_coordinator.publish(&channel, &message_json).await
            .map_err(|e| AgentError::CommunicationError(e.to_string()))?;
        
        Ok(())
    }
    
    pub async fn broadcast_message(&self, from_agent: &str, message_type: &str, content: serde_json::Value) -> AgentResult<()> {
        log::info!("Broadcasting message from {}: {}", from_agent, message_type);
        
        let subscriptions = self.agent_subscriptions.lock().unwrap().clone();
        
        for agent_id in subscriptions.keys() {
            if agent_id != from_agent {
                let message = AgentMessage {
                    id: Uuid::new_v4(),
                    from_agent: from_agent.to_string(),
                    to_agent: agent_id.clone(),
                    message_type: message_type.to_string(),
                    content: content.clone(),
                    timestamp: chrono::Utc::now(),
                };
                
                self.send_message(message).await?;
            }
        }
        
        Ok(())
    }
    
    pub async fn subscribe_to_events(&self, agent_id: &str, callback: Box<dyn Fn(AgentMessage) -> AgentResult<()> + Send + Sync>) -> AgentResult<()> {
        log::info!("Agent {} subscribing to events", agent_id);
        
        let _channel = format!("agent:{}", agent_id);
        
        let mut handlers = self.message_handlers.lock().unwrap();
        handlers.insert(agent_id.to_string(), callback);
        
        Ok(())
    }
    
    pub async fn start_message_listener(&self, agent_id: &str, agent: Arc<Mutex<dyn Agent>>) -> AgentResult<()> {
        let channel = format!("agent:{}", agent_id);
        let redis = self.redis_coordinator.clone();
        
        tokio::spawn(async move {
            loop {
                match redis.subscribe(&channel).await {
                    Ok(_) => {
                        while let Ok(message) = redis.get_message().await {
                            if let Ok(agent_message) = serde_json::from_str::<AgentMessage>(&message) {
                                let mut agent_guard = agent.lock().unwrap();
                                if let Err(e) = agent_guard.handle_message(agent_message) {
                                    log::error!("Error handling message: {}", e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Error in message listener: {}", e);
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    }
                }
            }
        });
        
        Ok(())
    }
}

pub struct CollaborationProtocol {
    communication_manager: Arc<AgentCommunicationManager>,
}

impl CollaborationProtocol {
    pub fn new(communication_manager: Arc<AgentCommunicationManager>) -> Self {
        Self {
            communication_manager,
        }
    }
    
    pub async fn delegate_task(&self, from_agent: &str, to_agent: &str, task_description: &str, task_id: Uuid) -> AgentResult<()> {
        log::info!("Delegating task from {} to {}: {}", from_agent, to_agent, task_description);
        
        let message = AgentMessage {
            id: Uuid::new_v4(),
            from_agent: from_agent.to_string(),
            to_agent: to_agent.to_string(),
            message_type: "task_delegation".to_string(),
            content: json!({
                "task_id": task_id.to_string(),
                "description": task_description,
            }),
            timestamp: chrono::Utc::now(),
        };
        
        self.communication_manager.send_message(message).await
    }
    
    pub async fn request_assistance(&self, from_agent: &str, to_agent: &str, request: &str) -> AgentResult<()> {
        log::info!("Agent {} requesting assistance from {}: {}", from_agent, to_agent, request);
        
        let message = AgentMessage {
            id: Uuid::new_v4(),
            from_agent: from_agent.to_string(),
            to_agent: to_agent.to_string(),
            message_type: "assistance_request".to_string(),
            content: json!({
                "request": request,
            }),
            timestamp: chrono::Utc::now(),
        };
        
        self.communication_manager.send_message(message).await
    }
    
    pub async fn share_result(&self, from_agent: &str, to_agent: &str, result: serde_json::Value) -> AgentResult<()> {
        log::info!("Agent {} sharing result with {}", from_agent, to_agent);
        
        let message = AgentMessage {
            id: Uuid::new_v4(),
            from_agent: from_agent.to_string(),
            to_agent: to_agent.to_string(),
            message_type: "result_sharing".to_string(),
            content: result,
            timestamp: chrono::Utc::now(),
        };
        
        self.communication_manager.send_message(message).await
    }
    
    pub async fn coordinate_action(&self, initiator: &str, participants: Vec<String>, action: &str, context: serde_json::Value) -> AgentResult<()> {
        log::info!("Coordinating action {} initiated by {} with participants: {:?}", action, initiator, participants);
        
        let message = AgentMessage {
            id: Uuid::new_v4(),
            from_agent: initiator.to_string(),
            to_agent: "all".to_string(),
            message_type: "coordination".to_string(),
            content: json!({
                "action": action,
                "participants": participants,
                "context": context,
            }),
            timestamp: chrono::Utc::now(),
        };
        
        self.communication_manager.broadcast_message(initiator, "coordination", message.content).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_serialization() {
        let message = AgentMessage {
            id: Uuid::new_v4(),
            from_agent: "agent1".to_string(),
            to_agent: "agent2".to_string(),
            message_type: "test".to_string(),
            content: json!({"data": "test"}),
            timestamp: chrono::Utc::now(),
        };
        
        let serialized = serde_json::to_string(&message).unwrap();
        let deserialized: AgentMessage = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(message.from_agent, deserialized.from_agent);
        assert_eq!(message.to_agent, deserialized.to_agent);
    }
}
