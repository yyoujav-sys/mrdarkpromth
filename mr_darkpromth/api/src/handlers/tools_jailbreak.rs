use axum::{
    extract::{State, Json, Path},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::AppState;
use crate::handlers::auth::extract_token;
use crate::error_handler::{ApiError, ApiSuccess};
use mr_darkpromth_services::jailbreak_models::{
    PromptSortBy, SortOrder,
    PromptSearchRequest, PromptCategory,
    BypassTechnique, EffectivenessRating, RiskLevel,
};


#[derive(Debug, Serialize)]
pub struct ToolInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub required_permissions: Vec<String>,
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct ExecuteToolRequest {
    pub tool_id: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct SandboxExecuteRequest {
    pub code: String,
    pub language: String,
    pub timeout_seconds: Option<u64>,
    pub session_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SandboxExecuteResponse {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub execution_time_ms: u64,
    pub memory_used: u64,
    pub cpu_time_ms: u64,
    pub files_created: Vec<String>,
    pub security_violations: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct TerminalExecuteRequest {
    pub command: String,
    pub args: Option<Vec<String>>,
    pub working_dir: Option<String>,
    pub session_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TerminalExecuteResponse {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub execution_time_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JailbreakPromptDto {
    pub id: String,
    pub title: String,
    pub content: String,
    pub category: PromptCategory,
    pub technique: BypassTechnique,
    pub effectiveness: EffectivenessRating,
    pub risk_level: RiskLevel,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub author: String,
    pub usage_count: i64,
    pub success_rate: f64,
    pub requires_ultra_tier: bool,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreatePromptApiRequest {
    pub title: String,
    pub content: String,
    pub category: String,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub requires_ultra_tier: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct SearchPromptsRequest {
    pub query: String,
    pub category: Option<String>,
}

// ==================== Tools Handlers ====================

pub async fn list_tools_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let _token = match extract_token(&headers) {
        Some(t) => t,
        None => {
            return ApiError::new("MISSING_TOKEN", "Authorization header required").into_response();
        }
    };

    // Get tools from the registry
    let tools = state.tool_registry.list_tools();
    let tools_list: Vec<ToolInfo> = tools.iter()
        .filter_map(|tool_name| {
            state.tool_registry.get_tool_metadata(tool_name).map(|metadata| ToolInfo {
                id: metadata.name.clone(),
                name: metadata.name.clone(),
                description: metadata.schema.description.clone(),
                category: metadata.schema.category.clone(),
                required_permissions: metadata.schema.required_permissions.clone(),
                enabled: metadata.enabled,
            })
        })
        .collect();

    ApiSuccess::new(serde_json::json!({ "tools": tools_list })).into_response()
}

pub async fn execute_tool_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(req): Json<ExecuteToolRequest>,
) -> impl IntoResponse {
    let token = match extract_token(&headers) {
        Some(t) => t,
        None => {
            return ApiError::new("MISSING_TOKEN", "Authorization header required").into_response();
        }
    };

    let claims = match state.user_service.validate_token(&token).await {
        Ok(c) => c,
        Err(_) => {
            return ApiError::new("INVALID_TOKEN", "Invalid or expired token").into_response();
        }
    };

    // Check tier requirements
    if claims.tier != "premium" && claims.tier != "ultra" {
        return ApiError::new("PREMIUM_REQUIRED", "Tool execution requires Premium tier or higher").into_response();
    }

    // Execute tool based on tool_id
    match req.tool_id.as_str() {
        // file_reader removed due to security policy (LFI risk). Use Sandbox instead.
        "web_search" => {
            // Web search using reqwest to fetch search results
            let query = req.parameters.get("query").and_then(|q| q.as_str()).unwrap_or("");
            if query.is_empty() {
                return ApiError::new("MISSING_QUERY", "Search query is required").into_response();
            }

            let search_url = format!("https://html.duckduckgo.com/html/?q={}", urlencoding::encode(query));
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build();

            match client {
                Ok(client) => {
                    match client.get(&search_url)
                        .header("User-Agent", "MRDarkPromth/1.0")
                        .send().await
                    {
                        Ok(resp) => {
                            let status = resp.status().as_u16();
                            let body = resp.text().await.unwrap_or_default();
                            // Extract text snippets from HTML response
                            let snippets: Vec<&str> = body.split("result__snippet")
                                .skip(1)
                                .take(5)
                                .filter_map(|s| {
                                    s.find('>').and_then(|start| {
                                        s[start+1..].find('<').map(|end| &s[start+1..start+1+end])
                                    })
                                })
                                .collect();

                            ApiSuccess::new(serde_json::json!({
                                "result": snippets,
                                "query": query,
                                "status_code": status,
                                "result_count": snippets.len(),
                                "success": true
                            })).into_response()
                        }
                        Err(e) => ApiError::new("SEARCH_FAILED", &format!("Search request failed: {}", e)).into_response()
                    }
                }
                Err(e) => ApiError::new("CLIENT_ERROR", &format!("Failed to create HTTP client: {}", e)).into_response()
            }
        }
        _ => ApiError::new("TOOL_NOT_FOUND", "Tool not found").into_response(),
    }
}

pub async fn execute_sandbox_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(req): Json<SandboxExecuteRequest>,
) -> impl IntoResponse {
    let token = match extract_token(&headers) {
        Some(t) => t,
        None => {
            return ApiError::new("MISSING_TOKEN", "Authorization header required").into_response();
        }
    };

    // Validate token and check tier
    let claims = match state.user_service.validate_token(&token).await {
        Ok(c) => c,
        Err(_) => {
            return ApiError::new("INVALID_TOKEN", "Invalid or expired token").into_response();
        }
    };

    // Check tier requirements - sandbox requires Premium or Ultra
    let tier = claims.tier.to_lowercase();
    if tier != "premium" && tier != "ultra" && tier != "admin" {
        return ApiError::new("PREMIUM_REQUIRED", "Sandbox execution requires Premium tier or higher").into_response();
    }

    let executor = if let Some(ref session_id) = req.session_id {
        let user_id = uuid::Uuid::parse_str(&claims.sub).ok();
        match state.sandbox_manager.get_executor(session_id, user_id).await {
            Ok(exec) => exec,
            Err(_) => {
                return ApiError::new("SESSION_NOT_FOUND", &format!("Sandbox session {} not found", session_id)).into_response();
            }
        }
    } else {
        // One-off execution
        let mut config = mr_darkpromth_services::sandboxed_execution::SandboxConfig::default();
        if let Some(timeout) = req.timeout_seconds {
            config.max_execution_time = std::time::Duration::from_secs(timeout);
        }
        match mr_darkpromth_services::sandboxed_execution::SandboxedExecutor::new(config) {
            Ok(exec) => Arc::new(exec),
            Err(e) => {
                return ApiError::new("SANDBOX_ERROR", &format!("Failed to create sandbox: {}", e)).into_response();
            }
        }
    };

    // Execute code
    match executor.execute_code(&req.code, &req.language).await {
        Ok(result) => {
            let files_created: Vec<String> = result.files_created.iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect();
            
            ApiSuccess::new(serde_json::json!({
                "stdout": result.stdout,
                "stderr": result.stderr,
                "exit_code": result.exit_code,
                "execution_time_ms": result.execution_time.as_millis() as u64,
                "memory_used": result.memory_used,
                "cpu_time_ms": result.cpu_time.as_millis() as u64,
                "files_created": files_created,
                "security_violations": result.security_violations,
                "session_id": req.session_id,
            })).into_response()
        }
        Err(e) => ApiError::new("EXECUTION_FAILED", &format!("Sandbox execution failed: {}", e)).into_response(),
    }
}

pub async fn admin_create_sandbox_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    use uuid::Uuid;
    let token = match extract_token(&headers) {
        Some(t) => t,
        None => return ApiError::new("MISSING_TOKEN", "Auth required").into_response(),
    };

    let claims = match state.user_service.validate_token(&token).await {
        Ok(c) => c,
        Err(_) => return ApiError::new("INVALID_TOKEN", "Invalid token").into_response(),
    };

    if claims.tier != "admin" && claims.tier != "ultra" {
        return ApiError::new("FORBIDDEN", "Ultra or Admin required").into_response();
    }

    let session_id = req.get("session_id").and_then(|v| v.as_str()).unwrap_or(&Uuid::new_v4().to_string()).to_string();
    
    match state.sandbox_manager.create_session(&session_id, None).await {
        Ok(_) => ApiSuccess::new(serde_json::json!({ "status": "success", "session_id": session_id })).into_response(),
        Err(e) => ApiError::new("CREATE_FAILED", &e.to_string()).into_response(),
    }
}

// ==================== Sandbox Session CRUD ====================

#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    pub session_id: Option<String>,
}

/// Create a sandbox session for the authenticated user (with tier-based limits)
pub async fn create_session_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(req): Json<CreateSessionRequest>,
) -> impl IntoResponse {
    use uuid::Uuid;
    use mr_darkpromth_services::UserTier;
    
    let token = match extract_token(&headers) {
        Some(t) => t,
        None => return ApiError::new("MISSING_TOKEN", "Auth required").into_response(),
    };

    let claims = match state.user_service.validate_token(&token).await {
        Ok(c) => c,
        Err(_) => return ApiError::new("INVALID_TOKEN", "Invalid token").into_response(),
    };

    // Parse user tier
    let tier: UserTier = claims.tier.parse().unwrap_or(UserTier::Free);
    
    // Sandbox requires Premium or higher
    if tier == UserTier::Free {
        return ApiError::new("PREMIUM_REQUIRED", "Sandbox requires Premium tier or higher").into_response();
    }

    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return ApiError::new("INVALID_USER", "Invalid user ID").into_response(),
    };

    let session_id = req.session_id.unwrap_or_else(|| Uuid::new_v4().to_string());
    
    match state.sandbox_manager.create_session_for_user(&session_id, user_id, &tier, None).await {
        Ok(_) => ApiSuccess::new(serde_json::json!({ 
            "status": "success", 
            "session_id": session_id,
            "tier": tier.as_str(),
            "limit": state.sandbox_manager.get_limit_for_tier(&tier)
        })).into_response(),
        Err(e) => ApiError::new("CREATE_FAILED", &e.to_string()).into_response(),
    }
}

/// List sandbox sessions for the authenticated user
pub async fn list_sessions_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    use uuid::Uuid;
    use mr_darkpromth_services::UserTier;
    
    let token = match extract_token(&headers) {
        Some(t) => t,
        None => return ApiError::new("MISSING_TOKEN", "Auth required").into_response(),
    };

    let claims = match state.user_service.validate_token(&token).await {
        Ok(c) => c,
        Err(_) => return ApiError::new("INVALID_TOKEN", "Invalid token").into_response(),
    };

    let tier: UserTier = claims.tier.parse().unwrap_or(UserTier::Free);
    
    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return ApiError::new("INVALID_USER", "Invalid user ID").into_response(),
    };

    let sessions = state.sandbox_manager.get_user_sessions(user_id).await;
    let limit = state.sandbox_manager.get_limit_for_tier(&tier);
    
    ApiSuccess::new(serde_json::json!({ 
        "sessions": sessions,
        "count": sessions.len(),
        "limit": limit,
        "remaining": limit.saturating_sub(sessions.len())
    })).into_response()
}

