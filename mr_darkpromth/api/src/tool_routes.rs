use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::auth_middleware::AuthenticatedUser;
use mr_darkpromth_services::{
    ExecutorConfig, MasterToolExecutor, ToolExecutionContext, ToolRegistry, ToolResult,
};
use mr_darkpromth_services::sandboxed_execution::{SandboxedExecutor, SandboxConfig};

#[derive(Debug, Serialize, ToSchema)]
pub struct ToolSummary {
    pub name: String,
    pub description: String,
    pub category: String,
    pub required_permissions: Vec<String>,
    pub enabled: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ToolListResponse {
    pub tools: Vec<ToolSummary>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ToolExecuteRequest {
    pub tool_name: String,
    pub input: serde_json::Value,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ToolResultDto {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
    pub execution_time_ms: u64,
    pub tool_name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ToolExecuteResponse {
    pub request_id: String,
    pub result: ToolResultDto,
    pub execution_time_ms: u64,
    pub started_at: String,
    pub completed_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SandboxExecuteRequest {
    pub code: String,
    pub language: String,
    pub timeout_seconds: Option<u64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SandboxExecuteResponse {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub execution_time_ms: u64,
    pub memory_used: u64,
    pub cpu_time_ms: u64,
    pub files_created: Vec<String>,
    pub security_violations: Vec<String>,
}

fn normalize_tier(tier: &str) -> String {
    tier.to_lowercase()
}

fn has_advanced_access(tier: &str) -> bool {
    matches!(normalize_tier(tier).as_str(), "premium" | "ultra")
}

fn map_tool_result(result: ToolResult) -> ToolResultDto {
    ToolResultDto {
        success: result.success,
        data: result.data,
        error: result.error,
        execution_time_ms: result.execution_time_ms,
        tool_name: result.tool_name,
    }
}

#[utoipa::path(
    get,
    path = "/api/tools",
    responses(
        (status = 200, description = "List available tools", body = ToolListResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    tag = "tools",
    security(("bearer_auth" = []))
)]
pub async fn list_tools(
    auth_user: Option<AuthenticatedUser>,
    registry: web::Data<Arc<ToolRegistry>>,
) -> impl Responder {
    let user = match auth_user {
        Some(user) => user,
        None => {
            return HttpResponse::Unauthorized().json(ErrorResponse {
                error: "Unauthorized".to_string(),
                message: "Authentication required".to_string(),
            })
        }
    };

    let tools = registry
        .list_tools()
        .into_iter()
        .filter_map(|tool_name| {
            registry.get_tool_metadata(&tool_name).map(|metadata| ToolSummary {
                name: metadata.name,
                description: metadata.schema.description,
                category: metadata.schema.category,
                required_permissions: metadata.schema.required_permissions,
                enabled: metadata.enabled,
            })
        })
        .collect::<Vec<_>>();

    let _ = user; // Keep for potential auditing hooks.

    HttpResponse::Ok().json(ToolListResponse { tools })
}

#[utoipa::path(
    post,
    path = "/api/tools/execute",
    request_body = ToolExecuteRequest,
    responses(
        (status = 200, description = "Tool executed", body = ToolExecuteResponse),
        (status = 400, description = "Execution failed", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse)
    ),
    tag = "tools",
    security(("bearer_auth" = []))
)]
pub async fn execute_tool(
    auth_user: Option<AuthenticatedUser>,
    executor: web::Data<Arc<MasterToolExecutor>>,
    request: web::Json<ToolExecuteRequest>,
) -> impl Responder {
    let user = match auth_user {
        Some(user) => user,
        None => {
            return HttpResponse::Unauthorized().json(ErrorResponse {
                error: "Unauthorized".to_string(),
                message: "Authentication required".to_string(),
            })
        }
    };

    if !has_advanced_access(&user.tier) {
        return HttpResponse::Forbidden().json(ErrorResponse {
            error: "Forbidden".to_string(),
            message: "Premium or Ultra tier required for tool execution".to_string(),
        });
    }

    let mut metadata = HashMap::new();
    metadata.insert("user_id".to_string(), user.user_id.to_string());

    let context = ToolExecutionContext {
        agent_id: "api_gateway".to_string(),
        user_tier: normalize_tier(&user.tier),
        request_id: Uuid::new_v4().to_string(),
        metadata,
    };

    let execution_request = mr_darkpromth_services::ExecutionRequest {
        id: Uuid::new_v4(),
        tool_name: request.tool_name.clone(),
        input: request.input.clone(),
        context,
        timeout_ms: request.timeout_ms,
        priority: mr_darkpromth_services::ExecutionPriority::Normal,
    };

    match executor.execute(execution_request).await {
        Ok(response) => match response.result {
            Ok(tool_result) => HttpResponse::Ok().json(ToolExecuteResponse {
                request_id: response.request_id.to_string(),
                result: map_tool_result(tool_result),
                execution_time_ms: response.execution_time_ms,
                started_at: response.started_at.to_rfc3339(),
                completed_at: response.completed_at.to_rfc3339(),
            }),
            Err(err) => HttpResponse::BadRequest().json(ErrorResponse {
                error: "Execution Failed".to_string(),
                message: err.to_string(),
            }),
        },
        Err(err) => HttpResponse::BadRequest().json(ErrorResponse {
            error: "Execution Failed".to_string(),
            message: err.to_string(),
        }),
    }
}

#[utoipa::path(
    post,
    path = "/api/sandbox/execute",
    request_body = SandboxExecuteRequest,
    responses(
        (status = 200, description = "Sandbox execution result", body = SandboxExecuteResponse),
        (status = 400, description = "Execution failed", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse)
    ),
    tag = "sandbox",
    security(("bearer_auth" = []))
)]
pub async fn execute_sandbox(
    auth_user: Option<AuthenticatedUser>,
    request: web::Json<SandboxExecuteRequest>,
) -> impl Responder {
    let user = match auth_user {
        Some(user) => user,
        None => {
            return HttpResponse::Unauthorized().json(ErrorResponse {
                error: "Unauthorized".to_string(),
                message: "Authentication required".to_string(),
            })
        }
    };

    if !has_advanced_access(&user.tier) {
        return HttpResponse::Forbidden().json(ErrorResponse {
            error: "Forbidden".to_string(),
            message: "Premium or Ultra tier required for sandbox execution".to_string(),
        });
    }

    let mut config = SandboxConfig::default();
    if let Some(timeout_seconds) = request.timeout_seconds {
        config.max_execution_time = std::time::Duration::from_secs(timeout_seconds.max(1));
    }

    let executor = match SandboxedExecutor::new(config) {
        Ok(executor) => executor,
        Err(err) => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "Sandbox Error".to_string(),
                message: err.to_string(),
            })
        }
    };

    match executor.execute_code(&request.code, &request.language).await {
        Ok(result) => {
            let files_created = result
                .files_created
                .iter()
                .map(|path| path.to_string_lossy().to_string())
                .collect::<Vec<_>>();

            HttpResponse::Ok().json(SandboxExecuteResponse {
                exit_code: result.exit_code,
                stdout: result.stdout,
                stderr: result.stderr,
                execution_time_ms: result.execution_time.as_millis() as u64,
                memory_used: result.memory_used,
                cpu_time_ms: result.cpu_time.as_millis() as u64,
                files_created,
                security_violations: result.security_violations,
            })
        }
        Err(err) => HttpResponse::BadRequest().json(ErrorResponse {
            error: "Sandbox Execution Failed".to_string(),
            message: err.to_string(),
        }),
    }
}

pub fn build_tool_executor(registry: Arc<ToolRegistry>) -> Arc<MasterToolExecutor> {
    Arc::new(MasterToolExecutor::with_registry(
        registry,
        ExecutorConfig::default(),
    ))
}
