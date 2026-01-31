use actix_web::{web, App, HttpServer};
use actix_cors::Cors;
use crate::routes::*;
use crate::websocket::*;
use crate::jailbreak_api::*;
use crate::auth_routes::*;
use crate::user_routes::*;
use crate::tool_routes::{build_tool_executor, execute_sandbox, execute_tool, list_tools};
use crate::terminal_routes::execute_terminal;
use crate::auth_middleware::{AuthMiddleware, OptionalAuthMiddleware};
use crate::openapi::configure_swagger_ui;
use std::env;
use std::sync::{Arc, Mutex};
use mr_darkpromth_services::{ToolRegistry, ToolRegistryConfig, UserService};
use mr_darkpromth_services::cerebras_integration::CerebrasClient;
use mr_darkpromth_services::RedisCoordinator;
use mr_darkpromth_services::ultra_tier_logic::UltraTierLogic;
use mr_darkpromth_services::sandboxed_execution::{SandboxedExecutor, SandboxConfig};
use sqlx::PgPool;
use mr_darkpromth_db::UserRepository;

pub struct Server {
    pool: PgPool,
    jwt_secret: String,
}

impl Server {
    pub fn new(pool: PgPool, jwt_secret: String) -> Self {
        Self { pool, jwt_secret }
    }

