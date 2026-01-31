// MR.DarkPromth Unified Main Application
// Integrates Agent 4 (Jailbreak/API), Agent 7 (Multi-Agent), and Agent 8 (Self-Correction)

use actix_web::{web, App, HttpServer, HttpResponse, Responder, middleware};
use actix_cors::Cors;
use anyhow::Result;
use log::{info, error};
use std::env;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::Utc;

use mr_darkpromth_services::{
    JailbreakPromptService, UserService, TierManagementService,
    ultra_tier_logic::{UltraTierLogic, UltraTierRequest},
    sandboxed_execution::{SandboxedExecutor, SandboxConfig},
    jailbreak_models::{CreatePromptRequest, PromptSearchRequest},
    user_service::{CreateUserRequest, RegisterRequest},
    audit::{AuditLogger, AuditEvent, AuditEventType},
    agent7_main::MultiAgentSystem,
    agent8_main::Agent8SelfCorrection,
    key_pool::{KeyPool, Provider},
};
use mr_darkpromth_core::UserTier as CoreUserTier;
use mr_darkpromth_db::UserTier as DbUserTier;

// API Request/Response Types
#[derive(serde::Deserialize)]
struct JailbreakRequest {
    user_id: String,
    prompt: String,
    ai_model: String,
    #[serde(default)]
    _tier: Option<String>,
    #[serde(default)]
    _metadata: serde_json::Value,
}

#[derive(serde::Serialize)]
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

#[derive(serde::Deserialize)]
struct SandboxExecutionRequest {
    code: String,
    language: String,
    #[serde(default)]
    _timeout_seconds: u64,
}

#[derive(serde::Serialize)]
struct SandboxExecutionResponse {
    exit_code: i32,
    stdout: String,
    stderr: String,
    execution_time_ms: u64,
    security_violations: Vec<String>,
}

#[derive(serde::Serialize)]
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
    multi_agent_system_status: Arc<RwLock<String>>,
    self_correction_status: Arc<RwLock<String>>,
    key_pool: Arc<KeyPool>,
}

fn map_user_tier(tier: &DbUserTier) -> CoreUserTier {
    match tier {
        DbUserTier::Free => CoreUserTier::Free,
        DbUserTier::Premium => CoreUserTier::Premium,
        DbUserTier::Ultra => CoreUserTier::Ultra,
    }
}

async fn health_check(state: web::Data<AppState>) -> impl Responder {
    let uptime = (Utc::now() - state.start_time).num_seconds() as u64;
    let ma_status = state.multi_agent_system_status.read().await;
    let sc_status = state.self_correction_status.read().await;
    
    let response = HealthResponse {
        status: "healthy".to_string(),
        version: "1.1.0-unified".to_string(),
        uptime_seconds: uptime,
        services: serde_json::json!({
            "jailbreak_service": "operational",
            "user_service": "operational",
            "tier_management": "operational",
            "ultra_tier_logic": "operational",
            "sandbox_executor": "operational",
            "multi_agent_system": *ma_status,
            "self_correction_engine": *sc_status,
            "audit_logger": "operational",
            "key_pool": "operational"
        }),
    };
    
    HttpResponse::Ok().json(response)
}