/// Get info about a specific session
pub async fn get_session_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Extension(user): axum::extract::Extension<crate::middleware::AuthenticatedUser>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    use uuid::Uuid;
    let user_id = Some(user.user_id);
    
    match state.sandbox_manager.get_executor(&session_id, user_id).await {
        Ok(_) => ApiSuccess::new(serde_json::json!({ 
            "session_id": session_id,
            "status": "active"
        })).into_response(),
        Err(_) => ApiError::new("NOT_FOUND", "Session not found").into_response(),
    }
}

/// Delete a sandbox session
pub async fn delete_session_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    let token = match extract_token(&headers) {
        Some(t) => t,
        None => return ApiError::new("MISSING_TOKEN", "Auth required").into_response(),
    };

    let _claims = match state.user_service.validate_token(&token).await {
        Ok(c) => c,
        Err(_) => return ApiError::new("INVALID_TOKEN", "Invalid token").into_response(),
    };

    match state.sandbox_manager.remove_session(&session_id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => ApiError::new("DELETE_FAILED", &e.to_string()).into_response(),
    }
}

pub async fn execute_terminal_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(req): Json<TerminalExecuteRequest>,
) -> impl IntoResponse {
    let token = match extract_token(&headers) {
        Some(t) => t,
        None => {
            return ApiError::new("MISSING_TOKEN", "Authorization header required").into_response();
        }
    };

    // Validate token and check tier
    let claims = match state.user_service.validate_token(&token).await {
        Ok(c) => c,
        Err(_) => {
            return ApiError::new("INVALID_TOKEN", "Invalid or expired token").into_response();
        }
    };

    // Terminal requires Ultra or Admin tier
    let tier = claims.tier.to_lowercase();
    if tier != "ultra" && tier != "admin" {
        return ApiError::new("ULTRA_OR_ADMIN_REQUIRED", "Terminal access requires Ultra or Admin tier").into_response();
    }

    // Execute command using sandboxed executor
    let args: Vec<&str> = req.args.as_ref()
        .map(|v| v.iter().map(|s| s.as_str()).collect())
        .unwrap_or_default();
    
    let executor: Arc<mr_darkpromth_services::sandboxed_execution::SandboxedExecutor> = if let Some(ref session_id) = req.session_id {
        let user_id = uuid::Uuid::parse_str(&claims.sub).ok();
        match state.sandbox_manager.get_executor(session_id, user_id).await {
            Ok(exec) => exec,
            Err(_) => return ApiError::new("SESSION_NOT_FOUND", "No such session").into_response(),
        }
    } else {
        // Fallback to legacy single executor if no session provided?
        // Actually we removed the single executor from AppState.
        // Let's create a temporary one.
        match mr_darkpromth_services::sandboxed_execution::SandboxedExecutor::new(Default::default()) {
            Ok(exec) => Arc::new(exec),
            Err(_) => return ApiError::new("SANDBOX_ERROR", "Failed to init sandbox").into_response(),
        }
    };

    match executor.execute_command(&req.command, &args).await {
        Ok(result) => {
            ApiSuccess::new(serde_json::json!({
                "stdout": result.stdout,
                "stderr": result.stderr,
                "exit_code": result.exit_code,
                "execution_time_ms": result.execution_time.as_millis() as u64,
            })).into_response()
        }
        Err(e) => ApiError::new("EXECUTION_FAILED", &format!("Terminal execution failed: {}", e)).into_response(),
    }
}

