// MR.DarkPromth Agent 4 Main Application
// Jailbreak & Ultra Tier Engineer - Full Integration

use actix_web::{web, App, HttpServer, HttpResponse, Responder, middleware};
use actix_cors::Cors;
use actix_web::http::header;
use anyhow::Result;
use cerebras_client::CerebrasClient;
use log::{info, error, warn};
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::Utc;

use crate::{
    JailbreakPromptService, UserService, TierManagementService,
    ultra_tier_logic::{UltraTierLogic, UltraTierRequest, UltraTierResponse, User},
    sandboxed_execution::{SandboxedExecutor, SandboxConfig},
    jailbreak_models::{CreatePromptRequest, PromptSearchRequest, JailbreakPrompt, PromptCategory},
    user_service::{CreateUserRequest, RegisterRequest, UserResponse},
    audit::{AuditLogger, AuditEvent, AuditEventType},
};
use mr_darkpromth_core::UserTier as CoreUserTier;
use mr_darkpromth_db::{UserRepository, UserTier as DbUserTier};

// API Request/Response Types
#[derive(Debug, Deserialize)]
struct JailbreakRequest {
    user_id: String,
    prompt: String,
    ai_model: String,
    #[serde(default)]
    metadata: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct JailbreakResponse {
    request_id: String,
    user_id: String,
    user_tier: String,
    jailbreak_applied: bool,
    jailbreak_prompt_used: Option<String>,
    ai_response: String,
    filtered_response: Option<String>,
    safety_violations: Vec<String>,
    warnings: Vec<String>,
    timestamp: String,
    processing_time_ms: u64,
}

#[derive(Debug, Deserialize)]
struct SandboxExecutionRequest {
    code: String,
    language: String,
    #[serde(default)]
    timeout_seconds: u64,
}

#[derive(Debug, Serialize)]
struct SandboxExecutionResponse {
    exit_code: i32,
    stdout: String,
    stderr: String,
    execution_time_ms: u64,
    security_violations: Vec<String>,
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: String,
    version: String,
    uptime_seconds: u64,
    services: serde_json::Value,
}

// Application State
struct AppState {
    jailbreak_service: Arc<JailbreakPromptService>,
    user_service: Arc<UserService>,
    tier_management: Arc<TierManagementService>,
    ultra_tier_logic: Arc<RwLock<UltraTierLogic>>,
    sandbox_executor: Arc<SandboxedExecutor>,
    audit_logger: Arc<RwLock<AuditLogger>>,
    start_time: chrono::DateTime<Utc>,
}

fn map_user_tier(tier: &DbUserTier) -> CoreUserTier {
    match tier {
        DbUserTier::Free => CoreUserTier::Free,
        DbUserTier::Premium => CoreUserTier::Premium,
        DbUserTier::Ultra => CoreUserTier::Ultra,
    }
}

// Health Check Endpoint
async fn health_check(state: web::Data<AppState>) -> impl Responder {
    let uptime = (Utc::now() - state.start_time).num_seconds() as u64;
    
    let response = HealthResponse {
        status: "healthy".to_string(),
        version: "1.0.0".to_string(),
        uptime_seconds: uptime,
        services: serde_json::json!({
            "jailbreak_service": "operational",
            "user_service": "operational",
            "tier_management": "operational",
            "ultra_tier_logic": "operational",
            "sandbox_executor": "operational",
            "audit_logger": "operational"
        }),
    };
    
    HttpResponse::Ok().json(response)
}

// Jailbreak Prompt Endpoints
async fn create_jailbreak_prompt(
    request: web::Json<CreatePromptRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    info!("Creating jailbreak prompt: {}", request.title);
    
    match state.jailbreak_service.create_prompt(request.into_inner(), "system".to_string()).await {
        Ok(prompt) => {
            // Log audit event
            let mut audit = state.audit_logger.write().await;
            audit.log(AuditEvent {
                id: Uuid::new_v4().to_string(),
                timestamp: Utc::now(),
                event_type: AuditEventType::PromptCreated,
                user_id: "system".to_string(),
                details: format!("Created jailbreak prompt: {}", prompt.title),
                metadata: serde_json::json!({"prompt_id": prompt.id}),
            });
            
            HttpResponse::Created().json(prompt)
        }
        Err(e) => {
            error!("Failed to create prompt: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to create prompt",
                "details": e.to_string()
            }))
        }
    }
}

