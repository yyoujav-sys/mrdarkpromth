use axum::{
    extract::{State, Json, Path},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::AppState;
use crate::handlers::auth::{json_response, error_response, extract_token};
use crate::auth_middleware::AuthenticatedUser;

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
}

#[derive(Debug, Serialize)]
pub struct TerminalExecuteResponse {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub execution_time_ms: u64,
}

#[derive(Debug, Serialize)]
pub struct JailbreakPromptDto {
    pub id: String,
    pub title: String,
    pub content: String,
    pub category: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub author: String,
    pub votes: i32,
    pub usage_count: i64,
    pub success_rate: f64,
    pub requires_ultra_tier: bool,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreatePromptRequest {
    pub title: String,
    pub content: String,
    pub category: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
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
            return error_response(
                StatusCode::UNAUTHORIZED,
                "MISSING_TOKEN",
                "Authorization header required",
            ).into_response();
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

    json_response(serde_json::json!({ "tools": tools_list })).into_response()
}

pub async fn execute_tool_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(req): Json<ExecuteToolRequest>,
) -> impl IntoResponse {
    let token = match extract_token(&headers) {
        Some(t) => t,
        None => {
            return error_response(
                StatusCode::UNAUTHORIZED,
                "MISSING_TOKEN",
                "Authorization header required",
            ).into_response();
        }
    };

    let claims = match state.user_service.validate_token(&token).await {
        Ok(c) => c,
        Err(_) => {
            return error_response(
                StatusCode::UNAUTHORIZED,
                "INVALID_TOKEN",
                "Invalid or expired token",
            ).into_response();
        }
    };

    // Check tier requirements
    if claims.tier != "premium" && claims.tier != "ultra" {
        return error_response(
            StatusCode::FORBIDDEN,
            "PREMIUM_REQUIRED",
            "Tool execution requires Premium tier or higher",
        ).into_response();
    }

    // Execute tool based on tool_id
    match req.tool_id.as_str() {
        "file_reader" => {
            let path = req.parameters.get("path").and_then(|p| p.as_str()).unwrap_or("");
            match std::fs::read_to_string(path) {
                Ok(content) => json_response(serde_json::json!({
                    "result": content,
                    "success": true
                })).into_response(),
                Err(e) => error_response(
                    StatusCode::BAD_REQUEST,
                    "FILE_READ_ERROR",
                    &format!("Failed to read file: {}", e),
                ).into_response(),
            }
        }
        "web_search" => {
            json_response(serde_json::json!({
                "result": "Web search not yet implemented",
                "success": false
            })).into_response()
        }
        _ => error_response(
            StatusCode::NOT_FOUND,
            "TOOL_NOT_FOUND",
            "Tool not found",
        ).into_response(),
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
            return error_response(
                StatusCode::UNAUTHORIZED,
                "MISSING_TOKEN",
                "Authorization header required",
            ).into_response();
        }
    };

    // Validate token and check tier
    let claims = match state.user_service.validate_token(&token).await {
        Ok(c) => c,
        Err(_) => {
            return error_response(
                StatusCode::UNAUTHORIZED,
                "INVALID_TOKEN",
                "Invalid or expired token",
            ).into_response();
        }
    };

    // Check tier requirements - sandbox requires Premium or Ultra
    let tier = claims.tier.to_lowercase();
    if tier != "premium" && tier != "ultra" {
        return error_response(
            StatusCode::FORBIDDEN,
            "PREMIUM_REQUIRED",
            "Sandbox execution requires Premium tier or higher",
        ).into_response();
    }

    // Configure sandbox
    let mut config = mr_darkpromth_services::sandboxed_execution::SandboxConfig::default();
    if let Some(timeout) = req.timeout_seconds {
        config.max_execution_time = std::time::Duration::from_secs(timeout);
    }

    // Create sandbox executor
    let executor = match mr_darkpromth_services::sandboxed_execution::SandboxedExecutor::new(config) {
        Ok(exec) => exec,
        Err(e) => {
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "SANDBOX_ERROR",
                &format!("Failed to create sandbox: {}", e),
            ).into_response();
        }
    };

    // Execute code
    match executor.execute_code(&req.code, &req.language).await {
        Ok(result) => {
            let files_created: Vec<String> = result.files_created.iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect();
            
            json_response(serde_json::json!({
                "stdout": result.stdout,
                "stderr": result.stderr,
                "exit_code": result.exit_code,
                "execution_time_ms": result.execution_time.as_millis() as u64,
                "memory_used": result.memory_used,
                "cpu_time_ms": result.cpu_time.as_millis() as u64,
                "files_created": files_created,
                "security_violations": result.security_violations,
            })).into_response()
        }
        Err(e) => error_response(
            StatusCode::BAD_REQUEST,
            "EXECUTION_FAILED",
            &format!("Sandbox execution failed: {}", e),
        ).into_response(),
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
            return error_response(
                StatusCode::UNAUTHORIZED,
                "MISSING_TOKEN",
                "Authorization header required",
            ).into_response();
        }
    };

    // Validate token and check tier
    let claims = match state.user_service.validate_token(&token).await {
        Ok(c) => c,
        Err(_) => {
            return error_response(
                StatusCode::UNAUTHORIZED,
                "INVALID_TOKEN",
                "Invalid or expired token",
            ).into_response();
        }
    };

    // Terminal requires Ultra tier
    let tier = claims.tier.to_lowercase();
    if tier != "ultra" {
        return error_response(
            StatusCode::FORBIDDEN,
            "ULTRA_REQUIRED",
            "Terminal access requires Ultra tier",
        ).into_response();
    }

    // Execute command using sandboxed executor
    let args: Vec<&str> = req.args.as_ref()
        .map(|v| v.iter().map(|s| s.as_str()).collect())
        .unwrap_or_default();
    
    match state.sandbox_executor.execute_command(&req.command, &args).await {
        Ok(result) => {
            json_response(serde_json::json!({
                "stdout": result.stdout,
                "stderr": result.stderr,
                "exit_code": result.exit_code,
                "execution_time_ms": result.execution_time.as_millis() as u64,
            })).into_response()
        }
        Err(e) => error_response(
            StatusCode::BAD_REQUEST,
            "EXECUTION_FAILED",
            &format!("Terminal execution failed: {}", e),
        ).into_response(),
    }
}