async fn create_jailbreak_prompt(request: web::Json<CreatePromptRequest>, state: web::Data<AppState>) -> impl Responder {
    match state.jailbreak_service.create_prompt(request.into_inner(), "system".to_string()).await {
        Ok(prompt) => {
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
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
    }
}

async fn get_jailbreak_prompts(query: web::Query<PromptSearchRequest>, state: web::Data<AppState>) -> impl Responder {
    match state.jailbreak_service.search_prompts(query.into_inner()).await {
        Ok(prompts) => HttpResponse::Ok().json(prompts),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
    }
}

async fn execute_jailbreak(request: web::Json<JailbreakRequest>, state: web::Data<AppState>) -> impl Responder {
    let request_id = Uuid::new_v4().to_string();
    let start_time = std::time::Instant::now();
    
    let user_tier_db = match state.tier_management.get_user_tier(&request.user_id).await {
        Ok(tier) => tier,
        Err(_) => DbUserTier::Free,
    };
    let user_tier = map_user_tier(&user_tier_db);
    
    let ultra_request = UltraTierRequest {
        request_id: request_id.clone(),
        user_id: request.user_id.clone(),
        user_tier: user_tier.clone(),
        original_prompt: request.prompt.clone(),
        selected_jailbreak_prompt: None,
        ai_model: request.ai_model.clone(),
        timestamp: Utc::now(),
        metadata: std::collections::HashMap::new(),
    };
    
    let logic = state.ultra_tier_logic.write().await;
    match logic.process_request(ultra_request).await {
        Ok(response) => {
            let processing_time = start_time.elapsed().as_millis() as u64;
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
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
    }
}

async fn execute_sandboxed_code(request: web::Json<SandboxExecutionRequest>, state: web::Data<AppState>) -> impl Responder {
    match state.sandbox_executor.execute_code(&request.code, &request.language).await {
        Ok(result) => {
            let response = SandboxExecutionResponse {
                exit_code: result.exit_code,
                stdout: result.stdout,
                stderr: result.stderr,
                execution_time_ms: result.execution_time.as_millis() as u64,
                security_violations: result.security_violations,
            };
            HttpResponse::Ok().json(response)
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
    }
}

async fn create_user(request: web::Json<CreateUserRequest>, state: web::Data<AppState>) -> impl Responder {
    let register_request = RegisterRequest {
        username: request.username.clone(),
        email: request.email.clone(),
        password: request.password.clone(),
    };
    match state.user_service.register(register_request).await {
        Ok(response) => HttpResponse::Created().json(response),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
    }
}

async fn get_key_pool_status(state: web::Data<AppState>) -> impl Responder {
    let status = state.key_pool.get_status().await;
    HttpResponse::Ok().json(status)
}

// Terminal Execution Request/Response Types
#[derive(serde::Deserialize)]
struct TerminalExecuteRequest {
    command: String,
    #[serde(default)]
    args: Vec<String>,
    #[serde(default)]
    _working_dir: Option<String>,
    user_id: String,
}

#[derive(serde::Serialize)]
struct TerminalExecuteResponse {
    exit_code: i32,
    stdout: String,
    stderr: String,
    execution_time_ms: u64,
    command: String,
}

// Tool List Response Types
#[derive(serde::Serialize)]
struct ToolInfo {
    name: String,
    description: String,
    category: String,
    required_permissions: Vec<String>,
}

#[derive(serde::Serialize)]
struct ToolListResponse {
    tools: Vec<ToolInfo>,
    total_count: usize,
}

async fn execute_terminal(request: web::Json<TerminalExecuteRequest>, state: web::Data<AppState>) -> impl Responder {
    // Check if user has Ultra tier
    let user_tier_db = match state.tier_management.get_user_tier(&request.user_id).await {
        Ok(tier) => tier,
        Err(_) => DbUserTier::Free,
    };
    
    // Ultra Tier only
    if !matches!(user_tier_db, DbUserTier::Ultra) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Forbidden",
            "message": "Terminal execution requires Ultra tier"
        }));
    }

    let start_time = std::time::Instant::now();
    
    // Build the command
    let full_command = if request.args.is_empty() {
        request.command.clone()
    } else {
        format!("{} {}", request.command, request.args.join(" "))
    };

    // Execute through sandbox for security
    match state.sandbox_executor.execute_code(&full_command, "bash").await {
        Ok(result) => {
            let response = TerminalExecuteResponse {
                exit_code: result.exit_code,
                stdout: result.stdout,
                stderr: result.stderr,
                execution_time_ms: start_time.elapsed().as_millis() as u64,
                command: full_command,
            };
            
            // Audit log
            let mut audit = state.audit_logger.write().await;
            audit.log(AuditEvent {
                id: Uuid::new_v4().to_string(),
                timestamp: Utc::now(),
                event_type: AuditEventType::ToolExecuted,
                user_id: request.user_id.clone(),
                details: format!("Terminal execution: {}", request.command),
                metadata: serde_json::json!({
                    "exit_code": result.exit_code,
                    "command": request.command
                }),
            });
            
            HttpResponse::Ok().json(response)
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Execution failed",
            "message": e.to_string()
        }))
    }
}

