use crate::agent::AgentMessage;
use anyhow::Result;
use redis::{Client, Commands};
use serde_json;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

#[derive(Clone)]
pub struct RedisEventBus {
    client: Client,
    #[allow(dead_code)]
    agent_id: String,
    message_handlers: Arc<RwLock<std::collections::HashMap<String, mpsc::UnboundedSender<AgentMessage>>>>,
}

impl RedisEventBus {
    pub fn new(redis_url: &str, agent_id: String) -> Result<Self> {
        let client = Client::open(redis_url)?;
        
        Ok(Self {
            client,
            agent_id,
            message_handlers: Arc::new(RwLock::new(std::collections::HashMap::new())),
        })
    }
    
    pub async fn publish_message(&self, channel: &str, message: AgentMessage) -> Result<()> {
        let mut conn = self.client.get_connection()?;
        let message_json = serde_json::to_string(&message)?;
        
        conn.publish::<_, _, ()>(channel, message_json)?;
        
        Ok(())
    }
    
    pub async fn subscribe_to_channel(
        &self,
        channel: String,
        sender: mpsc::UnboundedSender<AgentMessage>,
    ) -> Result<()> {
        let channel_clone = channel.clone();
        let client_clone = self.client.clone();
        let sender_clone = sender.clone();
        
        // Store the sender for this channel
        self.message_handlers.write().await.insert(channel.clone(), sender);
        
        tokio::spawn(async move {
            let mut conn = client_clone.get_connection().expect("Failed to get Redis connection");
            let mut pubsub = conn.as_pubsub();
            pubsub.subscribe(&channel_clone).expect("Failed to subscribe to channel");
            
            loop {
                match pubsub.get_message() {
                    Ok(msg) => {
                        if let Ok(data) = msg.get_payload::<String>() {
                            match serde_json::from_str::<AgentMessage>(&data) {
                                Ok(agent_message) => {
                                    // Only process messages not sent by this agent
                                    if agent_message.sender != channel_clone {
                                        if let Err(_) = sender_clone.send(agent_message) {
                                            // Channel closed, break the loop
                                            break;
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Failed to deserialize message: {}", e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error receiving message: {}", e);
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    }
                }
            }
        });
        
        Ok(())
    }
    
    pub async fn send_direct_message(&self, recipient: &str, message: AgentMessage) -> Result<()> {
        let channel = format!("agent:{}", recipient);
        self.publish_message(&channel, message).await
    }
    
    pub async fn broadcast_to_all_agents(&self, message: AgentMessage) -> Result<()> {
        self.publish_message("agents:all", message).await
    }
    
    pub async fn send_to_coordinator(&self, message: AgentMessage) -> Result<()> {
        self.publish_message("agent:coordinator", message).await
    }
}

pub struct MessageRouter {
    event_bus: RedisEventBus,
    incoming_messages: mpsc::UnboundedReceiver<AgentMessage>,
    outgoing_messages: mpsc::UnboundedSender<AgentMessage>,
}

impl MessageRouter {
    pub fn new(event_bus: RedisEventBus) -> (Self, mpsc::UnboundedReceiver<AgentMessage>) {
        let (_incoming_tx, incoming_rx) = mpsc::unbounded_channel();
        let (outgoing_tx, outgoing_rx) = mpsc::unbounded_channel();
        
        let router = Self {
            event_bus,
            incoming_messages: incoming_rx,
            outgoing_messages: outgoing_tx,
        };
        
        (router, outgoing_rx)
    }
    
    pub async fn start(&mut self, agent_id: &str) -> Result<()> {
        let agent_channel = format!("agent:{}", agent_id);
        let all_agents_channel = "agents:all".to_string();
        
        // Subscribe to agent-specific channel
        let (tx, _) = mpsc::unbounded_channel();
        self.event_bus.subscribe_to_channel(agent_channel, tx).await?;
        
        // Subscribe to broadcast channel
        let (tx2, _) = mpsc::unbounded_channel();
        self.event_bus.subscribe_to_channel(all_agents_channel, tx2).await?;
        
        Ok(())
    }
    
    pub async fn send_message(&self, message: AgentMessage) -> Result<()> {
        self.outgoing_messages.send(message)?;
        Ok(())
    }
    
    pub async fn receive_message(&mut self) -> Option<AgentMessage> {
        self.incoming_messages.recv().await
    }
}

#[derive(Debug, Clone)]
pub struct AgentCommunicationConfig {
    pub redis_url: String,
    pub agent_id: String,
    pub heartbeat_interval_secs: u64,
    pub message_timeout_secs: u64,
}

impl Default for AgentCommunicationConfig {
    fn default() -> Self {
        Self {
            redis_url: "redis://localhost:6379".to_string(),
            agent_id: "unknown".to_string(),
            heartbeat_interval_secs: 30,
            message_timeout_secs: 60,
        }
    }
}

pub struct AgentCommunicator {
    config: AgentCommunicationConfig,
    event_bus: RedisEventBus,
    router: MessageRouter,
    is_running: bool,
}

impl AgentCommunicator {
    pub fn new(config: AgentCommunicationConfig) -> Result<Self> {
        let event_bus = RedisEventBus::new(&config.redis_url, config.agent_id.clone())?;
        let (router, _) = MessageRouter::new(event_bus.clone());
        
        Ok(Self {
            config,
            event_bus,
            router,
            is_running: false,
        })
    }
    
    pub async fn start(&mut self) -> Result<()> {
        if self.is_running {
            return Ok(());
        }
        
        self.router.start(&self.config.agent_id).await?;
        self.is_running = true;
        
        // Start heartbeat
        self.start_heartbeat().await;
        
        Ok(())
    }
    
    pub async fn stop(&mut self) {
        self.is_running = false;
    }
    
    pub async fn send_message(&self, recipient: &str, message: AgentMessage) -> Result<()> {
        self.event_bus.send_direct_message(recipient, message).await
    }
    
    pub async fn broadcast_message(&self, message: AgentMessage) -> Result<()> {
        self.event_bus.broadcast_to_all_agents(message).await
    }
    
    pub async fn send_to_coordinator(&self, message: AgentMessage) -> Result<()> {
        self.event_bus.send_to_coordinator(message).await
    }
    
    pub async fn receive_message(&mut self) -> Option<AgentMessage> {
        self.router.receive_message().await
    }
    
    async fn start_heartbeat(&self) {
        let agent_id = self.config.agent_id.clone();
        let event_bus = self.event_bus.clone();
        let interval = self.config.heartbeat_interval_secs;
        
        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(tokio::time::Duration::from_secs(interval));
            
            loop {
                interval_timer.tick().await;
                
                let heartbeat = AgentMessage {
                    id: Uuid::new_v4(),
                    sender: agent_id.clone(),
                    recipient: "system".to_string(),
                    message_type: "heartbeat".to_string(),
                    payload: serde_json::json!({
                        "timestamp": chrono::Utc::now(),
                        "agent_id": agent_id
                    }),
                    timestamp: chrono::Utc::now(),
                };
                
                if let Err(e) = event_bus.publish_message("system:heartbeat", heartbeat).await {
                    eprintln!("Failed to send heartbeat: {}", e);
                }
            }
        });
    }
    
    pub fn config(&self) -> &AgentCommunicationConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::AgentState;
    
    #[tokio::test]
    async fn test_message_creation() {
        let message = AgentMessage {
            id: Uuid::new_v4(),
            sender: "test_agent".to_string(),
            recipient: "coordinator".to_string(),
            message_type: "test".to_string(),
            payload: serde_json::json!({"test": "data"}),
            timestamp: chrono::Utc::now(),
        };
        
        assert_eq!(message.sender, "test_agent");
        assert_eq!(message.recipient, "coordinator");
        assert_eq!(message.message_type, "test");
    }
    
    #[tokio::test]
    async fn test_communication_config() {
        let config = AgentCommunicationConfig::default();
        assert_eq!(config.redis_url, "redis://localhost:6379");
        assert_eq!(config.heartbeat_interval_secs, 30);
        assert_eq!(config.message_timeout_secs, 60);
    }
}