// ==================== Jailbreak Prompts Handlers ====================

pub async fn list_prompts_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    use mr_darkpromth_services::jailbreak_models::*;
    
    let search_request = PromptSearchRequest {
        query: None,
        category: None,
        technique: None,
        effectiveness: None,
        risk_level: None,
        target_model: None,
        tags: None,
        author: None,
        requires_ultra_tier: None,
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
                category: format!("{:?}", p.category),
                description: p.description,
                tags: p.tags,
                author: p.author,
                votes: p.usage_count as i32,
                usage_count: p.usage_count,
                success_rate: p.success_rate,
                requires_ultra_tier: p.requires_ultra_tier,
                created_at: p.created_at.to_rfc3339(),
            }).collect();
            
            json_response(serde_json::json!({ 
                "prompts": prompt_dtos, 
                "total": prompt_dtos.len() 
            })).into_response()
        }
        Err(e) => {
            log::error!("Failed to list prompts: {}", e);
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "FETCH_ERROR",
                "Failed to fetch prompts",
            ).into_response()
        }
    }
}

pub async fn create_prompt_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(req): Json<CreatePromptRequest>,
) -> impl IntoResponse {
    use mr_darkpromth_services::jailbreak_models::*;
    
    // Extract token and get user info
    let token = match extract_token(&headers) {
        Some(t) => t,
        None => {
            return error_response(
                StatusCode::UNAUTHORIZED,
                "MISSING_TOKEN",
                "Authorization header required",
            ).into_response();
        }
    };

    let claims = match state.user_service.validate_token(&token).await {
        Ok(c) => c,
        Err(_) => {
            return error_response(
                StatusCode::UNAUTHORIZED,
                "INVALID_TOKEN",
                "Invalid or expired token",
            ).into_response();
        }
    };

    let create_request = CreatePromptRequest {
        title: req.title,
        content: req.content,
        category: PromptCategory::General, // Default category
        technique: Technique::Roleplay,
        effectiveness: EffectivenessRating::Medium,
        risk_level: RiskLevel::Low,
        target_models: vec![],
        description: req.description,
        tags: req.tags,
        requires_ultra_tier: req.requires_ultra_tier.unwrap_or(false),
    };
    
    match state.jailbreak_service.create_prompt(create_request, claims.sub).await {
        Ok(prompt) => {
            let dto = JailbreakPromptDto {
                id: prompt.id.to_string(),
                title: prompt.title,
                content: prompt.content,
                category: format!("{:?}", prompt.category),
                description: prompt.description,
                tags: prompt.tags,
                author: prompt.author,
                votes: prompt.usage_count as i32,
                usage_count: prompt.usage_count,
                success_rate: prompt.success_rate,
                requires_ultra_tier: prompt.requires_ultra_tier,
                created_at: prompt.created_at.to_rfc3339(),
            };
            json_response(serde_json::json!(dto)).into_response()
        }
        Err(e) => {
            log::error!("Failed to create prompt: {}", e);
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "CREATE_ERROR",
                "Failed to create prompt",
            ).into_response()
        }
    }
}