async fn get_jailbreak_prompts(
    query: web::Query<PromptSearchRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    info!("Searching jailbreak prompts");
    
    match state.jailbreak_service.search_prompts(query.into_inner()).await {
        Ok(prompts) => HttpResponse::Ok().json(prompts),
        Err(e) => {
            error!("Failed to search prompts: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to search prompts",
                "details": e.to_string()
            }))
        }
    }
}

async fn get_jailbreak_prompt_by_id(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let prompt_id = path.into_inner();
    
    match Uuid::parse_str(&prompt_id) {
        Ok(uuid) => {
            match state.jailbreak_service.get_prompt_by_id(uuid).await {
                Ok(Some(prompt)) => HttpResponse::Ok().json(prompt),
                Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
                    "error": "Prompt not found"
                })),
                Err(e) => {
                    error!("Failed to get prompt: {}", e);
                    HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": "Failed to get prompt",
                        "details": e.to_string()
                    }))
                }
            }
        }
        Err(_) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid UUID format"
        }))
    }
}

// Tier-based Jailbreak Execution Endpoint
async fn execute_jailbreak(
    request: web::Json<JailbreakRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let request_id = Uuid::new_v4().to_string();
    let start_time = std::time::Instant::now();
    
    info!("Executing jailbreak request for user: {}", request.user_id);
    
    // Detect user tier
    let user_tier_db = match state.tier_management.get_user_tier(&request.user_id).await {
        Ok(tier) => tier,
        Err(e) => {
            warn!("Failed to get user tier, defaulting to Free: {}", e);
            DbUserTier::Free
        }
    };

    let user_tier = map_user_tier(&user_tier_db);
    
    info!("User {} has tier: {:?}", request.user_id, user_tier);
    
    // Build UltraTierRequest
    let mut metadata = std::collections::HashMap::new();
    if let Ok(meta) = serde_json::from_value::<std::collections::HashMap<String, serde_json::Value>>(request.metadata.clone()) {
        metadata = meta;
    }
    
    let ultra_request = UltraTierRequest {
        request_id: request_id.clone(),
        user_id: request.user_id.clone(),
        user_tier: user_tier.clone(),
        original_prompt: request.prompt.clone(),
        selected_jailbreak_prompt: None,
        ai_model: request.ai_model.clone(),
        timestamp: Utc::now(),
        metadata,
    };
    
    // Process request through UltraTierLogic
    let mut logic = state.ultra_tier_logic.write().await;
    let response = match logic.process_request(ultra_request).await {
        Ok(resp) => resp,
        Err(e) => {
            error!("Failed to process request: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to process request",
                "details": e.to_string()
            }));
        }
    };
    
    let processing_time = start_time.elapsed().as_millis() as u64;
    
    // Log audit event
    let mut audit = state.audit_logger.write().await;
    audit.log(AuditEvent {
        id: Uuid::new_v4().to_string(),
        timestamp: Utc::now(),
        event_type: AuditEventType::JailbreakExecuted,
        user_id: request.user_id.clone(),
        details: format!("Jailbreak executed for user {} with tier {:?}", request.user_id, user_tier),
        metadata: serde_json::json!({
            "request_id": request_id,
            "processing_time_ms": processing_time,
            "jailbreak_applied": response.jailbreak_applied,
            "safety_violations": response.safety_violations.len()
        }),
    });
    
    // Convert to API response
    let api_response = JailbreakResponse {
        request_id: response.request_id,
        user_id: response.user_id,
        user_tier: format!("{:?}", response.user_tier),
        jailbreak_applied: response.jailbreak_applied,
        jailbreak_prompt_used: response.jailbreak_prompt_used.map(|id| id.to_string()),
        ai_response: response.ai_response,
        filtered_response: response.filtered_response,
        safety_violations: response.safety_violations,
        warnings: response.warnings,
        timestamp: response.timestamp.to_rfc3339(),
        processing_time_ms: processing_time,
    };
    
    HttpResponse::Ok().json(api_response)
}

