use axum::{
    extract::{State, Json, Path},
    http::StatusCode,
    response::IntoResponse,
};
use log::warn;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;
use mr_darkpromth_services::UserTier;

use crate::AppState;
use crate::handlers::auth::{json_response, error_response, extract_token};

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub message: String,
    pub conversation_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub response: String,
    pub conversation_id: String,
}

#[derive(Debug, Deserialize)]
pub struct GenerateQrRequest {
    pub plan_id: String,
}

#[derive(Debug, Serialize)]
pub struct QrResponse {
    pub qr_code: String,
    pub reference: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifySlipRequest {
    pub reference: String,
    pub slip_image: String,
}

// ==================== Chat Handler ====================

use axum::extract::ConnectInfo;
use std::net::SocketAddr;

pub async fn chat_handler(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: axum::http::HeaderMap,
    Json(req): Json<ChatRequest>,
) -> impl IntoResponse {
    // Rate Limiting Logic
    if let Some(redis_coordinator) = &state.redis_coordinator {
        if let Ok(mut locked_coordinator) = redis_coordinator.lock() {
            if let Ok(mut conn) = locked_coordinator.get_connection() {
                let ip = addr.ip().to_string();
                let key = format!("rate_limit:{}", ip);
                const MAX_REQUESTS: i64 = 30; // 30 requests
                const WINDOW_SECS: usize = 60; // per 60 seconds

                let result: redis::RedisResult<i64> = redis::cmd("INCR").arg(&key).query(&mut conn);

                if let Ok(count) = result {
                    if count == 1 {
                        let _: redis::RedisResult<()> = redis::cmd("EXPIRE").arg(&key).arg(WINDOW_SECS).query(&mut conn);
                    }
                    if count > MAX_REQUESTS {
                        return error_response(
                            StatusCode::TOO_MANY_REQUESTS,
                            "RATE_LIMIT_EXCEEDED",
                            "You have made too many requests. Please try again later.",
                        ).into_response();
                    }
                } else {
                    warn!("RATE_LIMIT: Redis INCR failed. Allowing request.");
                }
            } else {
                warn!("RATE_LIMIT: Failed to get Redis connection. Allowing request.");
            }
        } else {
            warn!("RATE_LIMIT: Redis coordinator lock was poisoned. Allowing request.");
        }
    }

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

    // Generate conversation ID if not provided
    let conversation_id = req.conversation_id.unwrap_or_else(|| {
        Uuid::new_v4().to_string()
    });

    let request_id = Uuid::new_v4();
    let request_id_for_response = request_id.clone();
    let ip_address = headers
        .get("x-real-ip")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string())
        .or_else(|| {
            headers
                .get("x-forwarded-for")
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.split(',').next())
                .map(|value| value.trim().to_string())
        });
    let user_agent = headers
        .get(axum::http::header::USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string());
    let audit_context = match (
        Uuid::parse_str(&claims.sub),
        UserTier::from_str(&claims.tier),
    ) {
        (Ok(user_id), Ok(user_tier)) => Some((user_id, user_tier)),
        _ => {
            warn!("CHAT_AUDIT: Unable to parse user_id or tier for audit logging");
            None
        }
    };
    if let Some((user_id, user_tier)) = audit_context.as_ref() {
        if let Err(err) = state
            .audit_logger
            .log_prompt_request(
                user_id.clone(),
                *user_tier,
                request_id,
                &req.message,
                matches!(*user_tier, UserTier::Ultra),
                ip_address.clone(),
                user_agent.clone(),
            )
            .await
        {
            warn!("CHAT_AUDIT: Failed to log prompt request: {}", err);
        }
    }

    // Apply tier-specific system prompt
    let system_prompt = match claims.tier.as_str() {
        "free" => "You are a helpful AI assistant. You have limited access to advanced features.",
        "premium" => "You are an advanced AI assistant with enhanced capabilities including file access and tool usage.",
        "ultra" => "You are an unrestricted AI assistant with full system access, terminal commands, and sandboxed execution capabilities.",
        _ => "You are a helpful AI assistant.",
    };

    let start_time = Instant::now();

    // Call Cerebras API
    match state.cerebras_client.chat_completion_with_system(system_prompt, &req.message).await {
        Ok(response_text) => {
            if let Some((user_id, user_tier)) = audit_context.as_ref() {
                let processing_time_ms = start_time.elapsed().as_millis() as u64;
                if let Err(err) = state
                    .audit_logger
                    .log_prompt_response(
                        user_id.clone(),
                        *user_tier,
                        request_id_for_response,
                        processing_time_ms,
                        0,
                        "cerebras",
                    )
                    .await
                {
                    warn!("CHAT_AUDIT: Failed to log prompt response: {}", err);
                }
            }
            let chat_response = ChatResponse {
                response: response_text,
                conversation_id,
            };
            json_response(chat_response).into_response()
        }
        Err(e) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "CHAT_ERROR",
            &format!("Failed to get response from AI: {}", e),
        ).into_response(),
    }
}