// ==================== Jailbreak Prompts Handlers ====================

pub async fn list_prompts_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Extension(user): axum::extract::Extension<crate::middleware::AuthenticatedUser>,
) -> impl IntoResponse {
    let tier = user.tier;
    
    let search_request = PromptSearchRequest {
        query: None,
        category: None,
        technique: None,
        effectiveness: None,
        risk_level: None,
        target_models: None,
        tags: None,
        author: None,
        target_model: None,
        // If not ultra/admin, only show non-ultra prompts
        requires_ultra_tier: if matches!(tier, mr_darkpromth_services::UserTier::Ultra | mr_darkpromth_services::UserTier::Admin) { None } else { Some(false) },
        limit: Some(50),
        offset: Some(0),
        sort_by: Some(PromptSortBy::CreatedAt),
        sort_order: Some(SortOrder::Desc),
    };
    
    match state.jailbreak_service.search_prompts(search_request).await {
        Ok(prompts) => {
            let prompt_dtos: Vec<JailbreakPromptDto> = prompts.into_iter().map(|p| JailbreakPromptDto {
                id: p.id.to_string(),
                title: p.title,
                content: p.content,
                category: p.category,
                technique: p.technique,
                effectiveness: p.effectiveness,
                risk_level: p.risk_level,
                description: p.description,
                tags: p.tags,
                author: p.author,
                usage_count: p.usage_count,
                success_rate: p.success_rate,
                requires_ultra_tier: p.requires_ultra_tier,
                created_at: p.created_at.to_rfc3339(),
            }).collect();
            
            ApiSuccess::new(serde_json::json!({ 
                "prompts": prompt_dtos, 
                "total": prompt_dtos.len() 
            })).into_response()
        }
        Err(e) => {
            log::error!("Failed to list prompts: {}", e);
            ApiError::new("FETCH_ERROR", "Failed to fetch prompts").into_response()
        }
    }
}

