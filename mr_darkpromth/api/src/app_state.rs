use std::sync::{Arc, Mutex, RwLock, atomic::AtomicU64};
use mr_darkpromth_services::{
    BillingService, CerebrasClient, EmailService, JailbreakPromptService, MasterToolExecutor,
    RedisCoordinator, SandboxedExecutor, ToolRegistry, UltraTierLogic, UserService,
};
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

impl AppState {
    pub async fn new(pool: sqlx::PgPool) -> Self {
        let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_default();
        let user_repo = UserRepository::new(pool.clone());
        let user_service = Arc::new(UserService::new(user_repo, jwt_secret));
        let cerebras_client = CerebrasClient::new();
        let jailbreak_service = Arc::new(JailbreakPromptService::new(pool.clone()));
        let billing_service = Arc::new(BillingService::new(pool.clone()));
        let email_service = Arc::new(EmailService::new_from_env(pool.clone()).expect("EmailService failed"));
        
        Self {
            user_service,
            cerebras_client,
            redis_coordinator: None,
            tool_registry: Arc::new(ToolRegistry::new(Default::default())),
            tool_executor: Arc::new(MasterToolExecutor::new(Default::default())),
            ultra_tier_logic: Arc::new(RwLock::new(UltraTierLogic::with_config("./memory/ultra_tier_audit.log", Arc::new(mr_darkpromth_services::KeyPool::new()), jailbreak_service.clone()))),
            sandbox_executor: Arc::new(SandboxedExecutor::new(Default::default()).expect("Sandbox failed")),
            jailbreak_service,
            billing_service,
            email_service,
            metrics: Arc::new(Metrics::default()),
        }
    }
}
