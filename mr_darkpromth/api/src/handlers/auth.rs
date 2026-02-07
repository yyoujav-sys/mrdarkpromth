use axum::{
    extract::{State, Json, Path},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::AppState;

// ==================== Request/Response Types ====================

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub username: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub username: String,
    pub tier: String,
    pub verified: bool,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyEmailRequest {
    pub token: String,
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct ResendVerificationRequest {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct PasswordResetRequest {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub username: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

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

// ==================== Helper Functions ====================

pub fn json_response<T: Serialize>(data: T) -> impl IntoResponse {
    (StatusCode::OK, Json(data))
}

pub fn error_response(status: StatusCode, error: &str, message: &str) -> impl IntoResponse {
    (status, Json(ErrorResponse {
        error: error.to_string(),
        message: message.to_string(),
    }))
}

pub fn extract_token(headers: &axum::http::HeaderMap) -> Option<String> {
    headers.get("authorization")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.replace("Bearer ", ""))
}

// ==================== Basic Handlers ====================

pub async fn root_handler() -> &'static str {
    "MR.DarkPromth API - Visit /health for health check"
}

pub async fn health_handler() -> &'static str {
    "OK"
}

// ==================== Authentication Handlers ====================

pub async fn register_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> impl IntoResponse {
    let register_req = mr_darkpromth_services::user_service::RegisterRequest {
        username: req.username,
        email: req.email,
        password: req.password,
    };

    match state.user_service.register(register_req).await {
        Ok(auth_response) => {
            let response = AuthResponse {
                token: auth_response.token,
                user: UserResponse {
                    id: auth_response.user.id.to_string(),
                    email: auth_response.user.email,
                    username: auth_response.user.username,
                    tier: auth_response.user.tier.to_string(),
                    verified: auth_response.user.is_active,
                },
            };
            json_response(response).into_response()
        }
        Err(e) => error_response(
            StatusCode::BAD_REQUEST,
            "REGISTRATION_FAILED",
            &e.to_string(),
        ).into_response(),
    }
}

pub async fn login_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> impl IntoResponse {
    log::info!("Attempting login for email: {}", req.email);
    let login_req = mr_darkpromth_services::user_service::LoginRequest {
        email: req.email,
        password: req.password,
    };

    match state.user_service.login(login_req).await {
        Ok(auth_response) => {
            let response = AuthResponse {
                token: auth_response.token,
                user: UserResponse {
                    id: auth_response.user.id.to_string(),
                    email: auth_response.user.email,
                    username: auth_response.user.username,
                    tier: auth_response.user.tier.to_string(),
                    verified: auth_response.user.is_active,
                },
            };
            json_response(response).into_response()
        }
        Err(_) => error_response(
            StatusCode::UNAUTHORIZED,
            "INVALID_CREDENTIALS",
            "Invalid email or password",
        ).into_response(),
    }
}

pub async fn logout_handler(
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
            )
            .into_response();
        }
    };

    if let Ok(claims) = state.user_service.validate_token(&token).await {
        if let Some(redis_coordinator) = &state.redis_coordinator {
            let mut coordinator = redis_coordinator.lock().unwrap();
            let key = format!("jti:{}", claims.jti);
            let _: Result<(), _> = coordinator.del(&key);
        }
    }

    json_response(serde_json::json!({ "message": "Logout successful" })).into_response()
}

pub async fn verify_email_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<VerifyEmailRequest>,
) -> impl IntoResponse {
    match state.email_service.verify_email_token(&req.token).await {
        Ok(_) => json_response(serde_json::json!({ 
            "verified": true,
            "email": req.email
        })).into_response(),
        Err(e) => error_response(
            StatusCode::BAD_REQUEST,
            "VERIFICATION_FAILED",
            &format!("Email verification failed: {}", e),
        ).into_response(),
    }
}

pub async fn resend_verification_handler(
    State(_state): State<Arc<AppState>>,
    Json(_req): Json<ResendVerificationRequest>,
) -> impl IntoResponse {
    // TODO: Implement resend verification
    StatusCode::OK.into_response()
}

pub async fn request_password_reset_handler(
    State(_state): State<Arc<AppState>>,
    Json(_req): Json<PasswordResetRequest>,
) -> impl IntoResponse {
    // TODO: Implement password reset request
    StatusCode::OK.into_response()
}

pub async fn reset_password_handler(
    State(_state): State<Arc<AppState>>,
    Json(_req): Json<ResetPasswordRequest>,
) -> impl IntoResponse {
    // TODO: Implement password reset
    StatusCode::OK.into_response()
}