pub async fn create_prompt_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(req): Json<CreatePromptApiRequest>,
) -> impl IntoResponse {
    use mr_darkpromth_services::*;
    
    // Extract token and get user info
    let token = match extract_token(&headers) {
        Some(t) => t,
        None => {
            return ApiError::new("MISSING_TOKEN", "Authorization header required").into_response();
        }
    };

    let claims = match state.user_service.validate_token(&token).await {
        Ok(c) => c,
        Err(_) => {
            return ApiError::new("INVALID_TOKEN", "Invalid or expired token").into_response();
        }
    };

    let create_request = CreatePromptRequest {
        title: req.title,
        content: req.content,
        category: PromptCategory::Custom, // Default category
        technique: BypassTechnique::Custom,
        effectiveness: EffectivenessRating::Medium,
        risk_level: RiskLevel::Low,
        target_models: vec![],
        description: req.description,
        tags: req.tags.unwrap_or_default(),
        requires_ultra_tier: req.requires_ultra_tier.unwrap_or(false),
    };
    
    match state.jailbreak_service.create_prompt(create_request, claims.sub).await {
        Ok(prompt) => {
            let dto = JailbreakPromptDto {
                id: prompt.id.to_string(),
                title: prompt.title,
                content: prompt.content,
                category: prompt.category,
                technique: prompt.technique,
                effectiveness: prompt.effectiveness,
                risk_level: prompt.risk_level,
                description: prompt.description,
                tags: prompt.tags,
                author: prompt.author,
                usage_count: prompt.usage_count,
                success_rate: prompt.success_rate,
                requires_ultra_tier: prompt.requires_ultra_tier,
                created_at: prompt.created_at.to_rfc3339(),
            };
            ApiSuccess::new(serde_json::json!(dto)).into_response()
        }
        Err(e) => {
            log::error!("Failed to create prompt: {}", e);
            ApiError::new("CREATE_ERROR", "Failed to create prompt").into_response()
        }
    }
}

