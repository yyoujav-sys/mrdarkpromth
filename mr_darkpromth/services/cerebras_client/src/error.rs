use thiserror::Error;

#[derive(Debug, Error)]
pub enum CerebrasClientError {
    #[error("missing Cerebras API key")]
    MissingApiKey,
    #[error("invalid base url: {0}")]
    InvalidBaseUrl(String),
    #[error("invalid default model: {0}")]
    InvalidDefaultModel(String),
    #[error("invalid timeout seconds: {0}")]
    InvalidTimeout(String),
    #[error("invalid max retries: {0}")]
    InvalidRetryCount(String),
    #[error("invalid retry backoff milliseconds: {0}")]
    InvalidBackoff(String),
    #[error("invalid capacity value: {0}")]
    InvalidCapacity(String),
    #[error("invalid cooldown seconds: {0}")]
    InvalidCooldown(String),
    #[error("request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("unexpected status {status}: {body}")]
    HttpStatus {
        status: reqwest::StatusCode,
        body: String,
    },
    #[error("streaming responses are not supported by this method")]
    StreamingNotSupported,
    #[error("no healthy Cerebras API keys available")]
    NoHealthyKeys,
    #[error("request queue is full")]
    QueueFull,
}
