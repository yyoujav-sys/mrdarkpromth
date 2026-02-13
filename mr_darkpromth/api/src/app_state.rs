use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock};
use sqlx::PgPool;
use mr_darkpromth_services::{
    UserService, CerebrasClient, RedisCoordinator, ToolRegistry, MasterToolExecutor,
    UltraTierLogic, SandboxManager, SandboxConfig,
    JailbreakPromptService, BillingService, EmailService, AuditLogger, DatabaseAuditService,
    AutoDocService, TelemetryService, LearningSystem, KeyPool, SlackService, CloudflareService
};
use crate::event_hub::EventHub;

/// Global application state shared across all handlers
pub struct AppState {
    pub user_service: Arc<UserService>,
    pub cerebras_client: Arc<CerebrasClient>,
    pub redis_coordinator: Option<Arc<Mutex<RedisCoordinator>>>,
    pub tool_registry: Arc<ToolRegistry>,
    pub tool_executor: Arc<MasterToolExecutor>,
    pub ultra_tier_logic: Arc<RwLock<UltraTierLogic>>,
    pub sandbox_manager: Arc<SandboxManager>,
    pub jailbreak_service: Arc<JailbreakPromptService>,
    pub billing_service: Arc<BillingService>,
    pub email_service: Arc<EmailService>,
    pub audit_logger: Arc<AuditLogger>,
    pub auto_doc_service: Arc<AutoDocService>,
    pub telemetry_service: Arc<TelemetryService>,
    pub learning_system: Arc<Mutex<LearningSystem>>,
    pub metrics: Arc<Metrics>,
    pub key_pool: Arc<KeyPool>,
    pub event_hub: Arc<EventHub>,
    pub slack_service: Arc<SlackService>,
    pub cloudflare_service: Arc<CloudflareService>,
    pub pool: PgPool,
}

#[derive(Default)]
pub struct Metrics {
    pub users_total: std::sync::atomic::AtomicU64,
    pub conversations_total: std::sync::atomic::AtomicU64,
}

impl AppState {
    pub async fn new(pool: PgPool) -> Self {
        let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        let user_service = Arc::new(UserService::new(mr_darkpromth_db::UserRepository::new(pool.clone()), jwt_secret.clone(), redis_coordinator));
        let cerebras_client = Arc::new(CerebrasClient::new());
        
        let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
        let redis_coordinator = match RedisCoordinator::new(&redis_url, "api_gateway".to_string()) {
            Ok(c) => Some(Arc::new(Mutex::new(c))),
            Err(e) => {
                log::error!("Failed to connect to Redis: {}", e);
                None
            }
        };
        
        let jailbreak_service = Arc::new(JailbreakPromptService::new(pool.clone()));
        let billing_service = Arc::new(BillingService::new_with_pool(pool.clone()));
        let email_service = Arc::new(EmailService::new_from_env(pool.clone()).expect("EmailService failed"));
        let audit_logger = Arc::new(AuditLogger::new(Box::new(DatabaseAuditService::new(pool.clone()))));
        
        let cerebras_api_key = std::env::var("CEREBRAS_API_KEY").unwrap_or_default();
        let auto_doc_service = Arc::new(AutoDocService::new(cerebras_api_key, std::path::PathBuf::from("/app/mr_darkpromth")));
        let telemetry_service = Arc::new(TelemetryService::new("api_gateway".to_string()));
        let learning_system = Arc::new(Mutex::new(LearningSystem::new(pool.clone())));
        
        // Initialize new services
        let slack_service = Arc::new(SlackService::new_from_env());
        let cloudflare_service = Arc::new(CloudflareService::new_from_env());
        
        // Start background telemetry collection
        let telemetry_clone = telemetry_service.clone();
        tokio::spawn(async move {
            telemetry_clone.start_background_collection().await;
        });
        
        // Initialize and populate KeyPool
        let key_pool = Arc::new(KeyPool::new());
        
        // Load Cerebras keys
        let cerebras_keys = std::env::var("CEREBRAS_API_KEYS").unwrap_or_default();
        if !cerebras_keys.is_empty() {
            for (idx, key) in cerebras_keys.split(',').enumerate() {
                if !key.trim().is_empty() {
                    key_pool.add_key(
                        mr_darkpromth_services::key_pool::Provider::Cerebras,
                        key.trim().to_string(),
                        format!("Cerebras-{}", idx + 1)
                    ).await;
                }
            }
            log::info!("✅ Loaded {} Cerebras API keys into the pool", cerebras_keys.split(',').count());
        }
        
        // Load OpenRouter keys
        let openrouter_keys = std::env::var("OPENROUTER_API_KEYS").unwrap_or_default();
        if !openrouter_keys.is_empty() {
            for (idx, key) in openrouter_keys.split(',').enumerate() {
                if !key.trim().is_empty() {
                    key_pool.add_key(
                        mr_darkpromth_services::key_pool::Provider::OpenRouter,
                        key.trim().to_string(),
                        format!("OpenRouter-{}", idx + 1)
                    ).await;
                }
            }
            log::info!("✅ Loaded {} OpenRouter API keys into the pool", openrouter_keys.split(',').count());
        }
        
        let ultra_tier_logic = Arc::new(RwLock::new(UltraTierLogic::with_config(
            "./memory/ultra_tier_audit.log",
            key_pool.clone(),
            jailbreak_service.clone()
        )));

        let sandbox_manager = Arc::new(SandboxManager::new(SandboxConfig::default()));
        
        let metrics = Arc::new(Metrics::default());
        {
            let users_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
                .fetch_one(&pool)
                .await
                .unwrap_or(0);
            let conversations_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM chat_sessions")
                .fetch_one(&pool)
                .await
                .unwrap_or(0);
            metrics.users_total.store(users_count as u64, std::sync::atomic::Ordering::Relaxed);
            metrics.conversations_total.store(conversations_count as u64, std::sync::atomic::Ordering::Relaxed);
        }

        Self {
            user_service,
            cerebras_client,
            redis_coordinator,
            tool_registry: Arc::new(ToolRegistry::new(Default::default())),
            tool_executor: Arc::new(MasterToolExecutor::new(Default::default())),
            ultra_tier_logic,
            sandbox_manager,
            jailbreak_service,
            billing_service,
            email_service,
            audit_logger,
            auto_doc_service,
            telemetry_service,
            learning_system,
            metrics,
            key_pool,
            event_hub: Arc::new(EventHub::new()),
            slack_service,
            cloudflare_service,
            pool,
        }
    }
}