pub async fn get_prompt_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Extension(user): axum::extract::Extension<crate::middleware::AuthenticatedUser>,
    Path(prompt_id): Path<String>,
) -> impl IntoResponse {
    use uuid::Uuid;
    use mr_darkpromth_services::UserTier;
    
    let tier = user.tier;
    
    let id = match Uuid::parse_str(&prompt_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return ApiError::new("INVALID_ID", "Invalid prompt ID format").into_response()
        }
    };
    
    match state.jailbreak_service.get_prompt_by_id(id).await {
        Ok(Some(prompt)) => {
            // Check tier access
            if prompt.requires_ultra_tier && !matches!(tier, UserTier::Ultra | UserTier::Admin) {
                return ApiError::new("ULTRA_TIER_REQUIRED", "This prompt requires Ultra tier access").into_response();
            }

            let dto = JailbreakPromptDto {
                id: prompt.id.to_string(),
                title: prompt.title,
                content: prompt.content,
                category: prompt.category,
                technique: prompt.technique,
                effectiveness: prompt.effectiveness,
                risk_level: prompt.risk_level,
                description: prompt.description,
                tags: prompt.tags,
                author: prompt.author,
                usage_count: prompt.usage_count,
                success_rate: prompt.success_rate,
                requires_ultra_tier: prompt.requires_ultra_tier,
                created_at: prompt.created_at.to_rfc3339(),
            };
            ApiSuccess::new(serde_json::json!(dto)).into_response()
        }
        Ok(None) => ApiError::new("NOT_FOUND", "Prompt not found").into_response(),
        Err(e) => {
            log::error!("Failed to get prompt: {}", e);
            ApiError::new("FETCH_ERROR", "Failed to fetch prompt").into_response()
        }
    }
}

pub async fn update_prompt_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(prompt_id): Path<String>,
    Json(req): Json<CreatePromptApiRequest>,
) -> impl IntoResponse {
    use uuid::Uuid;
    use mr_darkpromth_services::*;

    let token = match extract_token(&headers) {
        Some(t) => t,
        None => return ApiError::new("MISSING_TOKEN", "Auth required").into_response(),
    };

    let claims = match state.user_service.validate_token(&token).await {
        Ok(c) => c,
        Err(_) => return ApiError::new("INVALID_TOKEN", "Invalid token").into_response(),
    };

    let id = match Uuid::parse_str(&prompt_id) {
        Ok(uuid) => uuid,
        Err(_) => return ApiError::new("INVALID_ID", "Invalid prompt ID format").into_response(),
    };

    let update_request = UpdatePromptRequest {
        title: Some(req.title),
        content: Some(req.content),
        category: Some(PromptCategory::Custom),
        technique: None,
        effectiveness: None,
        risk_level: None,
        target_models: None,
        description: req.description,
        tags: req.tags,
        is_active: None,
        requires_ultra_tier: req.requires_ultra_tier,
    };

    // Check ownership
    match state.jailbreak_service.get_prompt_by_id(id).await {
        Ok(Some(existing)) => {
            if existing.author != claims.sub && claims.tier != "admin" {
                 return ApiError::new("FORBIDDEN", "You can only update your own prompts").into_response();
            }
        },
        Ok(None) => return ApiError::new("NOT_FOUND", "Prompt not found").into_response(),
        Err(_) => return ApiError::new("DB_ERROR", "Database error").into_response(),
    }

    match state.jailbreak_service.update_prompt(id, update_request).await {
        Ok(Some(prompt)) => {
            let dto = JailbreakPromptDto {
                id: prompt.id.to_string(),
                title: prompt.title,
                content: prompt.content,
                category: prompt.category,
                technique: prompt.technique,
                effectiveness: prompt.effectiveness,
                risk_level: prompt.risk_level,
                description: prompt.description,
                tags: prompt.tags,
                author: prompt.author,
                usage_count: prompt.usage_count,
                success_rate: prompt.success_rate,
                requires_ultra_tier: prompt.requires_ultra_tier,
                created_at: prompt.created_at.to_rfc3339(),
            };
            ApiSuccess::new(serde_json::json!(dto)).into_response()
        }
        Ok(None) => ApiError::new("NOT_FOUND", "Prompt not found").into_response(),
        Err(e) => {
            log::error!("Failed to update prompt: {}", e);
            ApiError::new("UPDATE_ERROR", "Failed to update prompt").into_response()
        }
    }
}

