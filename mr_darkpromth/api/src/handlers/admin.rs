use axum::{
    extract::{State, Json, Path, ws::{WebSocketUpgrade, WebSocket, Message}},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use bigdecimal::BigDecimal;
use mr_darkpromth_services::LearningSystem;
use tokio::sync::MutexGuard;

use crate::AppState;
use crate::handlers::auth::extract_token;
use crate::error_handler::{ApiError, ApiSuccess};

#[derive(Debug, Serialize)]
pub struct AdminUserInfo {
    pub id: String,
    pub email: String,
    pub username: String,
    pub tier: String,
    pub verified: bool,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserStatusRequest {
    pub status: String,
}

// ==================== Admin Handlers ====================

pub async fn admin_list_users_handler(
    State(state): State<Arc<AppState>>,
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

    // Check if user is admin
    if claims.tier != "admin" {
        return ApiError::new("ADMIN_REQUIRED", "Admin access required").into_response();
    }

    match state.user_service.list_users(100, 0).await {
        Ok(users) => {
            let admin_users: Vec<AdminUserInfo> = users.into_iter().map(|u| AdminUserInfo {
                id: u.id.to_string(),
                email: u.email,
                username: u.username,
                tier: u.tier.to_string(),
                verified: u.is_active,
                created_at: u.created_at.to_string(),
            }).collect();
            
            ApiSuccess::new(serde_json::json!({
                "users": admin_users,
                "total": admin_users.len() as i64
            })).into_response()
        }
        Err(e) => {
            log::error!("Failed to list users: {}", e);
            ApiError::new("LIST_ERROR", "Failed to list users").into_response()
        }
    }
}

pub async fn admin_delete_user_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(user_id): Path<String>,
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

    // Check if user is admin
    if claims.tier != "admin" {
        return ApiError::new("ADMIN_REQUIRED", "Admin access required").into_response();
    }

    let user_id = match Uuid::parse_str(&user_id) {
        Ok(id) => id,
        Err(_) => {
            return ApiError::new("INVALID_UUID", "Invalid user ID format").into_response();
        }
    };

    match state.user_service.delete_user(user_id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            log::error!("Failed to delete user: {}", e);
            ApiError::new("DELETE_ERROR", "Failed to delete user").into_response()
        }
    }
}

pub async fn admin_update_user_status_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(_user_id): Path<String>,
    Json(_req): Json<UpdateUserStatusRequest>,
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

    // Check if user is admin
    if claims.tier != "admin" {
        return ApiError::new("ADMIN_REQUIRED", "Admin access required").into_response();
    }

    // Map status string to boolean is_active
    let is_active = match _req.status.to_lowercase().as_str() {
        "active" => true,
        "suspended" | "banned" | "inactive" => false,
        _ => return ApiError::new("INVALID_STATUS", "Status must be 'active', 'suspended', or 'inactive'").into_response(),
    };

    let user_id = match Uuid::parse_str(&_user_id) {
        Ok(id) => id,
        Err(_) => return ApiError::new("INVALID_UUID", "Invalid user ID format").into_response(),
    };

    let update_req = mr_darkpromth_db::UpdateUserRequest {
        username: None,
        email: None,
        tier: None,
        is_active: Some(is_active),
        language: None,
    };

    match state.user_service.update_user(user_id, update_req).await {
        Ok(Some(user)) => {
            ApiSuccess::new(serde_json::json!({
                "message": "User status updated successfully",
                "user": {
                    "id": user.id,
                    "username": user.username,
                    "email": user.email,
                    "status": if user.is_active { "active" } else { "suspended" }
                }
            })).into_response()
        }
        Ok(None) => ApiError::new("USER_NOT_FOUND", "User not found").into_response(),
        Err(e) => {
            log::error!("Failed to update user status: {}", e);
            ApiError::new("UPDATE_FAILED", "Failed to update user status").into_response()
        }
    }
}

pub async fn admin_metrics_handler(
    State(state): State<Arc<AppState>>,
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

    // Check if user is admin
    if claims.tier != "admin" {
        return ApiError::new("ADMIN_REQUIRED", "Admin access required").into_response();
    }

    // Get actual metrics from services
    let total_users = state.user_service.count_users().await.unwrap_or(0);
    let active_users = state.user_service.count_active_users().await.unwrap_or(0);
    
    // Calculate additional metrics
    let premium_users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE tier IN ('premium', 'ultra')")
        .fetch_one(&state.pool)
        .await
        .unwrap_or(0);
    
    let total_conversations: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM chat_sessions")
        .fetch_one(&state.pool)
        .await
        .unwrap_or(0);
        
    let revenue: BigDecimal = sqlx::query_scalar("SELECT COALESCE(SUM(amount), 0) FROM payments WHERE status = 'verified' AND verified_at >= date_trunc('month', now())")
        .fetch_one(&state.pool)
        .await
        .unwrap_or(BigDecimal::from(0));
    
    ApiSuccess::new(serde_json::json!({
        "total_users": total_users,
        "active_users": active_users,
        "premium_users": premium_users,
        "total_conversations": total_conversations,
        "revenue_this_month": revenue,
    })).into_response()
}