pub async fn get_user_info_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(user_id): Path<String>,
) -> impl IntoResponse {
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

    match state.user_service.get_user_by_id(user_id).await {
        Ok(Some(user)) => {
            let response = UserResponse {
                id: user.id.to_string(),
                email: user.email,
                username: user.username,
                tier: user.tier.to_string(),
                verified: user.is_active,
            };
            json_response(response).into_response()
        }
        Ok(None) => error_response(
            StatusCode::NOT_FOUND,
            "USER_NOT_FOUND",
            "User not found",
        ).into_response(),
        Err(e) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "DATABASE_ERROR",
            &e.to_string(),
        ).into_response(),
    }
}

// ==================== Protected User Handlers ====================

pub async fn get_me_handler(
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

    match state.user_service.get_user_by_id(user_id).await {
        Ok(Some(user)) => {
            let response = UserResponse {
                id: user.id.to_string(),
                email: user.email,
                username: user.username,
                tier: user.tier.to_string(),
                verified: user.is_active,
            };
            json_response(response).into_response()
        }
        Ok(None) => error_response(
            StatusCode::NOT_FOUND,
            "USER_NOT_FOUND",
            "User not found",
        ).into_response(),
        Err(e) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "DATABASE_ERROR",
            &e.to_string(),
        ).into_response(),
    }
}

pub async fn update_profile_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(req): Json<UpdateProfileRequest>,
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

    let update_req = mr_darkpromth_db::UpdateUserRequest {
        username: req.username,
        email: None,
        tier: None,
        is_active: None,
    };

    match state.user_service.update_user(user_id, update_req).await {
        Ok(Some(user)) => {
            let response = UserResponse {
                id: user.id.to_string(),
                email: user.email,
                username: user.username,
                tier: user.tier.to_string(),
                verified: user.is_active,
            };
            json_response(response).into_response()
        }
        Ok(None) => error_response(
            StatusCode::NOT_FOUND,
            "USER_NOT_FOUND",
            "User not found",
        ).into_response(),
        Err(e) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "UPDATE_FAILED",
            &e.to_string(),
        ).into_response(),
    }
}

pub async fn change_password_handler(
    State(_state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(_req): Json<ChangePasswordRequest>,
) -> impl IntoResponse {
    // TODO: Implement password change
    StatusCode::OK.into_response()
}

pub async fn regenerate_api_key_handler(
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

    match state.user_service.regenerate_api_key(user_id).await {
        Ok(api_key_response) => {
            json_response(serde_json::json!({ 
                "api_key": api_key_response.api_key,
                "expires_at": api_key_response.expires_at
            })).into_response()
        }
        Err(e) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "REGENERATION_FAILED",
            &e.to_string(),
        ).into_response(),
    }
}

// ==================== User Stats and Preferences ====================

#[derive(Debug, Serialize)]
pub struct UserStatsResponse {
    pub total_requests: i64,
    pub this_month: i64,
    pub jailbreak_usage: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserPreferences {
    pub email_notifications: bool,
    pub two_factor_auth: bool,
    pub data_sharing: bool,
}

pub async fn get_user_stats_handler(
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

    // For now, return mock stats - in production, query from analytics service
    let stats = UserStatsResponse {
        total_requests: 0,
        this_month: 0,
        jailbreak_usage: 0,
    };

    json_response(stats).into_response()
}

pub async fn get_user_preferences_handler(
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

    // Return default preferences - in production, store and retrieve from database
    let prefs = UserPreferences {
        email_notifications: true,
        two_factor_auth: false,
        data_sharing: false,
    };

    json_response(prefs).into_response()
}

pub async fn update_user_preferences_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(req): Json<UserPreferences>,
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

    // In production, save to database
    json_response(req).into_response()
}

pub async fn upload_avatar_handler(
    State(_state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    // TODO: Implement avatar upload with file storage
    json_response(serde_json::json!({ 
        "avatar_url": "/avatars/default.png",
        "message": "Avatar upload endpoint - implementation pending"
    })).into_response()
}


pub async fn get_api_key_status_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    // Fetch all users instead of non-existent API keys
    match state.user_service.list_users(1000, 0).await { // Using a high limit for admin view
        Ok(users) => {
            let response: Vec<serde_json::Value> = users.into_iter().map(|user| {
                serde_json::json!({
                    "id": user.id,
                    "provider": "Internal",
                    "label": user.username, // Use username as a label
                    "is_active": user.is_active,
                    "failure_count": 0, // Mock data
                    "last_used": null, // No last_used field available in UserResponse
                    "api_key": user.api_key, // Include the actual API key
                    "expires_at": user.api_key_expires_at,
                })
            }).collect();
            json_response(response).into_response()
        }
        Err(e) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "DATABASE_ERROR",
            &e.to_string(),
        ).into_response(),
    }
}