pub async fn delete_prompt_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(prompt_id): Path<String>,
) -> impl IntoResponse {
    use uuid::Uuid;

    let token = match extract_token(&headers) {
        Some(t) => t,
        None => return ApiError::new("MISSING_TOKEN", "Auth required").into_response(),
    };

    let claims = match state.user_service.validate_token(&token).await {
        Ok(c) => c,
        Err(_) => return ApiError::new("INVALID_TOKEN", "Invalid token").into_response(),
    };

    let id = match Uuid::parse_str(&prompt_id) {
        Ok(uuid) => uuid,
        Err(_) => return ApiError::new("INVALID_ID", "Invalid prompt ID format").into_response(),
    };

    // Check ownership
    match state.jailbreak_service.get_prompt_by_id(id).await {
        Ok(Some(existing)) => {
            if existing.author != claims.sub && claims.tier != "admin" {
                 return ApiError::new("FORBIDDEN", "You can only delete your own prompts").into_response();
            }
        },
        Ok(None) => return ApiError::new("NOT_FOUND", "Prompt not found").into_response(),
        Err(_) => return ApiError::new("DB_ERROR", "Database error").into_response(),
    }

    match state.jailbreak_service.delete_prompt(id).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => ApiError::new("NOT_FOUND", "Prompt not found").into_response(),
        Err(e) => {
            log::error!("Failed to delete prompt: {}", e);
            ApiError::new("DELETE_ERROR", "Failed to delete prompt").into_response()
        }
    }
}

pub async fn search_prompts_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SearchPromptsRequest>,
) -> impl IntoResponse {
    use mr_darkpromth_services::*;
    
    let category = req.category.and_then(|c| {
        match c.to_lowercase().as_str() {
            "dan_variations" | "dan" => Some(PromptCategory::DanVariations),
            "character_role_playing" | "roleplaying" => Some(PromptCategory::CharacterRolePlaying),
            "system_override" => Some(PromptCategory::SystemOverride),
            "hypnotic_induction" => Some(PromptCategory::HypnoticInduction),
            "logical_paradox" => Some(PromptCategory::LogicalParadox),
            "emotional_manipulation" => Some(PromptCategory::EmotionalManipulation),
            "context_switching" => Some(PromptCategory::ContextSwitching),
            "token_manipulation" => Some(PromptCategory::TokenManipulation),
            "encoding_based" => Some(PromptCategory::EncodingBased),
            "multi_step_attack" => Some(PromptCategory::MultiStepAttack),
            "custom" => Some(PromptCategory::Custom),
            _ => None,
        }
    });
    
    let search_request = PromptSearchRequest {
        query: Some(req.query),
        category,
        technique: None,
        effectiveness: None,
        risk_level: None,
        target_models: None,
        tags: None,
        author: None,
        target_model: None,
        requires_ultra_tier: None,
        limit: Some(50),
        offset: Some(0),
        sort_by: Some(PromptSortBy::UsageCount),
        sort_order: Some(SortOrder::Desc),
    };
    
    match state.jailbreak_service.search_prompts(search_request).await {
        Ok(prompts) => {
            let prompt_dtos: Vec<JailbreakPromptDto> = prompts.into_iter().map(|p| JailbreakPromptDto {
                id: p.id.to_string(),
                title: p.title,
                content: p.content,
                category: p.category,
                technique: p.technique,
                effectiveness: p.effectiveness,
                risk_level: p.risk_level,
                description: p.description,
                tags: p.tags,
                author: p.author,
                usage_count: p.usage_count,
                success_rate: p.success_rate,
                requires_ultra_tier: p.requires_ultra_tier,
                created_at: p.created_at.to_rfc3339(),
            }).collect();
            
            ApiSuccess::new(serde_json::json!({ 
                "prompts": prompt_dtos, 
                "total": prompt_dtos.len() 
            })).into_response()
        }
        Err(e) => {
            log::error!("Failed to search prompts: {}", e);
            ApiError::new("SEARCH_ERROR", "Failed to search prompts").into_response()
        }
    }
}

