// MR.DarkPromth User Event Coordinator
// Agent 5: User Management & Authentication Engineer
// Redis Event Bus Integration for User Events

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;
use anyhow::Result;

use crate::redis_coordination::{RedisCoordinator, CoordinationEvent, EventType};
use mr_darkpromth_db::{User, UserTier};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserEventType {
    UserRegistered,
    UserLoggedIn,
    UserLoggedOut,
    UserTierChanged,
    UserUpdated,
    UserDeactivated,
    UserActivated,
    UserDeleted,
    ApiKeyRegenerated,
    PasswordChanged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserEvent {
    pub event_type: UserEventType,
    pub user_id: Uuid,
    pub username: String,
    pub email: String,
    pub tier: UserTier,
    pub timestamp: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierChangeEvent {
    pub user_id: Uuid,
    pub username: String,
    pub old_tier: UserTier,
    pub new_tier: UserTier,
    pub reason: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRegistrationEvent {
    pub user_id: Uuid,
    pub username: String,
    pub email: String,
    pub tier: UserTier,
    pub registration_ip: Option<String>,
    pub timestamp: DateTime<Utc>,
}

pub struct UserEventCoordinator {
    redis_coordinator: Arc<RwLock<RedisCoordinator>>,
    #[allow(dead_code)]
    event_queue: Vec<UserEvent>,
}

impl UserEventCoordinator {
    pub fn new(redis_coordinator: RedisCoordinator) -> Self {
        Self {
            redis_coordinator: Arc::new(RwLock::new(redis_coordinator)),
            event_queue: Vec::new(),
        }
    }

    /// Publish user registration event
    pub async fn publish_user_registered(&self, user: &User, registration_ip: Option<String>) -> Result<()> {
        let event = UserEvent {
            event_type: UserEventType::UserRegistered,
            user_id: user.id,
            username: user.username.clone(),
            email: user.email.clone(),
            tier: user.tier.clone(),
            timestamp: Utc::now(),
            metadata: serde_json::json!({
                "registration_ip": registration_ip,
                "is_active": user.is_active,
            }),
        };

        self.publish_event(event).await
    }

    /// Publish user login event
    pub async fn publish_user_logged_in(&self, user: &User, login_ip: Option<String>) -> Result<()> {
        let event = UserEvent {
            event_type: UserEventType::UserLoggedIn,
            user_id: user.id,
            username: user.username.clone(),
            email: user.email.clone(),
            tier: user.tier.clone(),
            timestamp: Utc::now(),
            metadata: serde_json::json!({
                "login_ip": login_ip,
            }),
        };

        self.publish_event(event).await
    }

    /// Publish user logout event
    pub async fn publish_user_logged_out(&self, user: &User) -> Result<()> {
        let event = UserEvent {
            event_type: UserEventType::UserLoggedOut,
            user_id: user.id,
            username: user.username.clone(),
            email: user.email.clone(),
            tier: user.tier.clone(),
            timestamp: Utc::now(),
            metadata: serde_json::json!({}),
        };

        self.publish_event(event).await
    }

    /// Publish tier change event
    pub async fn publish_tier_changed(&self, user: &User, old_tier: UserTier, reason: String) -> Result<()> {
        let event = UserEvent {
            event_type: UserEventType::UserTierChanged,
            user_id: user.id,
            username: user.username.clone(),
            email: user.email.clone(),
            tier: user.tier.clone(),
            timestamp: Utc::now(),
            metadata: serde_json::json!({
                "old_tier": old_tier,
                "new_tier": user.tier,
                "reason": reason,
            }),
        };

        self.publish_event(event).await
    }

    /// Publish user updated event
    pub async fn publish_user_updated(&self, user: &User) -> Result<()> {
        let event = UserEvent {
            event_type: UserEventType::UserUpdated,
            user_id: user.id,
            username: user.username.clone(),
            email: user.email.clone(),
            tier: user.tier.clone(),
            timestamp: Utc::now(),
            metadata: serde_json::json!({
                "is_active": user.is_active,
            }),
        };

        self.publish_event(event).await
    }

    /// Publish API key regenerated event
    pub async fn publish_api_key_regenerated(&self, user: &User) -> Result<()> {
        let event = UserEvent {
            event_type: UserEventType::ApiKeyRegenerated,
            user_id: user.id,
            username: user.username.clone(),
            email: user.email.clone(),
            tier: user.tier.clone(),
            timestamp: Utc::now(),
            metadata: serde_json::json!({
                "api_key_expires_at": user.api_key_expires_at,
            }),
        };

        self.publish_event(event).await
    }

    /// Publish password changed event
    pub async fn publish_password_changed(&self, user: &User) -> Result<()> {
        let event = UserEvent {
            event_type: UserEventType::PasswordChanged,
            user_id: user.id,
            username: user.username.clone(),
            email: user.email.clone(),
            tier: user.tier.clone(),
            timestamp: Utc::now(),
            metadata: serde_json::json!({}),
        };

        self.publish_event(event).await
    }

    /// Publish user deactivated event
    pub async fn publish_user_deactivated(&self, user: &User, reason: String) -> Result<()> {
        let event = UserEvent {
            event_type: UserEventType::UserDeactivated,
            user_id: user.id,
            username: user.username.clone(),
            email: user.email.clone(),
            tier: user.tier.clone(),
            timestamp: Utc::now(),
            metadata: serde_json::json!({
                "reason": reason,
            }),
        };

        self.publish_event(event).await
    }

    /// Publish user activated event
    pub async fn publish_user_activated(&self, user: &User) -> Result<()> {
        let event = UserEvent {
            event_type: UserEventType::UserActivated,
            user_id: user.id,
            username: user.username.clone(),
            email: user.email.clone(),
            tier: user.tier.clone(),
            timestamp: Utc::now(),
            metadata: serde_json::json!({}),
        };

        self.publish_event(event).await
    }

    /// Publish user deleted event
    pub async fn publish_user_deleted(&self, user_id: Uuid, username: String, email: String) -> Result<()> {
        let event = UserEvent {
            event_type: UserEventType::UserDeleted,
            user_id,
            username,
            email,
            tier: UserTier::Free, // Default value for deleted users
            timestamp: Utc::now(),
            metadata: serde_json::json!({}),
        };

        self.publish_event(event).await
    }

    async fn publish_event(&self, event: UserEvent) -> Result<()> {
        let mut coordinator = self.redis_coordinator.write().await;

        let payload = serde_json::to_value(&event)?;
        
        let coordination_event = CoordinationEvent {
            event_id: Uuid::new_v4().to_string(),
            agent_id: "agent5".to_string(),
            event_type: EventType::ResponseEvent,
            timestamp: Utc::now(),
            correlation_id: None,
            payload,
        };

        coordinator.publish_event(coordination_event).map_err(|e| anyhow::anyhow!("Redis error: {}", e))?;

        log::info!(
            "Published user event: {:?} for user {}",
            event.event_type,
            event.username
        );

        Ok(())
    }

    /// Subscribe to user events from other agents
    pub async fn subscribe_to_user_events(&self) -> Result<()> {
        let mut coordinator = self.redis_coordinator.write().await;
        coordinator.subscribe_to_events(&EventType::ResourceReady)?;
        Ok(())
    }

    /// Handle incoming user events from other agents
    pub async fn handle_incoming_event(&self, event: CoordinationEvent) -> Result<()> {
        match event.event_type {
            EventType::ResourceReady => {
                if let Some(resource) = event.payload.get("resource").and_then(|r| r.as_str()) {
                    if resource == "agent4_ready" {
                        log::info!("Agent 4 (Ultra Tier) is ready for user event coordination");
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Notify Agent 4 about Ultra Tier user changes
    pub async fn notify_ultra_tier_agent(&self, user_id: Uuid, is_ultra: bool) -> Result<()> {
        let mut coordinator = self.redis_coordinator.write().await;

        let payload = serde_json::json!({
            "event_type": if is_ultra { "user_upgraded_to_ultra" } else { "user_downgraded_from_ultra" },
            "user_id": user_id,
            "timestamp": Utc::now(),
        });

        let coordination_event = CoordinationEvent {
            event_id: Uuid::new_v4().to_string(),
            agent_id: "agent5".to_string(),
            event_type: EventType::ResponseEvent,
            timestamp: Utc::now(),
            correlation_id: None,
            payload,
        };

        coordinator.publish_event(coordination_event).map_err(|e| anyhow::anyhow!("Redis error: {}", e))?;

        log::info!("Notified Agent 4 about tier change for user {}", user_id);

        Ok(())
    }

    /// Broadcast user statistics to all agents
    pub async fn broadcast_user_stats(&self, stats: serde_json::Value) -> Result<()> {
        let mut coordinator = self.redis_coordinator.write().await;

        let payload = serde_json::json!({
            "event_type": "user_stats_update",
            "stats": stats,
            "timestamp": Utc::now(),
        });

        let coordination_event = CoordinationEvent {
            event_id: Uuid::new_v4().to_string(),
            agent_id: "agent5".to_string(),
            event_type: EventType::ResponseEvent,
            timestamp: Utc::now(),
            correlation_id: None,
            payload,
        };

        coordinator.publish_event(coordination_event).map_err(|e| anyhow::anyhow!("Redis error: {}", e))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_event_serialization() {
        let event = UserEvent {
            event_type: UserEventType::UserRegistered,
            user_id: Uuid::new_v4(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            tier: UserTier::Free,
            timestamp: Utc::now(),
            metadata: serde_json::json!({}),
        };

        let serialized = serde_json::to_string(&event).unwrap();
        assert!(serialized.contains("UserRegistered"));
    }
}
