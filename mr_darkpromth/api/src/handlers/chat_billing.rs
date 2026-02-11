use chrono::Utc;
use axum::{
    extract::{State, Json, Path, ConnectInfo},
    response::IntoResponse,
};
use log::warn;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use std::net::SocketAddr;
use uuid::Uuid;
use mr_darkpromth_services::{UserTier, UltraTierRequest, RedisCoordinator};
use crate::AppState;
use crate::handlers::auth::extract_token;
use crate::error_handler::{ApiError, ApiSuccess};
use tokio::sync::MutexGuard;

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

#[axum::debug_handler]
pub async fn chat_handler(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: axum::http::HeaderMap,
    Json(req): Json<ChatRequest>,
) -> axum::response::Response {
    let request_id = Uuid::new_v4();
    let ip_address = Some(addr.ip().to_string());
    let user_agent = headers.get(axum::http::header::USER_AGENT)
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    // Rate Limiting Logic
    if let Some(redis_coordinator) = &state.redis_coordinator {
        let locked_coordinator: MutexGuard<'_, RedisCoordinator> = redis_coordinator.lock().await;
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
                    return ApiError::new(
                        "RATE_LIMITED",
                        "Too many requests. Please try again later.",
                    ).into_response();
                }
            }
        }
    }

    // Auth Check
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

    // Verify or Create Chat Session
    let user_uuid = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return ApiError::new("INVALID_TOKEN", "Invalid user ID").into_response(),
    };

    let user_tier = match claims.tier.as_str() {
        "free" => UserTier::Free,
        "premium" => UserTier::Premium,
        "ultra" => UserTier::Ultra,
        "admin" => UserTier::Admin,
        _ => UserTier::Free,
    };

    // Check daily quota for non-admin users
    if !matches!(user_tier, UserTier::Admin) {
        match state.jailbreak_service.check_quota(user_uuid, user_tier).await {
            Ok(false) => {
                return ApiError::new(
                    "RATE_LIMITED",
                    "Daily message limit reached. Please upgrade your tier.",
                ).into_response();
            }
            Err(e) => warn!("Quota check failed: {}", e),
            _ => {}
        }
    }

    let conversation_uuid = if let Some(cid_str) = &req.conversation_id {
        Uuid::parse_str(cid_str).unwrap_or_else(|_| Uuid::new_v4())
    } else {
        Uuid::new_v4()
    };

    // Check if session exists
    let session_exists = sqlx::query("SELECT id FROM chat_sessions WHERE id = $1 AND user_id = $2")
        .bind(conversation_uuid)
        .bind(user_uuid)
        .fetch_optional(&state.pool)
        .await
        .unwrap_or(None)
        .is_some();

    if !session_exists {
        // Create new session
        if let Err(e) = sqlx::query("INSERT INTO chat_sessions (id, user_id, title, created_at, updated_at) VALUES ($1, $2, $3, NOW(), NOW())")
            .bind(conversation_uuid)
            .bind(user_uuid)
            .bind("New Conversation")
            .execute(&state.pool)
            .await {
            warn!("DB_ERROR: Failed to create chat session: {}", e);
             return ApiError::new("DATABASE_ERROR", "Failed to create chat session").into_response();
        }
    }

    // Insert User Message
    let user_msg_id = Uuid::new_v4();
    if let Err(e) = sqlx::query("INSERT INTO chat_messages (id, session_id, user_id, role, content, model, created_at) VALUES ($1, $2, $3, 'user', $4, $5, NOW())")
        .bind(user_msg_id)
        .bind(conversation_uuid)
        .bind(user_uuid)
        .bind(&req.message)
        .bind("default")
        .execute(&state.pool)
        .await {
        warn!("DB_ERROR: Failed to save user message: {}", e);
        return ApiError::new("DATABASE_ERROR", "Failed to save message").into_response();
    }

    // Audit Log (Async)
    let audit_context = if let Ok(user_id) = Uuid::parse_str(&claims.sub) {
        let user_tier = match claims.tier.as_str() {
            "free" => UserTier::Free,
            "premium" => UserTier::Premium,
            "ultra" => UserTier::Ultra,
            "admin" => UserTier::Admin,
            _ => UserTier::Free,
        };
        Some((user_id, user_tier))
    } else {
        None
    };

    if let Some(&(user_id, user_tier)) = audit_context.as_ref() {
        // ... Log logic ...
        if let Err(err) = state
            .audit_logger
            .log_prompt_request(
                user_id,
                user_tier,
                request_id,
                &req.message,
                false, // jailbreak_enabled
                ip_address.clone(),
                user_agent.clone(),
            )
            .await
        {
            warn!("CHAT_AUDIT: Failed to log prompt request: {}", err);
        }
    }

    // For Ultra Tier users, use the dedicated UltraTierLogic
    if matches!(claims.tier.as_str(), "ultra" | "admin") {
        let ultra_request = UltraTierRequest {
            request_id: request_id.to_string(),
            user_id: claims.sub.clone(),
            user_tier: match claims.tier.as_str() {
                "admin" => UserTier::Admin,
                _ => UserTier::Ultra,
            },
            original_prompt: req.message.clone(),
            selected_jailbreak_prompt: None,
            ai_model: "llama-3.3-70b".to_string(), // Default provider for now
            timestamp: Utc::now(),
            metadata: {
                let mut meta = std::collections::HashMap::new();
                if let Some(ip) = ip_address.clone() {
                    meta.insert("ip_address".to_string(), serde_json::Value::String(ip));
                }
                if let Some(ua) = user_agent.clone() {
                    meta.insert("user_agent".to_string(), serde_json::Value::String(ua));
                }
                meta
            },
        };

        // Solve Send trait and lifetime issues by cloning UltraTierLogic
        // Since tokio::sync::RwLock is now used, we can .await on it
        let ultra_logic = state.ultra_tier_logic.read().await;
        let result = ultra_logic.process_request(ultra_request).await;
        drop(ultra_logic); // Explicitly drop the lock early

        match result {
            Ok(resp) => {
                // Save Assistant Response (Ultra)
                let ai_msg_id = Uuid::new_v4();
                let _ = sqlx::query("INSERT INTO chat_messages (id, session_id, user_id, role, content, model, generation_time_ms, created_at) VALUES ($1, $2, $3, 'assistant', $4, $5, $6, NOW())")
                    .bind(ai_msg_id)
                    .bind(conversation_uuid)
                    .bind(user_uuid)
                    .bind(&resp.ai_response)
                    .bind("llama-3.3-70b")
                    .bind(0i64)
                    .execute(&state.pool)
                    .await;

                let chat_response = ChatResponse {
                    response: resp.ai_response,
                    conversation_id: conversation_uuid.to_string(),
                };
                return ApiSuccess::new(chat_response).into_response();
            }
            Err(e) => {
                warn!("ULTRA_LOGIC_ERROR: Failed to process ultra request: {}", e);
                // Fall back to standard flow if ultra logic fails
            }
        }
    }

    // Apply tier-specific system prompt (Standard Flow for Free/Premium)
    let system_prompt = match claims.tier.as_str() {
        "free" => "You are a helpful AI assistant. You have limited access to advanced features.",
        "premium" => "You are an advanced AI assistant with enhanced capabilities including file access and tool usage.",
        _ => "You are a helpful AI assistant.",
    };

    let start_time = Instant::now();

    // standard tier key rotation logic
    let mut response_text = String::new();
    let mut success = false;
    
    // 1. Try Cerebras First
    if let Some(api_key) = state.key_pool.get_best_key(mr_darkpromth_services::key_pool::Provider::Cerebras).await {
         let client = mr_darkpromth_services::CerebrasClient::with_api_key(api_key.key);
         match client.chat_completion_with_system(system_prompt, &req.message).await {
             Ok(resp) => {
                 response_text = resp;
                 success = true;
             },
             Err(e) => {
                 warn!("Cerebras standard chat failed for key {}: {}", api_key.id, e);
                 state.key_pool.report_failure(&api_key.id).await;
             }
         }
    }
    
    // 2. Failover to OpenRouter if Cerebras failed or no key available
    if !success {
        if let Some(api_key) = state.key_pool.get_best_key(mr_darkpromth_services::key_pool::Provider::OpenRouter).await {
             let client = mr_darkpromth_services::OpenRouterClient::new(api_key.key);
             // Use a cheap/fast model for standard tier failover
             match client.chat_completion(&req.message, Some("openai/gpt-4o-mini")).await {
                 Ok(resp) => {
                     response_text = resp;
                     success = true;
                     warn!("Standard chat failed over to OpenRouter");
                 },
                 Err(e) => {
                     warn!("OpenRouter failover failed: {}", e);
                     state.key_pool.report_failure(&api_key.id).await;
                 }
             }
        }
    }

    if success {
        if let Some(&(user_id, user_tier)) = audit_context.as_ref() {
            let processing_time_ms = start_time.elapsed().as_millis() as u64;
            if let Err(err) = state
                .audit_logger
                .log_prompt_response(
                    user_id,
                    user_tier,
                    request_id,
                    processing_time_ms,
                    0,
                    "llama-3.3-70b",
                )
                .await
            {
                warn!("CHAT_AUDIT: Failed to log prompt response: {}", err);
            }
        }

        // Save Assistant Response (Standard)
        let processing_time_ms = start_time.elapsed().as_millis() as i64; // Cast to i64 for DB
        let ai_msg_id = Uuid::new_v4();
        if let Err(e) = sqlx::query("INSERT INTO chat_messages (id, session_id, user_id, role, content, model, generation_time_ms, created_at) VALUES ($1, $2, $3, 'assistant', $4, $5, $6, NOW())")
            .bind(ai_msg_id)
            .bind(conversation_uuid)
            .bind(user_uuid)
            .bind(&response_text)
            .bind("llama-3.3-70b")
            .bind(processing_time_ms)
            .execute(&state.pool)
            .await {
            warn!("DB_ERROR: Failed to save assistant message: {}", e);
        }

        let chat_response = ChatResponse {
            response: response_text,
            conversation_id: conversation_uuid.to_string(),
        };
        ApiSuccess::new(chat_response).into_response()
    } else {
        ApiError::new(
            "EXTERNAL_SERVICE_ERROR",
            "Failed to get response from AI services (All providers failed)",
        ).into_response()
    }
}

