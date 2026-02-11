use std::sync::{Arc, atomic::AtomicU64};
use tokio::sync::RwLock;
use std::time::Duration;
use tokio::sync::Mutex;
use mr_darkpromth_services::{
    RedisCoordinator, ToolRegistry, UltraTierLogic, UserService,
    AutoDocService, TelemetryService, SandboxManager, SandboxedExecutorConfig,
    CerebrasClient, MasterToolExecutor, JailbreakPromptService, BillingService, EmailService,
};
use mr_darkpromth_services::LearningSystem;
use mr_darkpromth_services::AuditLogger;
use mr_darkpromth_services::DatabaseAuditService;
use mr_darkpromth_db::UserRepository;

#[derive(Clone)]
pub struct AppState {
    pub user_service: Arc<UserService>,
    pub cerebras_client: CerebrasClient,
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
    pub key_pool: Arc<mr_darkpromth_services::KeyPool>,
    pub pool: sqlx::PgPool,
}

pub struct Metrics {
    pub users_total: AtomicU64,
    pub conversations_total: AtomicU64,
    pub requests_total: AtomicU64,
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            users_total: AtomicU64::new(0),
            conversations_total: AtomicU64::new(0),
            requests_total: AtomicU64::new(0),
        }
    }
}

/// Validate critical environment variables before starting server
fn validate_environment() {
    use log::error;
    
    let mut missing = Vec::new();
    
    // Check JWT_SECRET
    if std::env::var("JWT_SECRET").unwrap_or_default().is_empty() {
        missing.push("JWT_SECRET");
    }
    
    // Check Database URL
    if std::env::var("DATABASE_URL").unwrap_or_default().is_empty() {
        missing.push("DATABASE_URL");
    }
    
    // Check AI Brain Keys (Cerebras or OpenRouter for failover)
    let cerebras_keys = std::env::var("CEREBRAS_API_KEYS").unwrap_or_default();
    let cerebras_key = std::env::var("CEREBRAS_API_KEY").unwrap_or_default();
    let openrouter_keys = std::env::var("OPENROUTER_API_KEYS").unwrap_or_default();
    if cerebras_keys.is_empty() && cerebras_key.is_empty() && openrouter_keys.is_empty() {
        missing.push("CEREBRAS_API_KEYS (or OPENROUTER_API_KEYS for failover)");
    }
    
    if !missing.is_empty() {
        error!("❌ CRITICAL: Missing required environment variables:");
        for var in &missing {
            error!("   - {}", var);
        }
        error!("");
        error!("Please set these environment variables before starting the server.");
        error!("The system will now exit.");
        std::process::exit(1);
    }
    
    log::info!("✅ All required environment variables are set");
}

impl AppState {
    pub async fn new(pool: sqlx::PgPool) -> Self {
        // Validate environment variables first
        validate_environment();
        
        let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_default();
        let user_repo = UserRepository::new(pool.clone());
        let redis_coordinator = {
            let redis_url = std::env::var("REDIS_URL").ok();
            if let Some(url) = redis_url {
                match RedisCoordinator::new(&url, "api_server".to_string()) {
                    Ok(coordinator) => Some(Arc::new(Mutex::new(coordinator))),
                    Err(e) => {
                        log::error!("Failed to connect to Redis: {}", e);
                        None
                    }
                }
            } else {
                log::warn!("REDIS_URL not set, Redis features will be disabled.");
                None
            }
        };

        let user_service = Arc::new(UserService::new(user_repo, jwt_secret, redis_coordinator.clone()));
        let cerebras_client = CerebrasClient::new(); // Reads from env internally
        let jailbreak_service = Arc::new(JailbreakPromptService::new(pool.clone()));
        let billing_service = Arc::new(BillingService::new_with_pool(pool.clone()));
        let email_service = Arc::new(EmailService::new_from_env(pool.clone()).expect("EmailService failed"));
        let audit_logger = Arc::new(AuditLogger::new(Box::new(DatabaseAuditService::new(pool.clone()))));
        let cerebras_api_key = std::env::var("CEREBRAS_API_KEY").unwrap_or_default();
        let auto_doc_service = Arc::new(AutoDocService::new(cerebras_api_key, std::path::PathBuf::from("/app/mr_darkpromth")));
        let telemetry_service = Arc::new(TelemetryService::new("api_gateway".to_string()));
        let learning_system = Arc::new(Mutex::new(LearningSystem::new(pool.clone())));
        
        // Start background telemetry collection
        let telemetry_clone = telemetry_service.clone();
        tokio::spawn(async move {
            telemetry_clone.start_background_collection().await;
        });
        
        // Initialize and populate KeyPool
        let key_pool = Arc::new(mr_darkpromth_services::KeyPool::new());
        
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
        
        // Load OpenRouter keys for High Availability Failover
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
            log::info!("✅ Loaded {} OpenRouter API keys into the pool (Failover enabled)", openrouter_keys.split(',').count());
        } else {
            log::warn!("⚠️  OPENROUTER_API_KEYS not found - High Availability Failover disabled!");
        }

        let ultra_tier_logic = Arc::new(RwLock::new(UltraTierLogic::with_config(
            "./memory/ultra_tier_audit.log",
            key_pool.clone(),
            jailbreak_service.clone()
        )));

        // Initialize sandbox manager with periodic cleanup
        let sandbox_manager = {
            let manager = Arc::new(SandboxManager::new(SandboxedExecutorConfig::default()));
            let manager_clone = manager.clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(300)); // Clean up every 5 minutes
                loop {
                    interval.tick().await;
                    let cleaned = manager_clone.cleanup_expired_sessions().await;
                    if cleaned > 0 {
                        log::info!("Cleaned up {} expired sandbox sessions", cleaned);
                    }
                }
            });
            manager
        };

        // Initialize metrics from actual DB state
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
            log::info!("📊 Metrics initialized: {} users, {} conversations", users_count, conversations_count);
        }

        Self {
            user_service,
            cerebras_client,
            redis_coordinator, // Reuse existing connection (no duplicate)
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
            pool,
        }
    }
}