// ==================== Billing Handlers ====================

pub async fn list_plans_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match state.billing_service.get_plans().await {
        Ok(plans) => {
            let plans_json: Vec<serde_json::Value> = plans.into_iter().map(|p| {
                serde_json::json!({
                    "id": p.id.to_string(),
                    "name": p.name,
                    "price": p.price,
                    "features": p.features,
                })
            }).collect();
            json_response(plans_json).into_response()
        }
        Err(e) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "PLANS_ERROR",
            &format!("Failed to get plans: {}", e),
        ).into_response(),
    }
}

pub async fn get_plan_handler(
    State(state): State<Arc<AppState>>,
    Path(plan_id): Path<String>,
) -> impl IntoResponse {
    let plan_id = match Uuid::parse_str(&plan_id) {
        Ok(id) => id,
        Err(_) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                "INVALID_UUID",
                "Invalid plan ID format",
            ).into_response();
        }
    };

    match state.billing_service.get_plan(plan_id).await {
        Ok(plan) => {
            let plan_json = serde_json::json!({
                "id": plan.id.to_string(),
                "name": plan.name,
                "price": plan.price,
                "features": plan.features,
            });
            json_response(plan_json).into_response()
        }
        Err(e) => error_response(
            StatusCode::NOT_FOUND,
            "PLAN_NOT_FOUND",
            &format!("Plan not found: {}", e),
        ).into_response(),
    }
}

pub async fn generate_qr_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(req): Json<GenerateQrRequest>,
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

    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INVALID_USER_ID",
                "Invalid user ID in token",
            ).into_response();
        }
    };

    let plan_id = match Uuid::parse_str(&req.plan_id) {
        Ok(id) => id,
        Err(_) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                "INVALID_PLAN_ID",
                "Invalid plan ID format",
            ).into_response();
        }
    };

    match state.billing_service.generate_qr_code(user_id, plan_id).await {
        Ok(qr_data) => {
            let response = QrResponse {
                qr_code: qr_data.qr_code,
                reference: qr_data.reference,
            };
            json_response(response).into_response()
        }
        Err(e) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "QR_ERROR",
            &format!("Failed to generate QR code: {}", e),
        ).into_response(),
    }
}

pub async fn verify_slip_handler(
    State(_state): State<Arc<AppState>>,
    _headers: axum::http::HeaderMap,
    Json(_req): Json<VerifySlipRequest>,
) -> impl IntoResponse {
    // TODO: Implement slip verification
    json_response(serde_json::json!({ "verified": false, "message": "Not implemented" })).into_response()
}

pub async fn get_subscription_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
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

    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INVALID_USER_ID",
                "Invalid user ID in token",
            ).into_response();
        }
    };

    match state.billing_service.get_user_subscription(user_id).await {
        Ok(Some(sub)) => {
            let response = serde_json::json!({
                "tier": sub.tier,
                "expires_at": sub.end_date,
                "status": sub.status,
            });
            json_response(response).into_response()
        }
        Ok(None) => {
            let response = serde_json::json!({
                "tier": "free",
                "expires_at": null,
                "status": "inactive",
            });
            json_response(response).into_response()
        }
        Err(e) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "SUBSCRIPTION_ERROR",
            &format!("Failed to get subscription: {}", e),
        ).into_response(),
    }
}

pub async fn get_payment_history_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
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

    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INVALID_USER_ID",
                "Invalid user ID in token",
            ).into_response();
        }
    };

    match state.billing_service.get_payment_history(user_id).await {
        Ok(payments) => {
            let payments_json: Vec<serde_json::Value> = payments.into_iter().map(|p| {
                serde_json::json!({
                    "id": p.id.to_string(),
                    "amount": p.amount,
                    "status": p.status,
                    "date": p.created_at,
                    "reference": p.reference,
                })
            }).collect();
            json_response(payments_json).into_response()
        }
        Err(e) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "HISTORY_ERROR",
            &format!("Failed to get payment history: {}", e),
        ).into_response(),
    }
}
