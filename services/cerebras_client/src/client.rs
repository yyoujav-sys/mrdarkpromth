use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use futures_util::stream::BoxStream;
use futures_util::{Stream, StreamExt};
use reqwest::Client as HttpClient;
use tokio::sync::{mpsc, oneshot, Mutex, Semaphore};
use tokio::time::sleep;

use crate::config::CerebrasConfig;
use crate::error::CerebrasClientError;
use crate::models::{Model, RequestIntent};
use crate::types::{
    ChatCompletionChunk, ChatCompletionRequest, ChatCompletionResponse, ChatMessage, ChatRequest, Role,
    Usage,
};

#[derive(Debug, Clone)]
pub enum PromptTemplate {
    Chat,
    Code,
    Reasoning,
}

impl PromptTemplate {
    pub fn from_intent(intent: &RequestIntent) -> Self {
        match intent {
            RequestIntent::Chat => PromptTemplate::Chat,
            RequestIntent::Code => PromptTemplate::Code,
            RequestIntent::Reasoning => PromptTemplate::Reasoning,
        }
    }

    pub fn system_prompt(&self) -> &'static str {
        match self {
            PromptTemplate::Chat => "You are a helpful assistant. Provide concise and accurate answers.",
            PromptTemplate::Code => "You are a senior software engineer. Respond with clear code and brief explanations.",
            PromptTemplate::Reasoning => "You are an expert analyst. Provide step-by-step reasoning and a final answer.",
        }
    }
}

#[derive(Debug, Clone)]
pub struct UsageTotals {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
    pub last_updated_epoch: u64,
}

#[derive(Debug, Clone)]
pub struct UsageSnapshot {
    pub per_key: HashMap<String, UsageTotals>,
    pub updated_epoch: u64,
}

#[derive(Clone)]
pub struct CerebrasClient {
    inner: Arc<Inner>,
}

struct Inner {
    http: HttpClient,
    config: CerebrasConfig,
    key_manager: Mutex<KeyManager>,
    usage: Mutex<UsageTracker>,
    semaphore: Semaphore,
    queue_tx: mpsc::Sender<ChatJob>,
}

struct ChatJob {
    request: ChatRequest,
    response_tx: oneshot::Sender<Result<ChatCompletionResponse, CerebrasClientError>>,
}

impl CerebrasClient {
    pub fn new(config: CerebrasConfig) -> Self {
        let http = HttpClient::builder()
            .timeout(config.timeout())
            .build()
            .unwrap_or_default();
        let key_manager = KeyManager::new(config.api_keys.clone(), config.key_cooldown_secs);
        let usage = UsageTracker::default();
        let semaphore = Semaphore::new(config.max_concurrent_requests.max(1));
        let (queue_tx, queue_rx) = mpsc::channel(config.queue_capacity.max(1));

        let inner = Arc::new(Inner {
            http,
            config,
            key_manager: Mutex::new(key_manager),
            usage: Mutex::new(usage),
            semaphore,
            queue_tx,
        });

        CerebrasClient::spawn_queue_workers(inner.clone(), queue_rx);

        Self { inner }
    }

    pub fn from_env() -> Result<Self, CerebrasClientError> {
        let config = CerebrasConfig::from_env()?;
        Ok(Self::new(config))
    }

    pub async fn chat_completion(
        &self,
        request: ChatRequest,
    ) -> Result<ChatCompletionResponse, CerebrasClientError> {
        if request.stream.unwrap_or(false) {
            return Err(CerebrasClientError::StreamingNotSupported);
        }

        let (response_tx, response_rx) = oneshot::channel();
        if self
            .inner
            .queue_tx
            .try_send(ChatJob { request, response_tx })
            .is_err()
        {
            return Err(CerebrasClientError::QueueFull);
        }

        response_rx.await.map_err(|_| CerebrasClientError::QueueFull)?
    }

    pub async fn chat_completion_stream(
        &self,
        request: ChatRequest,
    ) -> Result<BoxStream<'static, Result<ChatCompletionChunk, CerebrasClientError>>, CerebrasClientError> {
        let (completion_request, _) = self.inner.prepare_request(request, true)?;
        let url = format!("{}/chat/completions", self.inner.config.base_url);

        let key = self.next_key().await?;
        let response = self
            .inner
            .http
            .post(url)
            .bearer_auth(&key)
            .json(&completion_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            self.mark_failure(&key).await;
            return Err(CerebrasClientError::HttpStatus {
                status,
                body,
            });
        }

        self.mark_success(&key).await;