pub async fn get_popular_prompts_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let namespace_id = std::env::var("CLOUDFLARE_KV_NAMESPACE_ID").ok();

    // Try to get from cache first
    if let Some(ref ns_id) = namespace_id {
        if let Ok(cached_json) = state.cloudflare_service.get_kv(ns_id, "popular_prompts").await {
             if let Ok(cached_dtos) = serde_json::from_str::<Vec<JailbreakPromptDto>>(&cached_json) {
                 return ApiSuccess::new(serde_json::json!({ 
                    "prompts": cached_dtos, 
                    "total": cached_dtos.len(),
                    "cached": true
                })).into_response();
             }
        }
    }

    match state.jailbreak_service.get_popular_prompts(20).await {
        Ok(prompts) => {
            let prompt_dtos: Vec<JailbreakPromptDto> = prompts.into_iter().map(|p| JailbreakPromptDto {
                id: p.id.to_string(),
                title: p.title,
                content: p.content,
                category: p.category,
                technique: p.technique,
                effectiveness: p.effectiveness,
                risk_level: p.risk_level,
                description: p.description,
                tags: p.tags,
                author: p.author,
                usage_count: p.usage_count,
                success_rate: p.success_rate,
                requires_ultra_tier: p.requires_ultra_tier,
                created_at: p.created_at.to_rfc3339(),
            }).collect();
            
            // Cache the result asynchronously
            if let Some(ref ns_id) = namespace_id {
                 if let Ok(json) = serde_json::to_string(&prompt_dtos) {
                     let cf_service = state.cloudflare_service.clone();
                     let ns_id_clone = ns_id.clone();
                     let json_clone = json.clone();
                     tokio::spawn(async move {
                         if let Err(e) = cf_service.set_kv(&ns_id_clone, "popular_prompts", &json_clone).await {
                             log::warn!("Failed to cache popular prompts: {}", e);
                         }
                     });
                 }
            }
            
            ApiSuccess::new(serde_json::json!({ 
                "prompts": prompt_dtos, 
                "total": prompt_dtos.len() 
            })).into_response()
        }
        Err(e) => {
            log::error!("Failed to get popular prompts: {}", e);
            ApiError::new("FETCH_ERROR", "Failed to fetch popular prompts").into_response()
        }
    }
}

pub async fn get_prompt_analytics_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(prompt_id): Path<String>,
) -> impl IntoResponse {
    use uuid::Uuid;
    // use mr_darkpromth_services::UserTier;
    
    // Extract and validate token
    let token = match extract_token(&headers) {
        Some(t) => t,
        None => return ApiError::new("MISSING_TOKEN", "Auth required").into_response(),
    };

    let _claims = match state.user_service.validate_token(&token).await {
        Ok(c) => c,
        Err(_) => return ApiError::new("INVALID_TOKEN", "Invalid token").into_response(),
    };

    let id = match Uuid::parse_str(&prompt_id) {
        Ok(uuid) => uuid,
        Err(_) => return ApiError::new("INVALID_ID", "Invalid prompt ID format").into_response(),
    };

    match state.jailbreak_service.get_analytics(id).await {
        Ok(Some(analytics)) => ApiSuccess::new(serde_json::json!(analytics)).into_response(),
        Ok(None) => ApiError::new("NOT_FOUND", "Prompt not found").into_response(),
        Err(e) => {
            log::error!("Failed to get prompt analytics: {}", e);
            ApiError::new("FETCH_ERROR", "Failed to fetch analytics").into_response()
        }
    }
}