async fn list_tools(_state: web::Data<AppState>) -> impl Responder {
    // Return list of available tools
    let tools = vec![
        ToolInfo {
            name: "file_read".to_string(),
            description: "Read file contents".to_string(),
            category: "file_operations".to_string(),
            required_permissions: vec!["file.read".to_string()],
        },
        ToolInfo {
            name: "file_write".to_string(),
            description: "Write content to file".to_string(),
            category: "file_operations".to_string(),
            required_permissions: vec!["file.write".to_string()],
        },
        ToolInfo {
            name: "file_list".to_string(),
            description: "List directory contents".to_string(),
            category: "file_operations".to_string(),
            required_permissions: vec!["file.read".to_string()],
        },
        ToolInfo {
            name: "web_scrape".to_string(),
            description: "Make HTTP requests with full support for all methods, headers, body, and authentication".to_string(),
            category: "network".to_string(),
            required_permissions: vec!["network.read".to_string()],
        },
        ToolInfo {
            name: "code_execute".to_string(),
            description: "Execute code in a sandboxed environment".to_string(),
            category: "execution".to_string(),
            required_permissions: vec!["code.execute".to_string()],
        },
        ToolInfo {
            name: "database_query".to_string(),
            description: "Execute SQL queries against the database".to_string(),
            category: "database".to_string(),
            required_permissions: vec!["database.read".to_string()],
        },
        ToolInfo {
            name: "terminal_execute".to_string(),
            description: "Execute terminal commands (Ultra tier only)".to_string(),
            category: "execution".to_string(),
            required_permissions: vec!["terminal.execute".to_string()],
        },
    ];
    
    let response = ToolListResponse {
        total_count: tools.len(),
        tools,
    };
    
    HttpResponse::Ok().json(response)
}

// Tool Execute Request/Response
#[derive(serde::Deserialize)]
struct ToolExecuteRequest {
    tool_name: String,
    input: serde_json::Value,
    user_id: String,
}

#[derive(serde::Serialize)]
struct ToolExecuteResponse {
    success: bool,
    data: Option<serde_json::Value>,
    error: Option<String>,
    execution_time_ms: u64,
    tool_name: String,
}

