use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use log::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Provider {
    Cerebras,
    OpenRouter,
    Anthropic,
    OpenAI,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: String,
    pub provider: Provider,
    pub key: String,
    pub label: String,
    pub is_active: bool,
    pub last_used: Option<DateTime<Utc>>,
    pub failure_count: u32,
    pub rate_limit_until: Option<DateTime<Utc>>,
}

pub struct KeyPool {
    keys: Arc<RwLock<Vec<ApiKey>>>,
    #[allow(dead_code)]
    provider_indices: Arc<RwLock<HashMap<String, usize>>>,
}

impl Default for KeyPool {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyPool {
    pub fn new() -> Self {
        Self {
            keys: Arc::new(RwLock::new(Vec::new())),
            provider_indices: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_key(&self, provider: Provider, key: String, label: String) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let api_key = ApiKey {
            id: id.clone(),
            provider,
            key,
            label,
            is_active: true,
            last_used: None,
            failure_count: 0,
            rate_limit_until: None,
        };
        
        let mut keys = self.keys.write().await;
        keys.push(api_key);
        info!("Added new API key for provider {:?} with ID {}", keys.last().unwrap().provider, id);
        id
    }

    pub async fn get_best_key(&self, provider: Provider) -> Option<ApiKey> {
        let mut keys = self.keys.write().await;
        let now = Utc::now();
        
        // Auto-recovery: If key is inactive but hasn't been tried for 10 minutes, reactivate it
        for k in keys.iter_mut() {
            if !k.is_active {
                if let Some(last) = k.last_used {
                    if now - last > chrono::Duration::minutes(10) {
                        k.is_active = true;
                        k.failure_count = 0;
                        info!("Automatically reactivating API key {} for provider {:?} after cooldown", k.id, k.provider);
                    }
                }
            }
        }

        // filtered keys for the requested provider
        let mut available_keys: Vec<(usize, &mut ApiKey)> = keys.iter_mut().enumerate()
            .filter(|(_, k)| {
                // Match provider efficiently
                let provider_match = match (&k.provider, &provider) {
                    (Provider::Cerebras, Provider::Cerebras) => true,
                    (Provider::OpenRouter, Provider::OpenRouter) => true,
                    (Provider::Anthropic, Provider::Anthropic) => true,
                    (Provider::OpenAI, Provider::OpenAI) => true,
                    (Provider::Custom(p1), Provider::Custom(p2)) => p1 == p2,
                    _ => false,
                };
                
                provider_match && k.is_active && k.rate_limit_until.is_none_or(|until| until < now)
            })
            .collect();

        if available_keys.is_empty() {
            warn!("No available keys for provider {:?}", provider);
            return None;
        }

        // Round-robin or least recently used logic
        available_keys.sort_by(|(_, a), (_, b)| a.last_used.cmp(&b.last_used));
        
        let (_index, selected_key) = available_keys.remove(0);
        selected_key.last_used = Some(now);
        
        Some(selected_key.clone())
    }

    pub async fn report_failure(&self, id: &str) {
        let mut keys = self.keys.write().await;
        if let Some(key) = keys.iter_mut().find(|k| k.id == id) {
            key.failure_count += 1;
            key.last_used = Some(Utc::now()); // Mark as used to put it at end of rotation
            if key.failure_count > 5 {
                warn!("Deactivating API key {} due to excessive failures. Cooldown initiated.", id);
                key.is_active = false;
            }
        }
    }

    pub async fn report_rate_limit(&self, id: &str, retry_after_seconds: u64) {
        let mut keys = self.keys.write().await;
        if let Some(key) = keys.iter_mut().find(|k| k.id == id) {
            let until = Utc::now() + chrono::Duration::seconds(retry_after_seconds as i64);
            info!("Key {} rate limited until {}", id, until);
            key.rate_limit_until = Some(until);
        }
    }

    pub async fn get_status(&self) -> Vec<ApiKey> {
        self.keys.read().await.clone()
    }
}
