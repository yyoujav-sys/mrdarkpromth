// Cerebras.ai Integration Module (Agent 3 client wrapper)
// Shared facade for agents and services

use anyhow::{Result, anyhow};
use cerebras_client::{
    CerebrasClient as CoreClient, CerebrasConfig, ChatMessage, ChatRequest, Model, RequestIntent, Role,
};
use log::{info, warn};
use std::env;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct CerebrasClient {
    inner: CoreClient,
    default_model: Arc<Mutex<Model>>,
}

fn parse_env_u64(key: &str) -> Option<u64> {
    env::var(key).ok().and_then(|value| value.parse::<u64>().ok())
}

fn parse_env_u32(key: &str) -> Option<u32> {
    env::var(key).ok().and_then(|value| value.parse::<u32>().ok())
}

fn parse_env_usize(key: &str) -> Option<usize> {
    env::var(key).ok().and_then(|value| value.parse::<usize>().ok())
}

impl CerebrasClient {
    pub fn new() -> Self {
        Self::from_env_or_default()
    }

    pub fn with_api_key(api_key: String) -> Self {
        Self::from_keys(vec![api_key])
    }

    pub async fn generate(&self, prompt: &str) -> Result<String> {
        self.chat_completion(prompt, None).await
    }

    pub async fn chat_completion(&self, prompt: &str, jailbreak_prompt: Option<&str>) -> Result<String> {
        self.chat_completion_with_model(prompt, jailbreak_prompt, None).await
    }

    pub async fn chat_completion_with_model(
        &self,
        prompt: &str,
        jailbreak_prompt: Option<&str>,
        model_override: Option<&str>,
    ) -> Result<String> {
        let mut messages = Vec::new();
        if let Some(system_prompt) = jailbreak_prompt {
            messages.push(ChatMessage {
                role: Role::System,
                content: system_prompt.to_string(),
            });
        }
        messages.push(ChatMessage {
            role: Role::User,
            content: prompt.to_string(),
        });

        self.execute_chat_with_model(messages, Some(RequestIntent::Chat), model_override)
            .await
    }

    pub async fn chat_completion_with_system(&self, system_prompt: &str, user_prompt: &str) -> Result<String> {
        let messages = vec![
            ChatMessage {
                role: Role::System,
                content: system_prompt.to_string(),
            },
            ChatMessage {
                role: Role::User,
                content: user_prompt.to_string(),
            },
        ];

        self.execute_chat(messages, Some(RequestIntent::Reasoning)).await
    }

    pub async fn usage_snapshot(&self) -> cerebras_client::UsageSnapshot {
        self.inner.usage_snapshot().await
    }

    pub fn set_model(&self, model: String) {
        if let Some(parsed) = Model::parse(&model) {
            if let Ok(mut guard) = self.default_model.lock() {
                *guard = parsed;
            }
        } else {
            warn!("Unknown Cerebras model requested: {}", model);
        }
    }

    pub fn get_model(&self) -> String {
        self.default_model
            .lock()
            .map(|model| model.as_str().to_string())
            .unwrap_or_else(|_| "llama-3.3-70b".to_string())
    }

    pub fn resolve_model_name(&self, model_override: Option<&str>) -> String {
        self.resolve_model(model_override).as_str().to_string()
    }

    async fn execute_chat(&self, messages: Vec<ChatMessage>, intent: Option<RequestIntent>) -> Result<String> {
        self.execute_chat_with_model(messages, intent, None).await
    }

    async fn execute_chat_with_model(
        &self,
        messages: Vec<ChatMessage>,
        intent: Option<RequestIntent>,
        model_override: Option<&str>,
    ) -> Result<String> {
        let model = self.resolve_model(model_override);
        let request = ChatRequest {
            messages,
            model: Some(model),
            max_tokens: Some(4096),
            temperature: Some(0.7),
            top_p: Some(0.9),
            stream: Some(false),
            intent,
        };

        let response = self.inner.chat_completion(request).await?;

        let choice = response
            .choices
            .first()
            .ok_or_else(|| anyhow!("No choices in Cerebras response"))?;

        Ok(choice.message.content.clone())
    }

    fn resolve_model(&self, model_override: Option<&str>) -> Model {
        let default_model = self
            .default_model
            .lock()
            .map(|model| model.clone())
            .unwrap_or(Model::Llama3_3_70b);

        match model_override.and_then(Model::parse) {
            Some(parsed) => parsed,
            None => {
                if let Some(model) = model_override {
                    warn!("Unknown Cerebras model requested: {}", model);
                }
                default_model
            }
        }
    }

    fn from_env_or_default() -> Self {
        let keys = load_keys_from_env();
        if keys.is_empty() {
            warn!("CEREBRAS_API_KEYS/CEREBRAS_API_KEY not set; using placeholder key");
        }
        Self::from_keys(if keys.is_empty() { vec!["missing_key".to_string()] } else { keys })
    }

    fn from_keys(keys: Vec<String>) -> Self {
        let mut config = CerebrasConfig::new(keys).unwrap_or_else(|_| {
            CerebrasConfig::new(vec!["missing_key".to_string()]).expect("fallback config")
        });
        apply_env_overrides(&mut config);
        let default_model = Arc::new(Mutex::new(config.default_model.clone()));
        let inner = CoreClient::new(config);
        info!("Cerebras client initialized");
        Self { inner, default_model }
    }
}

fn load_keys_from_env() -> Vec<String> {
    if let Ok(keys) = env::var("CEREBRAS_API_KEYS") {
        return keys
            .split(',')
            .map(|key| key.trim().to_string())
            .filter(|key| !key.is_empty())
            .collect();
    }

    if let Ok(key) = env::var("CEREBRAS_API_KEY") {
        let trimmed = key.trim().to_string();
        if !trimmed.is_empty() {
            return vec![trimmed];
        }
    }

    Vec::new()
}

fn apply_env_overrides(config: &mut CerebrasConfig) {
    if let Ok(base_url) = env::var("CEREBRAS_BASE_URL") {
        config.base_url = base_url;
    }
    if let Some(timeout) = parse_env_u64("CEREBRAS_TIMEOUT_SECS") {
        config.timeout_secs = timeout;
    }
    if let Some(max_retries) = parse_env_u32("CEREBRAS_MAX_RETRIES") {
        config.max_retries = max_retries;
    }
    if let Some(backoff) = parse_env_u64("CEREBRAS_RETRY_BACKOFF_MS") {
        config.retry_backoff_base_ms = backoff;
    }
    if let Some(concurrency) = parse_env_usize("CEREBRAS_MAX_CONCURRENT_REQUESTS") {
        config.max_concurrent_requests = concurrency;
    }
    if let Some(capacity) = parse_env_usize("CEREBRAS_QUEUE_CAPACITY") {
        config.queue_capacity = capacity;
    }
    if let Some(cooldown) = parse_env_u64("CEREBRAS_KEY_COOLDOWN_SECS") {
        config.key_cooldown_secs = cooldown;
    }
    if let Ok(model) = env::var("CEREBRAS_DEFAULT_MODEL") {
        if let Some(parsed) = Model::parse(&model) {
            config.default_model = parsed;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_model_override() {
        let client = CerebrasClient::with_api_key("test".to_string());
        assert_eq!(client.get_model(), "llama-3.3-70b");
    }
}
