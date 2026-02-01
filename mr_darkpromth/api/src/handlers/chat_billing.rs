use axum::{
    extract::{State, Json, Path},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

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

pub async fn chat_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(req): Json<ChatRequest>,
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

    let _claims = match state.user_service.validate_token(&token).await {
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

    // TODO: Integrate with Cerebras client for actual chat
    // For now return a placeholder response
    let chat_response = ChatResponse {
        response: format!("Echo: {}", req.message),
        conversation_id,
    };
    
    json_response(chat_response).into_response()
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

    let _claims = match state.user_service.validate_token(&token).await {
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

    let _claims = match state.user_service.validate_token(&token).await {
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

    let _claims = match state.user_service.validate_token(&token).await {
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