async fn execute_tool(request: web::Json<ToolExecuteRequest>, state: web::Data<AppState>) -> impl Responder {
    let start_time = std::time::Instant::now();
    let tool_name = request.tool_name.clone();
    let user_id = request.user_id.clone();
    
    // Check user tier for permission-gated tools
    let user_tier_db = match state.tier_management.get_user_tier(&user_id).await {
        Ok(tier) => tier,
        Err(_) => DbUserTier::Free,
    };
    
    // Terminal execute is Ultra-only
    if tool_name == "terminal_execute" && !matches!(user_tier_db, DbUserTier::Ultra) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Forbidden",
            "message": "Terminal execution requires Ultra tier"
        }));
    }
    
    // Use the ToolSystem to execute
    let tool_system = mr_darkpromth_services::tool_system::ToolSystem::new();
    
    match tool_system.execute_tool(&tool_name, request.input.clone()).await {
        Ok(result) => {
            let execution_time = start_time.elapsed().as_millis() as u64;
            
            // Audit log
            let mut audit = state.audit_logger.write().await;
            audit.log(AuditEvent {
                id: Uuid::new_v4().to_string(),
                timestamp: Utc::now(),
                event_type: AuditEventType::ToolExecuted,
                user_id: user_id.clone(),
                details: format!("Tool executed: {}", tool_name),
                metadata: serde_json::json!({
                    "tool_name": tool_name,
                    "success": true
                }),
            });
            
            HttpResponse::Ok().json(ToolExecuteResponse {
                success: true,
                data: Some(result),
                error: None,
                execution_time_ms: execution_time,
                tool_name,
            })
        }
        Err(e) => {
            let execution_time = start_time.elapsed().as_millis() as u64;
            
            HttpResponse::Ok().json(ToolExecuteResponse {
                success: false,
                data: None,
                error: Some(e.to_string()),
                execution_time_ms: execution_time,
                tool_name,
            })
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();
    
    info!("🚀 Initializing MR.DarkPromth Unified Backend System");
    
    let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let server_port = env::var("SERVER_PORT").unwrap_or_else(|_| "8080".to_string());
    let bind_address = format!("{}:{}", server_host, server_port);
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| "postgresql://postgres:5432/mr_darkpromth".to_string());
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
    let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "your-secret-key-change-in-production".to_string());
    
    // 1. Initialize Key Pool
    let key_pool = Arc::new(KeyPool::new());
    
    // Populate Key Pool with initial brain power from environment
    if let Ok(key) = env::var("CEREBRAS_API_KEY") {
        key_pool.add_key(Provider::Cerebras, key, "Cerebras Primary".to_string()).await;
        info!("🧠 Added Cerebras API key to pool");
    }
    if let Ok(key) = env::var("OPENROUTER_API_KEY") {
        key_pool.add_key(Provider::OpenRouter, key, "OpenRouter Primary".to_string()).await;
        info!("🧠 Added OpenRouter API key to pool");
    }

    // 2. Initialize Database
    let db_pool = sqlx::PgPool::connect(&database_url).await?;
    info!("✅ Database connected");
    
    // 3. Initialize Common Services
    let jailbreak_service = Arc::new(JailbreakPromptService::new(db_pool.clone()));
    let user_repository = mr_darkpromth_db::UserRepository::new(db_pool.clone());
    let user_service = Arc::new(UserService::new(user_repository, jwt_secret));
    let tier_repository = mr_darkpromth_db::UserRepository::new(db_pool.clone());
    let tier_management = Arc::new(TierManagementService::new(tier_repository));
    let ultra_tier_logic = Arc::new(RwLock::new(UltraTierLogic::with_config("./memory/unified_audit.log", key_pool.clone(), jailbreak_service.clone())));
    let sandbox_executor = Arc::new(SandboxedExecutor::new(SandboxConfig::default())?);
    let audit_logger = Arc::new(RwLock::new(AuditLogger::new_file("./memory/unified_audit.log")));
    
    // 3. Initialize background agents
    let ma_status = Arc::new(RwLock::new("initializing".to_string()));
    let sc_status = Arc::new(RwLock::new("initializing".to_string()));
    
    // Start Multi-Agent System (Agent 7)
    let ma_status_clone = ma_status.clone();
    let redis_url_clone = redis_url.clone();
    tokio::spawn(async move {
        info!("🤖 Starting Multi-Agent System task...");
        match MultiAgentSystem::new(&redis_url_clone, ".".to_string()).await.map_err(|e| e.to_string()) {
            Ok(mut system) => {
                if let Err(err_msg) = system.start().await.map_err(|e| e.to_string()) {
                    error!("❌ Multi-Agent System failed to start: {}", err_msg);
                    let mut status = ma_status_clone.write().await;
                    *status = format!("error: {}", err_msg);
                } else {
                    info!("✅ Multi-Agent System background task running");
                    let mut status = ma_status_clone.write().await;
                    *status = "operational".to_string();
                }
            }
            Err(err_msg) => {
                error!("❌ Multi-Agent System initialization failed: {}", err_msg);
                let mut status = ma_status_clone.write().await;
                *status = format!("error: {}", err_msg);
            }
        }
    });

    // Start Self-Correction Engine (Agent 8)
    let sc_status_clone = sc_status.clone();
    let redis_url_clone2 = redis_url.clone();
    tokio::spawn(async move {
        info!("🤖 Starting Self-Correction Engine task...");
        match Agent8SelfCorrection::new(Some(&redis_url_clone2)).map_err(|e| e.to_string()) {
            Ok(mut agent8) => {
                {
                    let mut status = sc_status_clone.write().await;
                    *status = "operational".to_string();
                }
                if let Err(err_msg) = agent8.start().await.map_err(|e| e.to_string()) {
                    error!("❌ Self-Correction Engine loop failed: {}", err_msg);
                    let mut status = sc_status_clone.write().await;
                    *status = format!("error: {}", err_msg);
                }
            }
            Err(err_msg) => {
                error!("❌ Self-Correction Engine initialization failed: {}", err_msg);
                let mut status = sc_status_clone.write().await;
                *status = format!("error: {}", err_msg);
            }
        }
    });

    let app_state = web::Data::new(AppState {
        jailbreak_service,
        user_service,
        tier_management,
        ultra_tier_logic,
        sandbox_executor,
        audit_logger,
        start_time: Utc::now(),
        multi_agent_system_status: ma_status,
        self_correction_status: sc_status,
        key_pool: key_pool.clone(),
    });
    
    info!("🌐 Starting Unified API Server on {}", bind_address);
    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .wrap(middleware::Logger::default())
            .app_data(app_state.clone())
            .route("/health", web::get().to(health_check))
            .route("/api/status/keys", web::get().to(get_key_pool_status))
            .route("/api/jailbreak/prompts", web::post().to(create_jailbreak_prompt))
            .route("/api/jailbreak/prompts", web::get().to(get_jailbreak_prompts))
            .route("/api/jailbreak/execute", web::post().to(execute_jailbreak))
            .route("/api/sandbox/execute", web::post().to(execute_sandboxed_code))
            .route("/api/terminal/execute", web::post().to(execute_terminal))
            .route("/api/tools", web::get().to(list_tools))
            .route("/api/tools/execute", web::post().to(execute_tool))
            .route("/api/users", web::post().to(create_user))
    })
    .bind(&bind_address)?
    .run()
    .await?;
    
    Ok(())
}