        let stream = response.bytes_stream();
        let parsed = Self::parse_stream(stream);
        Ok(parsed.boxed())
    }

    pub async fn usage_snapshot(&self) -> UsageSnapshot {
        let tracker = self.inner.usage.lock().await;
        tracker.snapshot()
    }

    fn spawn_queue_workers(inner: Arc<Inner>, mut queue_rx: mpsc::Receiver<ChatJob>) {
        tokio::spawn(async move {
            while let Some(job) = queue_rx.recv().await {
                let inner = inner.clone();
                tokio::spawn(async move {
                    let permit = inner.semaphore.acquire().await;
                    if permit.is_err() {
                        let _ = job.response_tx.send(Err(CerebrasClientError::QueueFull));
                        return;
                    }
                    let _permit = permit.unwrap();
                    let result = inner.execute_chat(job.request).await;
                    let _ = job.response_tx.send(result);
                });
            }
        });
    }

    async fn next_key(&self) -> Result<String, CerebrasClientError> {
        let mut manager = self.inner.key_manager.lock().await;
        manager.next_key()
    }

    async fn mark_success(&self, key: &str) {
        let mut manager = self.inner.key_manager.lock().await;
        manager.mark_success(key);
    }

    async fn mark_failure(&self, key: &str) {
        let mut manager = self.inner.key_manager.lock().await;
        manager.mark_failure(key);
    }

    fn parse_stream(
        stream: impl Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Send + 'static,
    ) -> impl Stream<Item = Result<ChatCompletionChunk, CerebrasClientError>> {
        let stream = Box::pin(stream);
        futures_util::stream::unfold((stream, String::new()), |(mut byte_stream, mut buffer)| async move {
            loop {
                match byte_stream.as_mut().next().await {
                    Some(Ok(bytes)) => {
                        let chunk = String::from_utf8_lossy(&bytes);
                        buffer.push_str(&chunk);

                        while let Some(newline_pos) = buffer.find('\n') {
                            let line = buffer[..newline_pos].trim().to_string();
                            buffer = buffer[newline_pos + 1..].to_string();

                            if line.is_empty() || !line.starts_with("data:") {
                                continue;
                            }

                            let payload = line.trim_start_matches("data:").trim();
                            if payload == "[DONE]" {
                                return None;
                            }

                            if payload.is_empty() {
                                continue;
                            }

                            let parsed = serde_json::from_str::<ChatCompletionChunk>(payload)
                                .map_err(CerebrasClientError::Serialization);
                            return Some((parsed, (byte_stream, buffer)));
                        }
                    }
                    Some(Err(error)) => {
                        return Some((Err(CerebrasClientError::RequestFailed(error)), (byte_stream, buffer)));
                    }
                    None => return None,
                }
            }
        })
    }
}

impl Inner {
    async fn execute_chat(
        &self,
        request: ChatRequest,
    ) -> Result<ChatCompletionResponse, CerebrasClientError> {
        let (completion_request, _) = self.prepare_request(request, false)?;
        let url = format!("{}/chat/completions", self.config.base_url);

        let mut attempt = 0;
        loop {
            let key = {
                let mut manager = self.key_manager.lock().await;
                manager.next_key()
            }?;

            let response = self
                .http
                .post(&url)
                .bearer_auth(&key)
                .json(&completion_request)
                .send()
                .await;

            match response {
                Ok(resp) if resp.status().is_success() => {
                    let payload: ChatCompletionResponse = resp.json().await?;
                    if let Some(usage) = payload.usage.clone() {
                        let mut tracker = self.usage.lock().await;
                        tracker.record_usage(&key, usage);
                    }
                    let mut manager = self.key_manager.lock().await;
                    manager.mark_success(&key);
                    return Ok(payload);
                }
                Ok(resp) => {
                    let status = resp.status();
                    let body = resp.text().await.unwrap_or_default();
                    let mut manager = self.key_manager.lock().await;
                    manager.mark_failure(&key);

                    if status == reqwest::StatusCode::TOO_MANY_REQUESTS || status.is_server_error() {
                        if attempt < self.config.max_retries {
                            let backoff = backoff_duration(self.config.retry_backoff_base_ms, attempt);
                            attempt += 1;
                            sleep(backoff).await;
                            continue;
                        }
                    }

                    return Err(CerebrasClientError::HttpStatus { status, body });
                }
                Err(error) => {
                    let mut manager = self.key_manager.lock().await;
                    manager.mark_failure(&key);

                    if attempt < self.config.max_retries {
                        let backoff = backoff_duration(self.config.retry_backoff_base_ms, attempt);
                        attempt += 1;
                        sleep(backoff).await;
                        continue;
                    }

                    return Err(CerebrasClientError::RequestFailed(error));
                }
            }
        }
    }