// ==================== Billing Handlers ====================

pub async fn list_plans_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match state.billing_service.get_plans().await {
        Ok(plans) => ApiSuccess::new(plans).into_response(),
        Err(e) => ApiError::new(
            "INTERNAL_ERROR",
            format!("Failed to get plans: {}", e),
        ).into_response(),
    }
}

pub async fn get_plan_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.billing_service.get_plan(id).await {
        Ok(plan) => ApiSuccess::new(plan).into_response(),
        Err(e) => ApiError::new(
            "NOT_FOUND",
            format!("Failed to get plan: {}", e),
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
            return ApiError::new("MISSING_TOKEN", "Authorization header required").into_response();
        }
    };

    let claims = match state.user_service.validate_token(&token).await {
        Ok(c) => c,
        Err(_) => {
            return ApiError::new("INVALID_TOKEN", "Invalid or expired token").into_response();
        }
    };

    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return ApiError::new("INVALID_USER_ID", "Invalid user ID in token").into_response();
        }
    };

    let plan_id = match Uuid::parse_str(&req.plan_id) {
        Ok(id) => id,
        Err(_) => {
            return ApiError::new("INVALID_INPUT", "Invalid plan ID format").into_response();
        }
    };

    match state.billing_service.generate_qr_code(user_id, plan_id).await {
        Ok(qr_data) => {
            let response = QrResponse {
                qr_code: qr_data.qr_code,
                reference: qr_data.reference,
            };
            ApiSuccess::new(response).into_response()
        }
        Err(e) => ApiError::new(
            "INTERNAL_ERROR",
            format!("Failed to generate QR code: {}", e),
        ).into_response(),
    }
}