    pub async fn start(&self, config: ServerConfig) -> std::io::Result<()> {
        let bind_address = format!("{}:{}", config.host, config.port);
        let jwt_secret = self.jwt_secret.clone();

        // Create user service
        let user_repository = UserRepository::new(self.pool.clone());
        let user_service = Arc::new(UserService::new(user_repository, jwt_secret.clone()));
        let cerebras_client = web::Data::new(CerebrasClient::new());
        
        // Initialize KeyPool and populate with ENV keys
        let key_pool = Arc::new(mr_darkpromth_services::key_pool::KeyPool::new());
        
        // Support single key (legacy)
        if let Ok(key) = env::var("CEREBRAS_API_KEY") {
             let _ = key_pool.add_key(
                 mr_darkpromth_services::key_pool::Provider::Cerebras, 
                 key, 
                 "Default Env Key".to_string()
             ).await;
        }

        // Support multiple keys (production)
        if let Ok(keys_str) = env::var("CEREBRAS_API_KEYS") {
            let keys: Vec<&str> = keys_str.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
            for (idx, key) in keys.iter().enumerate() {
                let _ = key_pool.add_key(
                    mr_darkpromth_services::key_pool::Provider::Cerebras, 
                    key.to_string(), 
                    format!("Env Key #{}", idx + 1)
                ).await;
            }
            log::info!("Loaded {} keys from CEREBRAS_API_KEYS", keys.len());
        }

        // Initialize JailbreakPromptService
        let jailbreak_service = Arc::new(mr_darkpromth_services::JailbreakPromptService::new(self.pool.clone()));

        let coordination_enabled = env::var("AGENT_COORDINATION_ENABLED")
            .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "on"))
            .unwrap_or(true);
        let redis_coordinator = if coordination_enabled {
            match env::var("REDIS_URL") {
                Ok(url) => match RedisCoordinator::new(&url, "api_gateway".to_string()) {
                    Ok(coordinator) => Some(Arc::new(Mutex::new(coordinator))),
                    Err(err) => {
                        log::warn!("Failed to initialize Redis coordinator at {}: {}", url, err);
                        None
                    }
                },
                Err(_) => {
                    log::info!("REDIS_URL not set; Redis coordination disabled");
                    None
                }
            }
        } else {
            log::info!("AGENT_COORDINATION_ENABLED disabled; Redis coordination disabled");
            None
        };
        let redis_coordinator = web::Data::new(redis_coordinator);

        let mut registry = ToolRegistry::new(ToolRegistryConfig::default());
        
        // Inject DB pool for database tools
        registry.set_db_pool(self.pool.clone());
        
        if let Err(err) = registry.initialize().await {
            let io_error = std::io::Error::new(std::io::ErrorKind::Other, err.to_string());
            return Err(io_error);
        }

        let tool_registry = Arc::new(registry);
        let tool_executor = build_tool_executor(tool_registry.clone());

        // Initialize Ultra Tier Logic with real services
        let ultra_tier_logic = Arc::new(std::sync::RwLock::new(UltraTierLogic::with_config(
            "./memory/ultra_tier_audit.log",
            key_pool.clone(),
            jailbreak_service.clone(),
        )));

        // Initialize Sandbox Executor
        let sandbox_executor = Arc::new(SandboxedExecutor::new(SandboxConfig::default())
            .expect("Failed to create sandbox executor"));

        log::info!("🚀 Starting API Gateway on {}", bind_address);
        log::info!("📊 Health check endpoint: http://{}/health", bind_address);
        log::info!("💬 WebSocket endpoint: ws://{}/ws/chat", bind_address);
        log::info!("🔓 Jailbreak API: http://{}/api/jailbreak/*", bind_address);
        log::info!("👤 Auth API: http://{}/api/auth/*", bind_address);
        log::info!("📋 User API: http://{}/api/users/*", bind_address);
        log::info!("🧠 Cerebras.ai chat integration enabled");
        log::info!("🔑 Key System: Initialized with {} key(s)", key_pool.get_status().await.len());

        HttpServer::new(move || {
            let cors = Cors::default()
                .allowed_origin_fn(|origin, _req_head| {
                    let origin_str = origin.to_str().unwrap_or("");
                    origin_str == "http://localhost:5173"
                        || origin_str == "http://localhost:3000"
                        || origin_str == "http://127.0.0.1:5173"
                        || origin_str == "http://127.0.0.1:3000"
                })
                .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
                .allowed_headers(vec![
                    "Authorization",
                    "Content-Type",
                    "Accept",
                    "X-Requested-With",
                ])
                .expose_headers(vec!["Content-Length"])
                .max_age(3600);

            let user_service = user_service.clone();
            let jwt_secret = jwt_secret.clone();
            let cerebras_client = cerebras_client.clone();
            let redis_coordinator = redis_coordinator.clone();
            let tool_registry = tool_registry.clone();
            let tool_executor = tool_executor.clone();
            let ultra_tier_logic = ultra_tier_logic.clone();
            let sandbox_executor = sandbox_executor.clone();

            let _auth_middleware = AuthMiddleware::new(user_service.clone(), jwt_secret.clone());
            let optional_auth_middleware = OptionalAuthMiddleware::new(user_service.clone(), jwt_secret.clone());

            App::new()
                .wrap(cors)
                .wrap(optional_auth_middleware)
                .app_data(web::Data::new(user_service.clone()))
                .app_data(cerebras_client.clone())
                .app_data(redis_coordinator.clone())
                .app_data(web::Data::new(tool_registry.clone()))
                .app_data(web::Data::new(tool_executor.clone()))
                .app_data(web::Data::new(ultra_tier_logic.clone()))
                .app_data(web::Data::new(sandbox_executor.clone()))
                .configure(configure_swagger_ui)
                .configure(crate::routes::configure_github_routes)
                // Health check
                .route("/health", web::get().to(get_health))
                // User info (public, for backward compatibility)
                .route("/api/users/{id}", web::get().to(get_user_info))
                // Auth endpoints
                .route("/api/auth/register", web::post().to(register))
                .route("/api/auth/login", web::post().to(login))
                .route("/api/auth/logout", web::post().to(logout))
                .route("/api/auth/me", web::get().to(get_me))
                // User profile endpoints (authenticated)
                .route("/api/users/me", web::get().to(get_me))
                .route("/api/users/me", web::put().to(update_profile))
                .route("/api/users/me/password", web::put().to(change_password))
                .route("/api/users/me/api-key", web::post().to(regenerate_api_key))
                // Admin user management (authenticated)
                .route("/api/admin/users", web::get().to(list_users))
                .route("/api/admin/users/{id}", web::delete().to(delete_user))
                .route("/api/admin/users/{id}/status", web::put().to(update_user_status))
                .route("/api/admin/metrics", web::get().to(get_admin_metrics))
                // Chat endpoint (authenticated)
                .route("/api/chat", web::post().to(post_chat))
                // Tooling and sandbox endpoints (authenticated)
                .route("/api/tools", web::get().to(list_tools))
                .route("/api/tools/execute", web::post().to(execute_tool))
                .route("/api/sandbox/execute", web::post().to(execute_sandbox))
                // Terminal execution endpoint (Ultra tier only)
                .route("/api/terminal/execute", web::post().to(execute_terminal))
                // WebSocket (authenticated)
                .route("/ws/chat", web::get().to(ws_chat_route))
                // Jailbreak API (authenticated)
                .route("/api/jailbreak/prompts", web::get().to(get_prompts))
                .route("/api/jailbreak/prompts", web::post().to(create_prompt))
                .route("/api/jailbreak/prompts/{id}", web::get().to(get_prompt_by_id))
                .route("/api/jailbreak/prompts/{id}", web::put().to(update_prompt))
                .route("/api/jailbreak/prompts/{id}", web::delete().to(delete_prompt))
                .route("/api/jailbreak/prompts/search", web::post().to(search_prompts))
                .route("/api/jailbreak/prompts/category/{category}", web::get().to(get_prompts_by_category))
                .route("/api/jailbreak/prompts/popular", web::get().to(get_popular_prompts))
                .route("/api/jailbreak/prompts/{id}/analytics", web::get().to(get_prompt_analytics))
                .route("/api/jailbreak/prompts/{id}/usage", web::post().to(record_prompt_usage))
                .default_service(web::route().to(not_found))
        })
        .bind(&bind_address)?
        .run()
        .await
    }
}
