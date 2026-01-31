use std::env;
use std::time::Duration;

use crate::error::CerebrasClientError;
use crate::models::Model;

pub const DEFAULT_BASE_URL: &str = "https://api.cerebras.ai/v1";
pub const DEFAULT_TIMEOUT_SECS: u64 = 60;
pub const DEFAULT_MAX_RETRIES: u32 = 3;
pub const DEFAULT_RETRY_BACKOFF_MS: u64 = 500;
pub const DEFAULT_MAX_CONCURRENT_REQUESTS: usize = 8;
pub const DEFAULT_QUEUE_CAPACITY: usize = 256;
pub const DEFAULT_KEY_COOLDOWN_SECS: u64 = 60;

#[derive(Debug, Clone)]
pub struct CerebrasConfig {
    pub api_keys: Vec<String>,
    pub base_url: String,
    pub timeout_secs: u64,
    pub default_model: Model,
    pub max_retries: u32,
    pub retry_backoff_base_ms: u64,
    pub max_concurrent_requests: usize,
    pub queue_capacity: usize,
    pub key_cooldown_secs: u64,
}

impl CerebrasConfig {
    pub fn new(api_keys: Vec<String>) -> Result<Self, CerebrasClientError> {
        if api_keys.is_empty() {
            return Err(CerebrasClientError::MissingApiKey);
        }

        Ok(Self {
            api_keys,
            base_url: DEFAULT_BASE_URL.to_string(),
            timeout_secs: DEFAULT_TIMEOUT_SECS,
            default_model: Model::Llama3_3_70b,
            max_retries: DEFAULT_MAX_RETRIES,
            retry_backoff_base_ms: DEFAULT_RETRY_BACKOFF_MS,
            max_concurrent_requests: DEFAULT_MAX_CONCURRENT_REQUESTS,
            queue_capacity: DEFAULT_QUEUE_CAPACITY,
            key_cooldown_secs: DEFAULT_KEY_COOLDOWN_SECS,
        })
    }

    pub fn from_env() -> Result<Self, CerebrasClientError> {
        let api_keys = env::var("CEREBRAS_API_KEYS")
            .map_err(|_| CerebrasClientError::MissingApiKey)?
            .split(',')
            .map(|key| key.trim().to_string())
            .filter(|key| !key.is_empty())
            .collect::<Vec<_>>();

        if api_keys.is_empty() {
            return Err(CerebrasClientError::MissingApiKey);
        }

        let base_url = env::var("CEREBRAS_BASE_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());
        let timeout_secs = parse_env_timeout("CEREBRAS_TIMEOUT_SECS")?.unwrap_or(DEFAULT_TIMEOUT_SECS);
        let max_retries = parse_env_u32("CEREBRAS_MAX_RETRIES")?.unwrap_or(DEFAULT_MAX_RETRIES);
        let retry_backoff_base_ms =
            parse_env_backoff("CEREBRAS_RETRY_BACKOFF_MS")?.unwrap_or(DEFAULT_RETRY_BACKOFF_MS);
        let max_concurrent_requests = parse_env_usize("CEREBRAS_MAX_CONCURRENT_REQUESTS")?
            .unwrap_or(DEFAULT_MAX_CONCURRENT_REQUESTS);
        let queue_capacity = parse_env_usize("CEREBRAS_QUEUE_CAPACITY")?
            .unwrap_or(DEFAULT_QUEUE_CAPACITY);
        let key_cooldown_secs = parse_env_cooldown("CEREBRAS_KEY_COOLDOWN_SECS")?
            .unwrap_or(DEFAULT_KEY_COOLDOWN_SECS);

        let default_model = env::var("CEREBRAS_DEFAULT_MODEL")
            .ok()
            .as_deref()
            .and_then(Model::parse)
            .unwrap_or(Model::Llama3_3_70b);

        Ok(Self {
            api_keys,
            base_url,
            timeout_secs,
            default_model,
            max_retries,
            retry_backoff_base_ms,
            max_concurrent_requests,
            queue_capacity,
            key_cooldown_secs,
        })
    }

    pub fn timeout(&self) -> Duration {
        Duration::from_secs(self.timeout_secs)
    }
}

fn parse_env_timeout(key: &str) -> Result<Option<u64>, CerebrasClientError> {
    match env::var(key) {
        Ok(value) => value
            .parse::<u64>()
            .map(Some)
            .map_err(|_| CerebrasClientError::InvalidTimeout(value)),
        Err(_) => Ok(None),
    }
}

fn parse_env_backoff(key: &str) -> Result<Option<u64>, CerebrasClientError> {
    match env::var(key) {
        Ok(value) => value
            .parse::<u64>()
            .map(Some)
            .map_err(|_| CerebrasClientError::InvalidBackoff(value)),
        Err(_) => Ok(None),
    }
}

fn parse_env_cooldown(key: &str) -> Result<Option<u64>, CerebrasClientError> {
    match env::var(key) {
        Ok(value) => value
            .parse::<u64>()
            .map(Some)
            .map_err(|_| CerebrasClientError::InvalidCooldown(value)),
        Err(_) => Ok(None),
    }
}

fn parse_env_u32(key: &str) -> Result<Option<u32>, CerebrasClientError> {
    match env::var(key) {
        Ok(value) => value
            .parse::<u32>()
            .map(Some)
            .map_err(|_| CerebrasClientError::InvalidRetryCount(value)),
        Err(_) => Ok(None),
    }
}

fn parse_env_usize(key: &str) -> Result<Option<usize>, CerebrasClientError> {
    match env::var(key) {
        Ok(value) => value
            .parse::<usize>()
            .map(Some)
            .map_err(|_| CerebrasClientError::InvalidCapacity(value)),
        Err(_) => Ok(None),
    }
}