pub async fn verify_slip_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(req): Json<VerifySlipRequest>,
) -> impl IntoResponse {
    // 1. Auth Check
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

    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return ApiError::new("INVALID_USER_ID", "Invalid user ID in token").into_response();
        }
    };

    // 2. Validate reference format and extract payment_id
    if !req.reference.starts_with("PAY-") {
        return ApiError::new("INVALID_INPUT", "Invalid payment reference format").into_response();
    }

    // 3. Get pending payment by reference
    match state.billing_service.get_payment_by_reference(&req.reference).await {
        Ok(Some(payment)) => {
            // Verify user owns this payment
            if payment.user_id != user_id {
                return ApiError::new("PERMISSION_DENIED", "This payment does not belong to you").into_response();
            }

            // 4. Verify the payment slip
            match state.billing_service.verify_user_payment_slip(
                &payment.id.to_string(),
                payment.amount,
                &req.reference,
            ).await {
                Ok(result) => {
                    let response = serde_json::json!({
                        "verified": result.verified,
                        "payment_id": result.payment_id.to_string(),
                        "amount": result.amount,
                        "reference": result.reference,
                        "message": if result.verified { "Payment verified successfully! Your tier has been upgraded." } else { "Verification failed" }
                    });
                    ApiSuccess::new(response).into_response()
                }
                Err(e) => ApiError::new(
                    "VERIFICATION_ERROR",
                    format!("Verification failed: {}", e),
                ).into_response(),
            }
        }
        Ok(None) => ApiError::new("NOT_FOUND", "No payment found with this reference").into_response(),
        Err(e) => ApiError::new(
            "DATABASE_ERROR",
            format!("Failed to lookup payment: {}", e),
        ).into_response(),
    }
}