    fn prepare_request(
        &self,
        mut request: ChatRequest,
        force_stream: bool,
    ) -> Result<(ChatCompletionRequest, Model), CerebrasClientError> {
        let model = self.resolve_model(&request);
        Self::apply_prompt_template(&mut request.messages, request.intent.as_ref());

        let completion_request = ChatCompletionRequest {
            model: model.as_str().to_string(),
            messages: request.messages,
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            top_p: request.top_p,
            stream: Some(force_stream),
        };

        Ok((completion_request, model))
    }

    fn resolve_model(&self, request: &ChatRequest) -> Model {
        if let Some(model) = request.model.clone() {
            return model;
        }

        if let Some(intent) = request.intent.as_ref() {
            return intent.default_model();
        }

        self.config.default_model.clone()
    }

    fn apply_prompt_template(messages: &mut Vec<ChatMessage>, intent: Option<&RequestIntent>) {
        let Some(intent) = intent else {
            return;
        };

        let has_system = messages.iter().any(|message| matches!(message.role, Role::System));
        if has_system {
            return;
        }

        let template = PromptTemplate::from_intent(intent);
        messages.insert(
            0,
            ChatMessage {
                role: Role::System,
                content: template.system_prompt().to_string(),
            },
        );
    }
}

impl Clone for Inner {
    fn clone(&self) -> Self {
        Self {
            http: self.http.clone(),
            config: self.config.clone(),
            key_manager: Mutex::new(KeyManager::new(
                self.config.api_keys.clone(),
                self.config.key_cooldown_secs,
            )),
            usage: Mutex::new(UsageTracker::default()),
            semaphore: Semaphore::new(self.config.max_concurrent_requests.max(1)),
            queue_tx: self.queue_tx.clone(),
        }
    }
}

struct KeyManager {
    keys: Vec<ApiKeyState>,
    next_index: usize,
    cooldown: Duration,
}

struct ApiKeyState {
    key: String,
    failure_count: u32,
    cooldown_until: Option<Instant>,
}

impl KeyManager {
    fn new(keys: Vec<String>, cooldown_secs: u64) -> Self {
        let keys = keys
            .into_iter()
            .map(|key| ApiKeyState {
                key,
                failure_count: 0,
                cooldown_until: None,
            })
            .collect();

        Self {
            keys,
            next_index: 0,
            cooldown: Duration::from_secs(cooldown_secs.max(1)),
        }
    }

    fn next_key(&mut self) -> Result<String, CerebrasClientError> {
        if self.keys.is_empty() {
            return Err(CerebrasClientError::NoHealthyKeys);
        }

        let total = self.keys.len();
        for _ in 0..total {
            let index = self.next_index % total;
            self.next_index = (self.next_index + 1) % total;

            let key_state = &mut self.keys[index];
            if let Some(until) = key_state.cooldown_until {
                if Instant::now() < until {
                    continue;
                }
                key_state.cooldown_until = None;
            }

            return Ok(key_state.key.clone());
        }

        Err(CerebrasClientError::NoHealthyKeys)
    }

    fn mark_failure(&mut self, key: &str) {
        if let Some(key_state) = self.keys.iter_mut().find(|state| state.key == key) {
            key_state.failure_count += 1;
            key_state.cooldown_until = Some(Instant::now() + self.cooldown);
        }
    }

    fn mark_success(&mut self, key: &str) {
        if let Some(key_state) = self.keys.iter_mut().find(|state| state.key == key) {
            key_state.failure_count = 0;
            key_state.cooldown_until = None;
        }
    }
}

#[derive(Default)]
struct UsageTracker {
    totals: HashMap<String, UsageTotals>,
}

impl UsageTracker {
    fn record_usage(&mut self, key: &str, usage: Usage) {
        let entry = self.totals.entry(key.to_string()).or_insert(UsageTotals {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
            last_updated_epoch: 0,
        });

        entry.prompt_tokens += usage.prompt_tokens as u64;
        entry.completion_tokens += usage.completion_tokens as u64;
        entry.total_tokens += usage.total_tokens as u64;
        entry.last_updated_epoch = current_epoch();
    }

    fn snapshot(&self) -> UsageSnapshot {
        UsageSnapshot {
            per_key: self.totals.clone(),
            updated_epoch: current_epoch(),
        }
    }
}

fn current_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn backoff_duration(base_ms: u64, attempt: u32) -> Duration {
    let shift = attempt.min(10);
    let multiplier = 1u64.checked_shl(shift).unwrap_or(u64::MAX);
    Duration::from_millis(base_ms.saturating_mul(multiplier))
}