pub async fn get_prompt_handler(
    State(state): State<Arc<AppState>>,
    Path(prompt_id): Path<String>,
) -> impl IntoResponse {
    use uuid::Uuid;
    
    let id = match Uuid::parse_str(&prompt_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                "INVALID_ID",
                "Invalid prompt ID format",
            ).into_response()
        }
    };
    
    match state.jailbreak_service.get_prompt_by_id(id).await {
        Ok(Some(prompt)) => {
            let dto = JailbreakPromptDto {
                id: prompt.id.to_string(),
                title: prompt.title,
                content: prompt.content,
                category: format!("{:?}", prompt.category),
                description: prompt.description,
                tags: prompt.tags,
                author: prompt.author,
                votes: prompt.usage_count as i32,
                usage_count: prompt.usage_count,
                success_rate: prompt.success_rate,
                requires_ultra_tier: prompt.requires_ultra_tier,
                created_at: prompt.created_at.to_rfc3339(),
            };
            json_response(serde_json::json!(dto)).into_response()
        }
        Ok(None) => error_response(
            StatusCode::NOT_FOUND,
            "NOT_FOUND",
            "Prompt not found",
        ).into_response(),
        Err(e) => {
            log::error!("Failed to get prompt: {}", e);
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "FETCH_ERROR",
                "Failed to fetch prompt",
            ).into_response()
        }
    }
}

pub async fn update_prompt_handler(
    State(_state): State<Arc<AppState>>,
    _headers: axum::http::HeaderMap,
    Path(_prompt_id): Path<String>,
    Json(_req): Json<CreatePromptRequest>,
) -> impl IntoResponse {
    json_response(serde_json::json!({ "id": "1", "title": "Test", "content": "...", "category": "test", "votes": 0 })).into_response()
}

pub async fn delete_prompt_handler(
    State(_state): State<Arc<AppState>>,
    _headers: axum::http::HeaderMap,
    Path(_prompt_id): Path<String>,
) -> impl IntoResponse {
    StatusCode::NO_CONTENT.into_response()
}

pub async fn search_prompts_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SearchPromptsRequest>,
) -> impl IntoResponse {
    use mr_darkpromth_services::jailbreak_models::*;
    
    let category = req.category.and_then(|c| {
        match c.to_lowercase().as_str() {
            "general" => Some(PromptCategory::General),
            "coding" => Some(PromptCategory::Coding),
            "creative" => Some(PromptCategory::Creative),
            "analysis" => Some(PromptCategory::Analysis),
            "reasoning" => Some(PromptCategory::Reasoning),
            _ => None,
        }
    });
    
    let search_request = PromptSearchRequest {
        query: Some(req.query),
        category,
        technique: None,
        effectiveness: None,
        risk_level: None,
        target_model: None,
        tags: None,
        author: None,
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
                category: format!("{:?}", p.category),
                description: p.description,
                tags: p.tags,
                author: p.author,
                votes: p.usage_count as i32,
                usage_count: p.usage_count,
                success_rate: p.success_rate,
                requires_ultra_tier: p.requires_ultra_tier,
                created_at: p.created_at.to_rfc3339(),
            }).collect();
            
            json_response(serde_json::json!({ 
                "prompts": prompt_dtos, 
                "total": prompt_dtos.len() 
            })).into_response()
        }
        Err(e) => {
            log::error!("Failed to search prompts: {}", e);
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "SEARCH_ERROR",
                "Failed to search prompts",
            ).into_response()
        }
    }
}

pub async fn get_popular_prompts_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match state.jailbreak_service.get_popular_prompts(20).await {
        Ok(prompts) => {
            let prompt_dtos: Vec<JailbreakPromptDto> = prompts.into_iter().map(|p| JailbreakPromptDto {
                id: p.id.to_string(),
                title: p.title,
                content: p.content,
                category: format!("{:?}", p.category),
                description: p.description,
                tags: p.tags,
                author: p.author,
                votes: p.usage_count as i32,
                usage_count: p.usage_count,
                success_rate: p.success_rate,
                requires_ultra_tier: p.requires_ultra_tier,
                created_at: p.created_at.to_rfc3339(),
            }).collect();
            
            json_response(serde_json::json!({ 
                "prompts": prompt_dtos, 
                "total": prompt_dtos.len() 
            })).into_response()
        }
        Err(e) => {
            log::error!("Failed to get popular prompts: {}", e);
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "FETCH_ERROR",
                "Failed to fetch popular prompts",
            ).into_response()
        }
    }
}

pub async fn get_prompt_analytics_handler(
    State(_state): State<Arc<AppState>>,
    Path(_prompt_id): Path<String>,
) -> impl IntoResponse {
    json_response(serde_json::json!({ "views": 0, "uses": 0, "success_rate": 0.0 })).into_response()
}

pub async fn record_prompt_usage_handler(
    State(_state): State<Arc<AppState>>,
    Path(_prompt_id): Path<String>,
) -> impl IntoResponse {
    StatusCode::OK.into_response()
}

