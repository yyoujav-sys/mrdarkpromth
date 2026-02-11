use axum::{
    extract::{State, Json, WebSocketUpgrade},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::AppState;
use crate::handlers::auth::extract_token;
use crate::error_handler::{ApiError, ApiSuccess};
use crate::i18n_utils::{Language, get_message};

/// Request for Ultra Terminal execution with elevated privileges
#[derive(Debug, Deserialize)]
pub struct UltraTerminalExecuteRequest {
    pub command: String,
    pub args: Option<Vec<String>>,
    pub working_dir: Option<String>,
    pub session_id: Option<String>,
    pub timeout_seconds: Option<u64>,
    pub stream_output: Option<bool>,
}

/// Response for Ultra Terminal execution
#[derive(Debug, Serialize)]
pub struct UltraTerminalExecuteResponse {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub execution_time_ms: u64,
    pub session_id: String,
    pub resource_usage: ResourceUsage,
    pub audit_log_id: String,
    pub security_checks_passed: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ResourceUsage {
    pub memory_mb: u64,
    pub cpu_percent: f64,
    pub processes_count: u32,
}

/// WebSocket message types for terminal streaming
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TerminalWsMessage {
    #[serde(rename = "input")]
    Input { data: String },
    #[serde(rename = "output")]
    Output { data: String, stream: String },
    #[serde(rename = "error")]
    Error { message: String },
    #[serde(rename = "status")]
    Status { status: String, exit_code: Option<i32> },
    #[serde(rename = "resource_update")]
    ResourceUpdate { memory_mb: u64, cpu_percent: f64 },
}

/// Handler for Ultra Terminal execution (Ultra Tier only)
pub async fn execute_ultra_terminal_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(req): Json<UltraTerminalExecuteRequest>,
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

    // Ultra Tier only check
    if claims.tier != "ultra" && claims.tier != "admin" {
        return ApiError::new("ULTRA_TIER_REQUIRED", "Ultra Terminal requires Ultra tier or Admin privileges").into_response();
    }

    // Get user language preference
    let lang_pref = Language::from_headers(&headers);

    let user_id = Uuid::parse_str(&claims.sub).unwrap_or_default();
    let session_id = req.session_id.unwrap_or_else(|| format!("ultra-terminal-{}", user_id));
    let timeout = req.timeout_seconds.unwrap_or(300); // 5 min default for Ultra

    // Create audit log entry
    let audit_id = Uuid::new_v4().to_string();
    
    // Execute with elevated privileges in root sandbox
    let executor = match state.sandbox_manager.get_or_create_ultra_session(
        &session_id,
        user_id,
        timeout,
    ).await {
        Ok(e) => e,
        Err(e) => {
            let error_msg = format!("{}: {}", get_message(&lang_pref, "sandbox_init_failed"), e);
            return ApiError::new("SANDBOX_ERROR", &error_msg).into_response();
        }
    };

    let args = req.args.unwrap_or_default();
    let args_refs: Vec<&str> = args.iter().map(|arg| arg.as_str()).collect();
    
    // Execute command with behavioral monitoring
    let result = executor.execute_command_ultra(
        &req.command,
        &args_refs,
        req.working_dir.as_deref(),
        &audit_id,
    ).await;

    match result {
        Ok(exec_result) => {
            ApiSuccess::new(serde_json::json!({
                "stdout": exec_result.stdout,
                "stderr": exec_result.stderr,
                "exit_code": exec_result.exit_code,
                "execution_time_ms": exec_result.execution_time.as_millis() as u64,
                "session_id": session_id,
                "resource_usage": {
                    "memory_mb": exec_result.memory_used / (1024 * 1024),
                    "cpu_percent": exec_result.cpu_time.as_secs_f64() * 100.0,
                    "processes_count": exec_result.processes_created,
                },
                "audit_log_id": audit_id,
                "security_checks_passed": if exec_result.security_violations.is_empty() {
                    vec!["host_protection".to_string(), "resource_limits".to_string(), "network_isolation".to_string()]
                } else {
                    Vec::new()
                },
            })).into_response()
        }
        Err(e) => {
            let error_msg = format!("{}: {}", get_message(&lang_pref, "execution_failed"), e);
            ApiError::new("EXECUTION_ERROR", &error_msg).into_response()
        }
    }
}

/// WebSocket handler for real-time Ultra Terminal streaming
pub async fn ultra_terminal_ws_handler(
    State(state): State<Arc<AppState>>,
    ws: WebSocketUpgrade,
    headers: axum::http::HeaderMap,
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

    // Ultra Tier only check
    if claims.tier != "ultra" && claims.tier != "admin" {
        return ApiError::new("ULTRA_TIER_REQUIRED", "Ultra Terminal WebSocket requires Ultra tier or Admin privileges").into_response();
    }

    ws.on_upgrade(move |socket| {
        handle_ultra_terminal_socket(socket, state, claims.sub)
    })
}

async fn handle_ultra_terminal_socket(
    socket: axum::extract::ws::WebSocket,
    state: Arc<AppState>,
    user_id: String,
) {
    use axum::extract::ws::Message;
    use futures::{sink::SinkExt, stream::StreamExt};
    use tokio::sync::Mutex;

    let (sender, mut receiver) = socket.split();
    let sender = Arc::new(Mutex::new(sender));

    // Send welcome message
    let welcome = serde_json::json!({
        "type": "status",
        "status": "connected",
        "message": "Strategic Link Established"
    });
    let _ = sender.lock().await.send(Message::Text(welcome.to_string().into())).await;

    while let Some(Ok(message)) = receiver.next().await {
        match message {
            Message::Text(text) => {
                if let Ok(req) = serde_json::from_str::<UltraTerminalExecuteRequest>(&text) {
                    let cmd_args = req.args.clone().unwrap_or_default();
                    let args_refs: Vec<&str> = cmd_args.iter().map(|arg| arg.as_str()).collect();
                    let audit_id = Uuid::new_v4().to_string();
                    let user_id_uuid = Uuid::parse_str(&user_id).unwrap_or_default();
                    let session_id = req.session_id.clone().unwrap_or_else(|| format!("ultra-terminal-{}", user_id));
                    
                    let executor_res = state.sandbox_manager.get_or_create_ultra_session(
                        &session_id,
                        user_id_uuid,
                        req.timeout_seconds.unwrap_or(3600),
                    ).await;

                    match executor_res {
                        Ok(executor) => {
                            match executor.execute_command_ultra_stream(
                                &req.command,
                                &args_refs,
                                req.working_dir.as_deref(),
                                &audit_id,
                            ).await {
                                Ok((_child, stdout, stderr)) => {
                                    use tokio::io::AsyncReadExt;
                                    
                                    let sender_stdout = Arc::clone(&sender);
                                    tokio::spawn(async move {
                                        let mut reader = stdout;
                                        let mut buffer = [0u8; 1024];
                                        while let Ok(n) = reader.read(&mut buffer).await {
                                            if n == 0 { break; }
                                            let data = String::from_utf8_lossy(&buffer[..n]).to_string();
                                            let _ = sender_stdout.lock().await.send(Message::Text(serde_json::json!({
                                                "type": "output",
                                                "stream": "stdout",
                                                "data": data
                                            }).to_string().into())).await;
                                        }
                                    });

                                    let sender_stderr = Arc::clone(&sender);
                                    tokio::spawn(async move {
                                        let mut reader = stderr;
                                        let mut buffer = [0u8; 1024];
                                        while let Ok(n) = reader.read(&mut buffer).await {
                                            if n == 0 { break; }
                                            let data = String::from_utf8_lossy(&buffer[..n]).to_string();
                                            let _ = sender_stderr.lock().await.send(Message::Text(serde_json::json!({
                                                "type": "output",
                                                "stream": "stderr",
                                                "data": data
                                            }).to_string().into())).await;
                                        }
                                    });
                                }
                                Err(e) => {
                                    let _ = sender.lock().await.send(Message::Text(serde_json::json!({
                                        "type": "error",
                                        "message": format!("Operational conflict: {}", e)
                                    }).to_string().into())).await;
                                }
                            }
                        }
                        Err(e) => {
                            let _ = sender.lock().await.send(Message::Text(serde_json::json!({
                                "type": "error",
                                "message": format!("Activation failed: {}", e)
                            }).to_string().into())).await;
                        }
                    }
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }
}

/// Handler to get Ultra Terminal session status
pub async fn get_ultra_terminal_session_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    axum::extract::Path(session_id): axum::extract::Path<String>,
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

    if claims.tier != "ultra" && claims.tier != "admin" {
        return ApiError::new("ULTRA_TIER_REQUIRED", "Ultra tier required").into_response();
    }

    // Return session status
    ApiSuccess::new(serde_json::json!({
        "session_id": session_id,
        "status": "active",
        "user_id": claims.sub,
        "tier": claims.tier,
        "features": ["root_access", "privileged_commands", "network_isolated", "resource_monitored"],
    })).into_response()
}