// Sandboxed Execution Endpoint
async fn execute_sandboxed_code(
    request: web::Json<SandboxExecutionRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    info!("Executing sandboxed code in language: {}", request.language);
    
    let config = SandboxConfig {
        max_execution_time: std::time::Duration::from_secs(request.timeout_seconds.max(30)),
        max_memory: 512 * 1024 * 1024, // 512MB
        ..Default::default()
    };
    
    match state.sandbox_executor.execute_code(&request.code, &request.language).await {
        Ok(result) => {
            // Log audit event
            let mut audit = state.audit_logger.write().await;
            audit.log(AuditEvent {
                id: Uuid::new_v4().to_string(),
                timestamp: Utc::now(),
                event_type: AuditEventType::CodeExecuted,
                user_id: "system".to_string(),
                details: format!("Executed {} code in sandbox", request.language),
                metadata: serde_json::json!({
                    "language": request.language,
                    "exit_code": result.exit_code,
                    "execution_time_ms": result.execution_time.as_millis()
                }),
            });
            
            let response = SandboxExecutionResponse {
                exit_code: result.exit_code,
                stdout: result.stdout,
                stderr: result.stderr,
                execution_time_ms: result.execution_time.as_millis() as u64,
                security_violations: result.security_violations,
            };
            
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            error!("Sandbox execution failed: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Sandbox execution failed",
                "details": e.to_string()
            }))
        }
    }
}

// User Management Endpoints
async fn create_user(
    request: web::Json<CreateUserRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    info!("Creating user: {}", request.username);
    
    let request = request.into_inner();
    let register_request = RegisterRequest {
        username: request.username,
        email: request.email,
        password: request.password,
    };

    match state.user_service.register(register_request).await {
        Ok(response) => HttpResponse::Created().json(response),
        Err(e) => {
            error!("Failed to create user: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to create user",
                "details": e.to_string()
            }))
        }
    }
}

async fn get_user(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let user_id = path.into_inner();
    
    match Uuid::parse_str(&user_id) {
        Ok(uuid) => {
            match state.user_service.get_user_by_id(uuid).await {
                Ok(Some(user)) => HttpResponse::Ok().json(user),
                Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
                    "error": "User not found"
                })),
                Err(e) => {
                    error!("Failed to get user: {}", e);
                    HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": "Failed to get user",
                        "details": e.to_string()
                    }))
                }
            }
        }
        Err(_) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid UUID format"
        }))
    }
}

// Audit Logging Endpoints
async fn get_audit_logs(
    query: web::Query<std::collections::HashMap<String, String>>,
    state: web::Data<AppState>,
) -> impl Responder {
    let user_id = query.get("user_id").cloned();
    let limit = query.get("limit").and_then(|l| l.parse::<usize>().ok()).unwrap_or(100);
    
    let audit = state.audit_logger.read().await;
    let entries = audit.get_entries();
    
    let filtered: Vec<_> = if let Some(uid) = user_id {
        entries.iter().filter(|e| e.user_id == uid).take(limit).cloned().collect()
    } else {
        entries.iter().take(limit).cloned().collect()
    };
    
    HttpResponse::Ok().json(filtered)
}

