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

    // Check if user is admin
    if claims.tier != "admin" {
        return error_response(
            StatusCode::FORBIDDEN,
            "ADMIN_REQUIRED",
            "Admin access required",
        ).into_response();
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
            
            json_response(serde_json::json!({
                "users": admin_users,
                "total": admin_users.len() as i64
            })).into_response()
        }
        Err(e) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "LIST_ERROR",
            &format!("Failed to list users: {}", e),
        ).into_response(),
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

    // Check if user is admin
    if claims.tier != "admin" {
        return error_response(
            StatusCode::FORBIDDEN,
            "ADMIN_REQUIRED",
            "Admin access required",
        ).into_response();
    }

    let user_id = match Uuid::parse_str(&user_id) {
        Ok(id) => id,
        Err(_) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                "INVALID_UUID",
                "Invalid user ID format",
            ).into_response();
        }
    };

    match state.user_service.delete_user(user_id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "DELETE_ERROR",
            &format!("Failed to delete user: {}", e),
        ).into_response(),
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

    // Check if user is admin
    if claims.tier != "admin" {
        return error_response(
            StatusCode::FORBIDDEN,
            "ADMIN_REQUIRED",
            "Admin access required",
        ).into_response();
    }

    // TODO: Implement user status update
    json_response(serde_json::json!({ "message": "Not implemented" })).into_response()
}

pub async fn admin_metrics_handler(
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

    // Check if user is admin
    if claims.tier != "admin" {
        return error_response(
            StatusCode::FORBIDDEN,
            "ADMIN_REQUIRED",
            "Admin access required",
        ).into_response();
    }

    // Get actual metrics from services
    let total_users = state.user_service.count_users().await.unwrap_or(0);
    let active_users = state.user_service.count_active_users().await.unwrap_or(0);
    
    json_response(serde_json::json!({
        "total_users": total_users,
        "active_users": active_users,
        "premium_users": 0,
        "total_conversations": 0,
        "revenue_this_month": 0.0,
    })).into_response()
}

// ==================== Metrics Handler (Prometheus) ====================

pub async fn metrics_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // Generate Prometheus-compatible metrics
    let users_total = state.metrics.users_total.load(std::sync::atomic::Ordering::Relaxed);
    let conversations_total = state.metrics.conversations_total.load(std::sync::atomic::Ordering::Relaxed);
    let requests_total = state.metrics.requests_total.load(std::sync::atomic::Ordering::Relaxed);
    
    let metrics_text = format!(
        "# HELP mrdarkpromth_users_total Total number of users\n\
         # TYPE mrdarkpromth_users_total gauge\n\
         mrdarkpromth_users_total {}\n\n\
         # HELP mrdarkpromth_conversations_total Total number of conversations\n\
         # TYPE mrdarkpromth_conversations_total gauge\n\
         mrdarkpromth_conversations_total {}\n\n\
         # HELP mrdarkpromth_api_requests_total Total API requests\n\
         # TYPE mrdarkpromth_api_requests_total counter\n\
         mrdarkpromth_api_requests_total {}\n",
        users_total,
        conversations_total,
        requests_total,
    );

    (
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        metrics_text,
    )
}