pub async fn admin_generate_docs_handler(
    State(state): State<Arc<AppState>>,
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

    // Check if user is admin
    if claims.tier != "admin" {
        return ApiError::new("ADMIN_REQUIRED", "Admin access required").into_response();
    }

    match state.auto_doc_service.scan_codebase().await {
        Ok(modules) => {
            // Save docs to a folder as well
            let docs_path = std::path::Path::new("./docs_generated");
            let _ = state.auto_doc_service.save_docs(&modules, docs_path).await;
            
            ApiSuccess::new(serde_json::json!({
                "status": "success",
                "modules_scanned": modules.len(),
                "modules": modules
            })).into_response()
        }
        Err(e) => {
            log::error!("Failed to scan codebase: {}", e);
            ApiError::new("DOC_SCAN_ERROR", "Failed to scan codebase").into_response()
        }
    }
}

/// List saved auto-generated documentation
pub async fn admin_list_docs_handler(
    State(state): State<Arc<AppState>>,
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

    // Check if user is admin
    if claims.tier != "admin" {
        return ApiError::new("ADMIN_REQUIRED", "Admin access required").into_response();
    }

    let docs_dir = std::path::Path::new("./docs_generated");
    if !docs_dir.exists() {
        return ApiSuccess::new(serde_json::json!({
            "status": "empty",
            "message": "No documentation generated yet. Use POST /api/admin/docs/generate first.",
            "modules": []
        })).into_response();
    }

    let mut modules = Vec::new();
    match std::fs::read_dir(docs_dir) {
        Ok(entries) => {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "json") {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) {
                            modules.push(value);
                        }
                    }
                }
            }
        }
        Err(e) => {
            log::error!("Failed to read docs directory: {}", e);
            return ApiError::new("READ_ERROR", "Failed to read documentation directory").into_response();
        }
    }

    ApiSuccess::new(serde_json::json!({
        "status": "ok",
        "total": modules.len(),
        "modules": modules
    })).into_response()
}

pub async fn admin_telemetry_ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
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

    // Check if user is admin
    if claims.tier != "admin" {
        return ApiError::new("ADMIN_REQUIRED", "Admin access required").into_response();
    }

    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    let mut receiver = state.telemetry_service.subscribe();

    while let Ok(metrics) = receiver.recv().await {
        let msg = match serde_json::to_string(&metrics) {
            Ok(m) => m,
            Err(_) => continue,
        };

        if socket.send(Message::Text(msg.into())).await.is_err() {
            // client disconnected
            return;
        }
    }
}

// ==================== Telemetry REST Endpoints ====================

#[derive(Debug, serde::Deserialize)]
pub struct TelemetryHistoryQuery {
    #[serde(default = "default_duration")]
    pub duration_secs: u64,
}

fn default_duration() -> u64 { 300 } // 5 minutes default

pub async fn telemetry_metrics_handler(
    State(state): State<Arc<AppState>>,
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

    // Admin-only endpoint
    if claims.tier != "admin" {
        return ApiError::new("ADMIN_REQUIRED", "Admin access required").into_response();
    }

    match state.telemetry_service.get_current_metrics().await {
        Some(metrics) => ApiSuccess::new(serde_json::json!({
            "status": "ok",
            "metrics": metrics
        })).into_response(),
        None => ApiSuccess::new(serde_json::json!({
            "status": "no_data",
            "message": "No metrics collected yet"
        })).into_response(),
    }
}

