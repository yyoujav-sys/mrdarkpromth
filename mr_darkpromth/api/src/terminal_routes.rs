use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::auth_middleware::AuthenticatedUser;
use crate::routes::ErrorResponse;
use mr_darkpromth_services::sandboxed_execution::SandboxedExecutor;
use std::sync::Arc;

#[derive(Debug, Deserialize, ToSchema)]
pub struct TerminalExecuteRequest {
    pub command: String,
    pub args: Option<Vec<String>>,
    pub working_dir: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TerminalResponse {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub execution_time_ms: u64,
}

#[utoipa::path(
    post,
    path = "/api/terminal/execute",
    request_body = TerminalExecuteRequest,
    responses(
        (status = 200, description = "Command executed", body = TerminalResponse),
        (status = 403, description = "Forbidden - Ultra tier required", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    tag = "terminal",
    security(("bearer_auth" = []))
)]
pub async fn execute_terminal(
    auth_user: AuthenticatedUser,
    request: web::Json<TerminalExecuteRequest>,
    sandbox_executor: web::Data<Arc<SandboxedExecutor>>,
) -> impl Responder {
    // ✅ Ultra Tier เท่านั้นที่รัน terminal ได้
    if auth_user.tier != "ultra" {
        return HttpResponse::Forbidden().json(ErrorResponse {
            error: "Forbidden".to_string(),
            message: "Terminal execution requires Ultra tier".to_string(),
        });
    }

    let args = request.args.clone().unwrap_or_default();
    let args_refs: Vec<&str> = args.iter().map(|arg| arg.as_str()).collect();
    let result = sandbox_executor.execute_command(
        &request.command,
        &args_refs,
    ).await;

    match result {
        Ok(exec_result) => HttpResponse::Ok().json(TerminalResponse {
            exit_code: exec_result.exit_code,
            stdout: exec_result.stdout,
            stderr: exec_result.stderr,
            execution_time_ms: exec_result.execution_time.as_millis() as u64,
        }),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse {
            error: "Execution failed".to_string(),
            message: e.to_string(),
        }),
    }
}
