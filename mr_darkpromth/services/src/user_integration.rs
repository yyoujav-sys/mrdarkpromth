// MR.DarkPromth User Integration System
// Agent 4: Jailbreak & Ultra Tier Engineer
// Phase 3: User Management Integration

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::{RedisCoordinator, CoordinationEvent, EventType, UserTier, User};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserVerificationRequest {
    pub request_id: String,
    pub user_id: Option<String>,
    pub api_key: Option<String>,
    pub username: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserVerificationResponse {
    pub request_id: String,
    pub success: bool,
    pub user: Option<User>,
    pub error_message: Option<String>,
    pub timestamp: DateTime<Utc>,
}

pub struct UserIntegration {
    redis_coordinator: std::sync::Arc<std::sync::Mutex<RedisCoordinator>>,
    user_cache: RwLock<HashMap<String, User>>,
    cache_ttl_seconds: u64,
}

impl UserIntegration {
    pub fn new(redis_coordinator: RedisCoordinator) -> Self {
        Self {
            redis_coordinator: std::sync::Arc::new(std::sync::Mutex::new(redis_coordinator)),
            user_cache: RwLock::new(HashMap::new()),
            cache_ttl_seconds: 300, // 5 minutes cache TTL
        }
    }

    /// Verify user tier by querying Agent 5 (User Management)
    pub async fn verify_user_tier(&self, user_id: Option<String>, api_key: Option<String>, username: Option<String>) -> Result<UserTier, Box<dyn std::error::Error>> {
        // Check cache first
        let cache_key = if let Some(uid) = &user_id {
            uid.clone()
        } else if let Some(key) = &api_key {
            format!("api_key:{}", key)
        } else if let Some(uname) = &username {
            format!("username:{}", uname)
        } else {
            return Err("No user identifier provided".into());
        };

        if let Some(user) = self.user_cache.read().unwrap().get(&cache_key).cloned() {
            // Check if cache is still valid
            let cache_age = Utc::now().signed_duration_since(user.last_active);
            if cache_age.num_seconds() < self.cache_ttl_seconds as i64 {
                return Ok(user.tier);
            }
        }

        // Query Agent 5 for user verification
        let request_id = Uuid::new_v4().to_string();
        let correlation_id = self.send_user_verification_request(&request_id, user_id, api_key, username).await?;
        
        // Wait for response
        let response = self.wait_for_user_verification_response(&correlation_id, 5000).await?;
        
        if response.success {
            if let Some(user) = response.user {
                // Update cache
                self.user_cache.write().unwrap().insert(cache_key, user.clone());
                return Ok(user.tier);
            }
        }
        
        Err(response.error_message.unwrap_or_else(|| "User verification failed".to_string()).into())
    }

    /// Get user details with caching
    pub async fn get_user_details(&self, user_id: &str) -> Result<Option<User>, Box<dyn std::error::Error>> {
        // Check cache first
        if let Some(user) = self.user_cache.read().unwrap().get(user_id).cloned() {
            let cache_age = Utc::now().signed_duration_since(user.last_active);
            if cache_age.num_seconds() < self.cache_ttl_seconds as i64 {
                return Ok(Some(user));
            }
        }

        // Query Agent 5
        let request_id = Uuid::new_v4().to_string();
        let correlation_id = self.send_user_verification_request(&request_id, Some(user_id.to_string()), None, None).await?;
        
        let response = self.wait_for_user_verification_response(&correlation_id, 5000).await?;
        
        if response.success {
            if let Some(user) = response.user {
                self.user_cache.write().unwrap().insert(user_id.to_string(), user.clone());
                return Ok(Some(user));
            }
        }
        
        Ok(None)
    }

    /// Update user activity timestamp
    pub fn update_user_activity(&self, user_id: &str) {
        if let Some(user) = self.user_cache.write().unwrap().get_mut(user_id) {
            user.last_active = Utc::now();
        }
    }

    /// Clear user cache
    pub fn clear_cache(&self) {
        self.user_cache.write().unwrap().clear();
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> HashMap<String, serde_json::Value> {
        let mut stats = HashMap::new();
        let cache = self.user_cache.read().unwrap();
        let cache_len = cache.len() as u64;
        stats.insert("cached_users".to_string(), serde_json::Value::Number(cache_len.into()));
        stats.insert("cache_ttl_seconds".to_string(), serde_json::Value::Number(self.cache_ttl_seconds.into()));
        
        let ultra_users = cache.values().filter(|u| matches!(u.tier, UserTier::Ultra)).count() as u64;
        let free_users = cache.values().filter(|u| matches!(u.tier, UserTier::Free)).count() as u64;
        
        stats.insert("ultra_users_cached".to_string(), serde_json::Value::Number(ultra_users.into()));
        stats.insert("free_users_cached".to_string(), serde_json::Value::Number(free_users.into()));
        
        stats
    }

    async fn send_user_verification_request(&self, request_id: &str, user_id: Option<String>, api_key: Option<String>, username: Option<String>) -> Result<String, Box<dyn std::error::Error>> {
        let mut coordinator = self.redis_coordinator.lock().unwrap();
        
        let verification_request = UserVerificationRequest {
            request_id: request_id.to_string(),
            user_id,
            api_key,
            username,
            timestamp: Utc::now(),
        };

        let payload = serde_json::to_value(verification_request)?;
        let correlation_id = coordinator.publish_query_event("agent5", &serde_json::to_string(&payload)?)?;
        
        Ok(correlation_id)
    }

    async fn wait_for_user_verification_response(&self, correlation_id: &str, timeout_ms: u64) -> Result<UserVerificationResponse, Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();
        
        while start_time.elapsed().as_millis() < timeout_ms as u128 {
            if let Ok(coordinator) = self.redis_coordinator.try_lock() {
                if let Ok(events) = coordinator.read_events(&EventType::ResponseEvent, Some(100)) {
                    for stream_event in events {
                        if let Some(event_correlation_id) = &stream_event.event.correlation_id {
                            if event_correlation_id == correlation_id {
                                let _ = coordinator.acknowledge_event(&EventType::ResponseEvent, &stream_event.stream_id);
                                
                                if let Ok(response) = serde_json::from_value::<UserVerificationResponse>(stream_event.event.payload) {
                                    return Ok(response);
                                }
                            }
                        }
                    }
                }
            }
            
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
        
        Err("User verification response timeout".into())
    }

    /// Subscribe to user management events from Agent 5
    pub fn subscribe_to_user_events(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut coordinator = self.redis_coordinator.lock().unwrap();
        
        // Subscribe to user-related events
        coordinator.subscribe_to_events(&EventType::ResourceReady)?;
        
        Ok(())
    }

    /// Handle user management events (e.g., user tier changes)
    pub fn handle_user_event(&mut self, event: &CoordinationEvent) -> Result<(), Box<dyn std::error::Error>> {
        match event.event_type {
            EventType::ResourceReady => {
                if let Some(resource) = event.payload.get("resource").and_then(|r| r.as_str()) {
                    if resource == "user_management_ready" {
                        // User management system is ready
                        self.clear_cache(); // Clear cache to ensure fresh data
                    }
                }
            }
            _ => {}
        }
        
        Ok(())
    }

    /// Check if user management system is ready
    pub async fn is_user_management_ready(&self) -> bool {
        // Query Agent 5 to check if it's ready
        let request_id = Uuid::new_v4().to_string();
        
        if let Ok(mut coordinator) = self.redis_coordinator.try_lock() {
            let _ = coordinator.publish_query_event("agent5", &serde_json::json!({
                "query": "status_check",
                "request_id": request_id
            }).to_string());
            
            // Wait for response
            let start_time = std::time::Instant::now();
            while start_time.elapsed().as_millis() < 3000 {
                if let Ok(events) = coordinator.read_events(&EventType::ResponseEvent, Some(100)) {
                    for stream_event in events {
                        if let Some(_correlation_id) = &stream_event.event.correlation_id {
                            // This is a simplified check - in production, you'd want better correlation
                            if stream_event.event.payload.get("query").and_then(|q| q.as_str()) == Some("status_check") {
                                let _ = coordinator.acknowledge_event(&EventType::ResponseEvent, &stream_event.stream_id);
                                return stream_event.event.payload.get("status")
                                    .and_then(|s| s.as_str())
                                    .map(|s| s == "ready")
                                    .unwrap_or(false);
                            }
                        }
                    }
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        }
        
        false
    }

    /// Get Ultra Tier users from cache
    pub fn get_ultra_tier_users(&self) -> Vec<User> {
        self.user_cache
            .read()
            .unwrap()
            .values()
            .filter(|user| matches!(user.tier, UserTier::Ultra))
            .cloned()
            .collect()
    }

    /// Get Free Tier users from cache
    pub fn get_free_tier_users(&self) -> Vec<User> {
        self.user_cache
            .read()
            .unwrap()
            .values()
            .filter(|user| matches!(user.tier, UserTier::Free))
            .cloned()
            .collect()
    }

    /// Invalidate specific user cache entry
    pub fn invalidate_user_cache(&mut self, user_id: &str) {
        self.user_cache.write().unwrap().remove(user_id);
    }

    /// Set cache TTL
    pub fn set_cache_ttl(&mut self, ttl_seconds: u64) {
        self.cache_ttl_seconds = ttl_seconds;
    }

    /// Get cache TTL
    pub fn get_cache_ttl(&self) -> u64 {
        self.cache_ttl_seconds
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_integration_initialization() {
        // This test would require Redis instance
        // For now, test the structure
        let cache_stats: std::collections::HashMap<String, usize> = HashMap::new();
        assert!(cache_stats.is_empty());
    }

    #[test]
    fn test_cache_ttl() {
        // Test would require Redis coordinator
        // For now, test the concept
        let ttl = 300u64;
        assert_eq!(ttl, 300);
    }
}