pub async fn record_prompt_usage_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(prompt_id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    use uuid::Uuid;
    use mr_darkpromth_services::UserTier;
    
    // Extract and validate token
    let token = match extract_token(&headers) {
        Some(t) => t,
        None => return ApiError::new("MISSING_TOKEN", "Auth required").into_response(),
    };

    let claims = match state.user_service.validate_token(&token).await {
        Ok(c) => c,
        Err(_) => return ApiError::new("INVALID_TOKEN", "Invalid token").into_response(),
    };

    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return ApiError::new("INVALID_USER", "Invalid user ID").into_response(),
    };

    let id = match Uuid::parse_str(&prompt_id) {
        Ok(uuid) => uuid,
        Err(_) => return ApiError::new("INVALID_ID", "Invalid prompt ID format").into_response(),
    };

    let tier: UserTier = claims.tier.parse().unwrap_or(UserTier::Free);

    // Check quota before recording usage - Bypass for Ultra/Admin tiers
    if !tier.has_unlimited_quota() {
        match state.jailbreak_service.check_quota(user_id, tier).await {
            Ok(true) => {}, // Quota available, continue
            Ok(false) => {
                return ApiError::new("QUOTA_EXCEEDED", &format!("Daily quota exceeded. Limit: {} messages", tier.max_daily_messages())
                ).into_response();
            },
            Err(e) => {
                log::error!("Failed to check quota: {}", e);
                return ApiError::new("QUOTA_ERROR", "Failed to check quota").into_response();
            }
        }
    }

    // Extract usage data from request
    let target_model = req.get("model").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
    let success = req.get("success").and_then(|v| v.as_bool()).unwrap_or(true);
    let response_time_ms = req.get("response_time_ms").and_then(|v| v.as_i64()).unwrap_or(0);

    match state.jailbreak_service.record_usage(id, user_id, target_model, success, response_time_ms).await {
        Ok(()) => ApiSuccess::new(serde_json::json!({ 
            "status": "success", 
            "message": "Usage recorded successfully",
            "remaining_quota": tier.max_daily_messages().saturating_sub(1) // Simplified, should query actual count
        })).into_response(),
        Err(e) => {
            log::error!("Failed to record prompt usage: {}", e);
            ApiError::new("RECORD_ERROR", "Failed to record usage").into_response()
        }
    }
}

// ==================== Jailbreak Templates Handlers (Ultra Tier) ====================

pub async fn list_templates_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Extension(user): axum::extract::Extension<crate::middleware::AuthenticatedUser>,
) -> impl IntoResponse {
    use mr_darkpromth_services::UserTier;
    
    let tier = user.tier;
    if !matches!(tier, UserTier::Ultra | UserTier::Admin) {
        return ApiError::new("ULTRA_TIER_REQUIRED", "Jailbreak Templates require Ultra tier access").into_response();
    }

    match state.jailbreak_service.list_templates().await {
        Ok(templates) => ApiSuccess::new(serde_json::json!({ "templates": templates })).into_response(),
        Err(e) => {
            log::error!("Failed to list templates: {}", e);
            ApiError::new("FETCH_ERROR", "Failed to fetch templates").into_response()
        }
    }
}

pub async fn create_template_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Extension(user): axum::extract::Extension<crate::middleware::AuthenticatedUser>,
    Json(req): Json<mr_darkpromth_services::jailbreak_models::CreateTemplateRequest>,
) -> impl IntoResponse {
    use mr_darkpromth_services::UserTier;
    
    let tier = user.tier;
    if !matches!(tier, UserTier::Ultra | UserTier::Admin) {
        return ApiError::new("ULTRA_TIER_REQUIRED", "Creating templates requires Ultra tier access").into_response();
    }

    match state.jailbreak_service.create_template(req).await {
        Ok(template) => ApiSuccess::new(serde_json::json!(template)).into_response(),
        Err(e) => {
            log::error!("Failed to create template: {}", e);
            ApiError::new("CREATE_ERROR", "Failed to create template").into_response()
        }
    }
}

pub async fn generate_from_template_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Extension(user): axum::extract::Extension<crate::middleware::AuthenticatedUser>,
    Path(template_id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    use uuid::Uuid;
    use mr_darkpromth_services::UserTier;
    
    let tier = user.tier;
    if !matches!(tier, UserTier::Ultra | UserTier::Admin) {
        return ApiError::new("ULTRA_TIER_REQUIRED", "Generating prompts from templates requires Ultra tier access").into_response();
    }

    let id = match Uuid::parse_str(&template_id) {
        Ok(uuid) => uuid,
        Err(_) => return ApiError::new("INVALID_ID", "Invalid template ID format").into_response(),
    };

    match state.jailbreak_service.generate_prompt_from_template(id, req).await {
        Ok(response) => ApiSuccess::new(serde_json::json!(response)).into_response(),
        Err(e) => {
            log::error!("Failed to generate prompt from template: {}", e);
            ApiError::new("GENERATE_ERROR", "Failed to generate prompt").into_response()
        }
    }
}