/// Unified dashboard summary endpoint
pub async fn dashboard_summary_handler(
    State(state): State<Arc<AppState>>,
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

    // Admin-only endpoint
    if claims.tier != "admin" {
        return ApiError::new("ADMIN_REQUIRED", "Admin access required").into_response();
    }

    // Get telemetry dashboard with request metrics
    let telemetry_summary = state.telemetry_service.get_dashboard_summary().await;
    
    // Get learning metrics
    let learning_metrics = {
        let ls: MutexGuard<'_, LearningSystem> = state.learning_system.lock().await;
        ls.get_metrics().clone()
    };
    
    ApiSuccess::new(serde_json::json!({
        "telemetry": telemetry_summary,
        "learning": {
            "total_corrections": learning_metrics.total_corrections,
            "successful_corrections": learning_metrics.successful_corrections,
            "failed_corrections": learning_metrics.failed_corrections,
            "average_confidence": learning_metrics.average_confidence,
            "average_validation_score": learning_metrics.average_validation_score,
            "most_common_errors": learning_metrics.most_common_errors
        },
        "timestamp": chrono::Utc::now()
    })).into_response()
}

pub async fn telemetry_history_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    axum::extract::Query(query): axum::extract::Query<TelemetryHistoryQuery>,
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

    // Admin-only endpoint
    if claims.tier != "admin" {
        return ApiError::new("ADMIN_REQUIRED", "Admin access required").into_response();
    }

    let history = state.telemetry_service.get_history(query.duration_secs).await;
    
    ApiSuccess::new(serde_json::json!({
        "status": "ok",
        "duration_secs": query.duration_secs,
        "count": history.len(),
        "metrics": history
    })).into_response()
}

// ==================== Metrics Handler (Prometheus) ====================

pub async fn metrics_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // Refresh metrics from DB on each scrape for accuracy
    {
        let users_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
            .fetch_one(&state.pool)
            .await
            .unwrap_or(0);
        let conversations_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM chat_sessions")
            .fetch_one(&state.pool)
            .await
            .unwrap_or(0);
        state.metrics.users_total.store(users_count as u64, std::sync::atomic::Ordering::Relaxed);
        state.metrics.conversations_total.store(conversations_count as u64, std::sync::atomic::Ordering::Relaxed);
    }

    // Generate Prometheus-compatible metrics
    let users_total = state.metrics.users_total.load(std::sync::atomic::Ordering::Relaxed);
    let conversations_total = state.metrics.conversations_total.load(std::sync::atomic::Ordering::Relaxed);
    
    // Get request metrics from TelemetryService
    let request_metrics = state.telemetry_service.get_request_metrics();
    
    // Get rate limit rejection metrics
    let (rl_total, rl_auth, rl_chat) = crate::rate_limiting_middleware::get_rate_limit_metrics();

    let metrics_text = format!(
        "# HELP mrdarkpromth_users_total Total number of users\n\
         # TYPE mrdarkpromth_users_total gauge\n\
         mrdarkpromth_users_total {}\n\n\
         # HELP mrdarkpromth_conversations_total Total number of conversations\n\
         # TYPE mrdarkpromth_conversations_total gauge\n\
         mrdarkpromth_conversations_total {}\n\n\
         # HELP mrdarkpromth_api_requests_total Total API requests\n\
         # TYPE mrdarkpromth_api_requests_total counter\n\
         mrdarkpromth_api_requests_total {}\n\n\
         # HELP mrdarkpromth_api_requests_failed_total Total failed API requests\n\
         # TYPE mrdarkpromth_api_requests_failed_total counter\n\
         mrdarkpromth_api_requests_failed_total {}\n\n\
         # HELP mrdarkpromth_api_latency_ms_total Total API latency in milliseconds\n\
         # TYPE mrdarkpromth_api_latency_ms_total counter\n\
         mrdarkpromth_api_latency_ms_total {}\n\n\
         # HELP mrdarkpromth_api_avg_latency_ms Average API latency in milliseconds\n\
         # TYPE mrdarkpromth_api_avg_latency_ms gauge\n\
         mrdarkpromth_api_avg_latency_ms {}\n\n\
         # HELP mrdarkpromth_rate_limit_rejected_total Total rate-limited requests (429)\n\
         # TYPE mrdarkpromth_rate_limit_rejected_total counter\n\
         mrdarkpromth_rate_limit_rejected_total {}\n\n\
         # HELP mrdarkpromth_rate_limit_rejected_auth_total Rate-limited auth requests\n\
         # TYPE mrdarkpromth_rate_limit_rejected_auth_total counter\n\
         mrdarkpromth_rate_limit_rejected_auth_total {}\n\n\
         # HELP mrdarkpromth_rate_limit_rejected_chat_total Rate-limited chat requests\n\
         # TYPE mrdarkpromth_rate_limit_rejected_chat_total counter\n\
         mrdarkpromth_rate_limit_rejected_chat_total {}\n",
        users_total,
        conversations_total,
        request_metrics.total_requests,
        request_metrics.failed_requests,
        request_metrics.total_latency_ms,
        request_metrics.avg_latency_ms,
        rl_total,
        rl_auth,
        rl_chat
    );

    (
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        metrics_text,
    )
}

