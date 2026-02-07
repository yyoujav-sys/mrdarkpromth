use std::sync::{Arc, Mutex, RwLock, atomic::AtomicU64};
use mr_darkpromth_services::{
    BillingService, CerebrasClient, EmailService, JailbreakPromptService, MasterToolExecutor,
    RedisCoordinator, SandboxedExecutor, ToolRegistry, UltraTierLogic, UserService,
};
use mr_darkpromth_services::audit::AuditLogger;
use mr_darkpromth_services::database_audit::DatabaseAuditService;
use mr_darkpromth_db::UserRepository;

#[derive(Clone)]
pub struct AppState {
    pub user_service: Arc<UserService>,
    pub cerebras_client: CerebrasClient,
    pub redis_coordinator: Option<Arc<Mutex<RedisCoordinator>>>,
    pub tool_registry: Arc<ToolRegistry>,
    pub tool_executor: Arc<MasterToolExecutor>,
    pub ultra_tier_logic: Arc<RwLock<UltraTierLogic>>,
    pub sandbox_executor: Arc<SandboxedExecutor>,
    pub jailbreak_service: Arc<JailbreakPromptService>,
    pub billing_service: Arc<BillingService>,
    pub email_service: Arc<EmailService>,
    pub audit_logger: Arc<AuditLogger>,
    pub metrics: Arc<Metrics>,
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
    
    // Check Cerebras API Keys
    let cerebras_keys = std::env::var("CEREBRAS_API_KEYS").unwrap_or_default();
    let cerebras_key = std::env::var("CEREBRAS_API_KEY").unwrap_or_default();
    if cerebras_keys.is_empty() && cerebras_key.is_empty() {
        missing.push("CEREBRAS_API_KEYS or CEREBRAS_API_KEY");
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
        let billing_service = Arc::new(BillingService::new(pool.clone()));
        let email_service = Arc::new(EmailService::new_from_env(pool.clone()).expect("EmailService failed"));
        let audit_logger = Arc::new(AuditLogger::new(Box::new(DatabaseAuditService::new(pool.clone()))));
        
        Self {
            user_service,
            cerebras_client,
            redis_coordinator: {
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
            },
            tool_registry: Arc::new(ToolRegistry::new(Default::default())),
            tool_executor: Arc::new(MasterToolExecutor::new(Default::default())),
            ultra_tier_logic: Arc::new(RwLock::new(UltraTierLogic::with_config("./memory/ultra_tier_audit.log", Arc::new(mr_darkpromth_services::KeyPool::new()), jailbreak_service.clone()))),
            sandbox_executor: Arc::new(SandboxedExecutor::new(Default::default()).expect("Sandbox failed")),
            jailbreak_service,
            billing_service,
            email_service,
            audit_logger,
            metrics: Arc::new(Metrics::default()),
        }
    }
}