pub async fn get_subscription_handler(
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

    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return ApiError::new("INVALID_USER_ID", "Invalid user ID in token").into_response();
        }
    };

    match state.billing_service.get_user_subscription(user_id).await {
        Ok(Some(sub)) => {
            let response = serde_json::json!({
                "tier": sub.tier,
                "expires_at": sub.end_date,
                "status": sub.status,
            });
            ApiSuccess::new(response).into_response()
        }
        Ok(None) => {
            let response = serde_json::json!({
                "tier": "free",
                "expires_at": null,
                "status": "inactive",
            });
            ApiSuccess::new(response).into_response()
        }
        Err(e) => ApiError::new(
            "DATABASE_ERROR",
            format!("Failed to get subscription: {}", e),
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
            return ApiError::new("MISSING_TOKEN", "Authorization header required").into_response();
        }
    };

    let claims = match state.user_service.validate_token(&token).await {
        Ok(c) => c,
        Err(_) => {
            return ApiError::new("INVALID_TOKEN", "Invalid or expired token").into_response();
        }
    };

    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return ApiError::new("INVALID_USER_ID", "Invalid user ID in token").into_response();
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
            ApiSuccess::new(payments_json).into_response()
        }
        Err(e) => ApiError::new(
            "DATABASE_ERROR",
            format!("Failed to get payment history: {}", e),
        ).into_response(),
    }
}

pub async fn get_chat_history_handler(
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

    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return ApiError::new("INVALID_USER_ID", "Invalid user ID in token").into_response();
        }
    };

    // Query chat sessions with last message and count
    let query = r#"
        SELECT 
            s.id, 
            s.title, 
            s.created_at, 
            COUNT(m.id) as message_count,
            (SELECT content FROM chat_messages WHERE session_id = s.id ORDER BY created_at DESC LIMIT 1) as last_message,
            (SELECT created_at FROM chat_messages WHERE session_id = s.id ORDER BY created_at DESC LIMIT 1) as last_message_timestamp
        FROM chat_sessions s
        LEFT JOIN chat_messages m ON s.id = m.session_id
        WHERE s.user_id = $1
        GROUP BY s.id
        ORDER BY s.updated_at DESC
        LIMIT 20
    "#;

    match sqlx::query_as::<_, (Uuid, String, chrono::DateTime<Utc>, i64, Option<String>, Option<chrono::DateTime<Utc>>)>(query)
        .bind(user_id)
        .fetch_all(&state.pool)
        .await 
    {
        Ok(sessions) => {
            let sessions_json: Vec<serde_json::Value> = sessions.into_iter().map(|(id, title, created_at, count, last_msg, last_ts)| {
                serde_json::json!({
                    "id": id.to_string(),
                    "title": title,
                    "created_at": created_at,
                    "message_count": count,
                    "last_message": last_msg.unwrap_or_default(),
                    "last_message_timestamp": last_ts.unwrap_or(created_at),
                })
            }).collect();

            let response = serde_json::json!({
                "conversations": sessions_json,
                "total": sessions_json.len(),
                "has_more": false
            });
            ApiSuccess::new(response).into_response()
        }
        Err(e) => {
             warn!("DB_ERROR: Failed to get chat history: {}", e);
             ApiError::new("DATABASE_ERROR", "Failed to retrieve chat history").into_response()
        }
    }
}