// ==================== Admin Billing & Verification Handlers ====================

#[derive(Debug, Deserialize)]
pub struct VerifySlipApprovalRequest {
    pub approved: bool,
    pub notes: Option<String>,
}

pub async fn admin_list_pending_verifications_handler(
    State(state): State<Arc<AppState>>, headers: axum::http::HeaderMap
) -> impl IntoResponse {
    let token = match extract_token(&headers) {
        Some(t) => t,
        None => return ApiError::new("MISSING_TOKEN", "Authorization required").into_response(),
    };
    let claims = match state.user_service.validate_token(&token).await {
        Ok(c) => c, Err(_) => return ApiError::new("INVALID_TOKEN", "Invalid token").into_response(),
    };
    if claims.tier != "admin" { return ApiError::new("ADMIN_REQUIRED", "Admin required").into_response(); }
    match state.billing_service.get_pending_verifications().await {
        Ok(verifications) => {
            let verifications_json: Vec<serde_json::Value> = verifications.into_iter().map(|v| {
                serde_json::json!({"id": v.id.to_string(), "payment_id": v.payment_id.to_string(), "user_id": v.user_id.to_string(), "slip_image_path": v.slip_image_path, "status": v.status, "submitted_at": v.submitted_at, "reviewed_by": v.reviewed_by.map(|id| id.to_string()), "reviewed_at": v.reviewed_at, "review_notes": v.review_notes})
            }).collect();
            ApiSuccess::new(serde_json::json!({"verifications": verifications_json, "total": verifications_json.len()})).into_response()
        }
        Err(e) => { log::error!("Failed to get pending verifications: {}", e); ApiError::new("VERIFICATION_LIST_ERROR", "Failed to fetch verifications").into_response() }
    }
}

pub async fn admin_verify_slip_handler(
    State(state): State<Arc<AppState>>, headers: axum::http::HeaderMap, Path(verification_id): Path<String>, Json(req): Json<VerifySlipApprovalRequest>
) -> impl IntoResponse {
    let token = match extract_token(&headers) {
        Some(t) => t, None => return ApiError::new("MISSING_TOKEN", "Authorization required").into_response(),
    };
    let claims = match state.user_service.validate_token(&token).await {
        Ok(c) => c, Err(_) => return ApiError::new("INVALID_TOKEN", "Invalid token").into_response(),
    };
    if claims.tier != "admin" { return ApiError::new("ADMIN_REQUIRED", "Admin required").into_response(); }
    let admin_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id, Err(_) => return ApiError::new("INVALID_ADMIN_ID", "Invalid admin ID").into_response(),
    };
    let verification_uuid = match Uuid::parse_str(&verification_id) {
        Ok(id) => id, Err(_) => return ApiError::new("INVALID_UUID", "Invalid verification ID").into_response(),
    };
    match state.billing_service.verify_payment_slip(verification_uuid, admin_id, req.approved, req.notes.as_deref()).await {
        Ok(result) => {
            let message = if req.approved { "Payment slip approved and user tier upgraded" } else { "Payment slip rejected" };
            ApiSuccess::new(serde_json::json!({"message": message, "verified": result.verified, "payment_id": result.payment_id.to_string(), "amount": result.amount, "reference": result.reference})).into_response()
        }
        Err(e) => { log::error!("Failed to verify payment slip: {}", e); ApiError::new("VERIFICATION_FAILED", &format!("Failed to process: {}", e)).into_response() }
    }
}

pub async fn get_llm_key_status_handler(
    State(state): State<Arc<AppState>>, headers: axum::http::HeaderMap
) -> impl IntoResponse {
    let token = match extract_token(&headers) {
        Some(t) => t, None => return ApiError::new("MISSING_TOKEN", "Authorization required").into_response(),
    };
    let claims = match state.user_service.validate_token(&token).await {
        Ok(c) => c, Err(_) => return ApiError::new("INVALID_TOKEN", "Invalid token").into_response(),
    };
    if claims.tier != "admin" { return ApiError::new("ADMIN_REQUIRED", "Admin required").into_response(); }
    let keys = state.key_pool.get_status().await;
    ApiSuccess::new(keys).into_response()
}

