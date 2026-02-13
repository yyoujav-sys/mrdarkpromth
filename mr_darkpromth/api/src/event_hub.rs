// MR.DarkPromth Event Hub
// Central broadcast hub for real-time WebSocket events

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use uuid::Uuid;

/// Events that can be pushed to connected WebSocket clients
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsEvent {
    /// Sync active sessions across clients
    #[serde(rename = "session_sync")]
    SessionSync {
        active_sessions: Vec<SessionInfo>,
        is_online: bool,
    },

    /// Chat message notification  
    #[serde(rename = "chat_notification")]
    ChatNotification {
        session_id: String,
        message_preview: String,
        role: String,
    },

    /// System-wide broadcast from admin
    #[serde(rename = "system_broadcast")]
    SystemBroadcast {
        event_type: String,
        payload: serde_json::Value,
    },

    /// Another user's online status changed (for contacts/admins)
    #[serde(rename = "user_status_changed")]
    UserStatusChanged {
        user_id: String,
        username: String,
        is_online: bool,
        client_type: String,
    },

    /// Server heartbeat to keep connection alive
    #[serde(rename = "heartbeat")]
    Heartbeat {
        timestamp: String,
        server_time: String,
    },

    /// Terminal session status update
    #[serde(rename = "terminal_update")]
    TerminalUpdate {
        session_id: String,
        status: String,
        exit_code: Option<i32>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub client_type: String,
    pub last_activity: String,
    pub created_at: String,
}

/// A targeted event: contains the user_id it's destined for, plus the event
#[derive(Debug, Clone)]
pub struct TargetedEvent {
    /// None = broadcast to all users
    pub target_user_id: Option<Uuid>,
    pub event: WsEvent,
}

/// Central event hub using tokio broadcast channel
pub struct EventHub {
    sender: broadcast::Sender<TargetedEvent>,
}

impl EventHub {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1024);
        Self { sender }
    }

    /// Subscribe to receive events (each WS connection calls this)
    pub fn subscribe(&self) -> broadcast::Receiver<TargetedEvent> {
        self.sender.subscribe()
    }

    /// Send an event to a specific user
    pub fn send_to_user(&self, user_id: Uuid, event: WsEvent) {
        let _ = self.sender.send(TargetedEvent {
            target_user_id: Some(user_id),
            event,
        });
    }

    /// Broadcast an event to all connected users
    pub fn broadcast_all(&self, event: WsEvent) {
        let _ = self.sender.send(TargetedEvent {
            target_user_id: None,
            event,
        });
    }

    /// Send a system broadcast event (persisted to DB + pushed to clients)
    pub fn send_system_event(&self, event_type: &str, payload: serde_json::Value) {
        self.broadcast_all(WsEvent::SystemBroadcast {
            event_type: event_type.to_string(),
            payload,
        });
    }
}
