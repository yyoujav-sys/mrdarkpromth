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
}