pub async fn admin_get_user_subscription_handler(
    State(state): State<Arc<AppState>>, headers: axum::http::HeaderMap, Path(user_id): Path<String>
) -> impl IntoResponse {
    let token = match extract_token(&headers) {
        Some(t) => t, None => return ApiError::new("MISSING_TOKEN", "Authorization required").into_response(),
    };
    let claims = match state.user_service.validate_token(&token).await {
        Ok(c) => c, Err(_) => return ApiError::new("INVALID_TOKEN", "Invalid token").into_response(),
    };
    if claims.tier != "admin" { return ApiError::new("ADMIN_REQUIRED", "Admin required").into_response(); }
    let user_uuid = match Uuid::parse_str(&user_id) {
        Ok(id) => id, Err(_) => return ApiError::new("INVALID_UUID", "Invalid user ID").into_response(),
    };
    match state.billing_service.get_user_subscription(user_uuid).await {
        Ok(Some(sub)) => ApiSuccess::new(serde_json::json!({"subscription": {"id": sub.id.to_string(), "tier": sub.tier, "status": sub.status, "start_date": sub.start_date, "end_date": sub.end_date, "auto_renew": sub.auto_renew}})).into_response(),
        Ok(None) => ApiSuccess::new(serde_json::json!({"subscription": null})).into_response(),
        Err(e) => { log::error!("Failed to get subscription: {}", e); ApiError::new("SUBSCRIPTION_ERROR", "Failed to fetch subscription").into_response() }
    }
}

pub async fn admin_get_user_payments_handler(
    State(state): State<Arc<AppState>>, headers: axum::http::HeaderMap, Path(user_id): Path<String>
) -> impl IntoResponse {
    let token = match extract_token(&headers) {
        Some(t) => t, None => return ApiError::new("MISSING_TOKEN", "Authorization required").into_response(),
    };
    let claims = match state.user_service.validate_token(&token).await {
        Ok(c) => c, Err(_) => return ApiError::new("INVALID_TOKEN", "Invalid token").into_response(),
    };
    if claims.tier != "admin" { return ApiError::new("ADMIN_REQUIRED", "Admin required").into_response(); }
    let user_uuid = match Uuid::parse_str(&user_id) {
        Ok(id) => id, Err(_) => return ApiError::new("INVALID_UUID", "Invalid user ID").into_response(),
    };
    match state.billing_service.get_payment_history(user_uuid).await {
        Ok(payments) => {
            let payments_json: Vec<serde_json::Value> = payments.into_iter().map(|p| {
                serde_json::json!({"id": p.id.to_string(), "amount": p.amount, "status": p.status, "payment_method": p.payment_method, "reference": p.reference, "created_at": p.created_at, "verified_at": p.verified_at, "verified_by": p.verified_by.map(|id| id.to_string())})
            }).collect();
            ApiSuccess::new(serde_json::json!({"payments": payments_json, "total": payments_json.len()})).into_response()
        }
        Err(e) => { log::error!("Failed to get payments: {}", e); ApiError::new("PAYMENTS_ERROR", "Failed to fetch payments").into_response() }
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserTierRequest {
    pub tier: String,
}

pub async fn admin_update_user_tier_handler(
    State(state): State<Arc<AppState>>, headers: axum::http::HeaderMap, Path(user_id): Path<String>, Json(req): Json<UpdateUserTierRequest>
) -> impl IntoResponse {
    let token = match extract_token(&headers) {
        Some(t) => t, None => return ApiError::new("MISSING_TOKEN", "Authorization required").into_response(),
    };
    let claims = match state.user_service.validate_token(&token).await {
        Ok(c) => c, Err(_) => return ApiError::new("INVALID_TOKEN", "Invalid token").into_response(),
    };
    if claims.tier != "admin" { return ApiError::new("ADMIN_REQUIRED", "Admin required").into_response(); }
    let user_uuid = match Uuid::parse_str(&user_id) {
        Ok(id) => id, Err(_) => return ApiError::new("INVALID_UUID", "Invalid user ID").into_response(),
    };
    let valid_tiers = ["free", "premium", "ultra"];
    if !valid_tiers.contains(&req.tier.as_str()) {
        return ApiError::new("INVALID_TIER", "Tier must be free, premium, or ultra").into_response();
    }
    match state.billing_service.update_user_tier(user_uuid, &req.tier).await {
        Ok(_) => ApiSuccess::new(serde_json::json!({"message": "User tier updated", "tier": req.tier})).into_response(),
        Err(e) => { log::error!("Failed to update tier: {}", e); ApiError::new("UPDATE_ERROR", "Failed to update tier").into_response() }
    }
}
