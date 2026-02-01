use axum::{
    extract::{State, Json, Path},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::AppState;
use crate::handlers::auth::{json_response, error_response, extract_token};

#[derive(Debug, Serialize)]
pub struct ToolInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub tier_required: String,
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
}

#[derive(Debug, Serialize)]
pub struct SandboxExecuteResponse {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub execution_time_ms: u64,
}

#[derive(Debug, Deserialize)]
pub struct TerminalExecuteRequest {
    pub command: String,
    pub working_dir: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct JailbreakPrompt {
    pub id: String,
    pub title: String,
    pub content: String,
    pub category: String,
    pub votes: i32,
}

#[derive(Debug, Deserialize)]
pub struct CreatePromptRequest {
    pub title: String,
    pub content: String,
    pub category: String,
}

#[derive(Debug, Deserialize)]
pub struct SearchPromptsRequest {
    pub query: String,
    pub category: Option<String>,
}

// ==================== Tools Handlers ====================

pub async fn list_tools_handler(
    State(_state): State<Arc<AppState>>,
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

    // TODO: Verify token and check tier
    let tools = vec![
        ToolInfo {
            id: "file_reader".to_string(),
            name: "File Reader".to_string(),
            description: "Read file contents".to_string(),
            tier_required: "premium".to_string(),
        },
        ToolInfo {
            id: "file_writer".to_string(),
            name: "File Writer".to_string(),
            description: "Write content to files".to_string(),
            tier_required: "premium".to_string(),
        },
        ToolInfo {
            id: "web_search".to_string(),
            name: "Web Search".to_string(),
            description: "Search the web".to_string(),
            tier_required: "premium".to_string(),
        },
    ];

    json_response(serde_json::json!({ "tools": tools })).into_response()
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
    State(_state): State<Arc<AppState>>,
    _headers: axum::http::HeaderMap,
    Json(_req): Json<SandboxExecuteRequest>,
) -> impl IntoResponse {
    json_response(serde_json::json!({
        "stdout": "",
        "stderr": "Sandbox not yet implemented",
        "exit_code": -1,
        "execution_time_ms": 0
    })).into_response()
}

pub async fn execute_terminal_handler(
    State(_state): State<Arc<AppState>>,
    _headers: axum::http::HeaderMap,
    Json(_req): Json<TerminalExecuteRequest>,
) -> impl IntoResponse {
    json_response(serde_json::json!({
        "stdout": "",
        "stderr": "Terminal access not yet implemented",
        "exit_code": -1,
        "execution_time_ms": 0
    })).into_response()
}

// ==================== Jailbreak Prompts Handlers ====================

pub async fn list_prompts_handler(
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    json_response(serde_json::json!({ "prompts": [], "total": 0 })).into_response()
}

pub async fn create_prompt_handler(
    State(_state): State<Arc<AppState>>,
    _headers: axum::http::HeaderMap,
    Json(_req): Json<CreatePromptRequest>,
) -> impl IntoResponse {
    json_response(serde_json::json!({ "id": "1", "title": "Test", "content": "...", "category": "test", "votes": 0 })).into_response()
}

pub async fn get_prompt_handler(
    State(_state): State<Arc<AppState>>,
    Path(_prompt_id): Path<String>,
) -> impl IntoResponse {
    json_response(serde_json::json!({ "id": "1", "title": "Test", "content": "...", "category": "test", "votes": 0 })).into_response()
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
    State(_state): State<Arc<AppState>>,
    Json(_req): Json<SearchPromptsRequest>,
) -> impl IntoResponse {
    json_response(serde_json::json!({ "prompts": [], "total": 0 })).into_response()
}

pub async fn get_popular_prompts_handler(
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    json_response(serde_json::json!({ "prompts": [], "total": 0 })).into_response()
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