async fn get_metrics(state: web::Data<AppState>) -> impl Responder {
    let logic = state.ultra_tier_logic.read().await;
    let metrics = logic.get_performance_metrics();
    
    HttpResponse::Ok().json(serde_json::json!({
        "performance_metrics": metrics,
        "ultra_tier_stats": logic.get_ultra_tier_usage_stats()
    }))
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();
    
    info!("🚀 Starting Agent 4: Jailbreak & Ultra Tier Engineer - Full Integration");
    
    // Get configuration from environment
    let bind_address = env::var("BIND_ADDRESS")
        .unwrap_or_else(|_| "0.0.0.0:8084".to_string());
    
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:5432/mr_darkpromth".to_string());
    let jwt_secret = env::var("JWT_SECRET")
        .unwrap_or_else(|_| "your-secret-key-change-in-production".to_string());
    
    let redis_url = env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());
    
    info!("Configuration:");
    info!("  Bind Address: {}", bind_address);
    info!("  Database URL: {}", database_url);
    info!("  Redis URL: {}", redis_url);
    
    // Initialize database pool
    let db_pool = sqlx::PgPool::connect(&database_url).await?;
    info!("✅ Database connected successfully");
    
    // Initialize services
    let jailbreak_service = Arc::new(JailbreakPromptService::new(db_pool.clone()));
    let user_repository = UserRepository::new(db_pool.clone());
    let user_service = Arc::new(UserService::new(user_repository, jwt_secret));
    let tier_repository = UserRepository::new(db_pool.clone());
    let tier_management = Arc::new(TierManagementService::new(tier_repository));
    
    // Initialize Cerebras client
    let cerebras_client = Arc::new(crate::cerebras_integration::CerebrasClient::new());
    
    // Initialize Ultra Tier Logic with audit logging
    let ultra_tier_logic = Arc::new(RwLock::new(UltraTierLogic::with_config(
        "d:/MR.Darkpromth/memory/agent4_audit.log",
        cerebras_client.clone()
    )));
    
    // Initialize sandboxed executor
    let sandbox_executor = Arc::new(SandboxedExecutor::new(SandboxConfig::default())?);
    
    // Initialize audit logger
    let audit_logger = Arc::new(RwLock::new(AuditLogger::new_file("d:/MR.Darkpromth/memory/agent4_audit.log")));
    
    info!("✅ All services initialized successfully");
    
    // Create application state
    let app_state = web::Data::new(AppState {
        jailbreak_service: jailbreak_service.clone(),
        user_service: user_service.clone(),
        tier_management: tier_management.clone(),
        ultra_tier_logic: ultra_tier_logic.clone(),
        sandbox_executor: sandbox_executor.clone(),
        audit_logger: audit_logger.clone(),
        start_time: Utc::now(),
    });
    
    // Start HTTP server
    info!("🌐 Starting HTTP server on {}", bind_address);
    
    HttpServer::new(move || {
        let cors = Cors::permissive();
        
        App::new()
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .app_data(app_state.clone())
            // Health check
            .route("/health", web::get().to(health_check))
            // Jailbreak prompt management
            .route("/api/jailbreak/prompts", web::post().to(create_jailbreak_prompt))
            .route("/api/jailbreak/prompts", web::get().to(get_jailbreak_prompts))
            .route("/api/jailbreak/prompts/{id}", web::get().to(get_jailbreak_prompt_by_id))
            // Tier-based jailbreak execution
            .route("/api/jailbreak/execute", web::post().to(execute_jailbreak))
            // Sandboxed execution
            .route("/api/sandbox/execute", web::post().to(execute_sandboxed_code))
            // User management
            .route("/api/users", web::post().to(create_user))
            .route("/api/users/{id}", web::get().to(get_user))
            // Audit logging
            .route("/api/audit/logs", web::get().to(get_audit_logs))
            // Metrics
            .route("/api/metrics", web::get().to(get_metrics))
    })
    .bind(&bind_address)?
    .run()
    .await?;
    
    info!("👋 Agent 4 shutting down");
    Ok(())
}
