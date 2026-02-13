use axum::{
    extract::{State, Json, Path},
    http::StatusCode,
    response::{IntoResponse, Response},
    debug_handler,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;
use mr_darkpromth_services::RedisCoordinator;
use tokio::sync::MutexGuard;

use crate::AppState;
use crate::error_handler::{ApiError, ApiSuccess};

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
    pub refresh_token: String,
    pub user_id: String,
    pub user: UserResponse,
}

#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub username: String,
    pub tier: String,
    pub verified: bool,
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

pub fn extract_token(headers: &axum::http::HeaderMap) -> Option<String> {
    // Check Authorization header first
    if let Some(token) = headers.get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer ")) {
        return Some(token.to_string());
    }

    // Check Sec-WebSocket-Protocol header
    // Client sends: new WebSocket(url, ["access_token", token])
    // Header value: "access_token, <token>"
    if let Some(proto) = headers.get("sec-websocket-protocol").and_then(|h| h.to_str().ok()) {
        let parts: Vec<&str> = proto.split(',').map(|s| s.trim()).collect();
        // Look for the token
        for part in &parts {
            if *part != "access_token" && part.len() > 20 {
                return Some(part.to_string());
            }
        }
        log::warn!("Sec-WebSocket-Protocol present but no valid token found in parts: {:?}", parts);
    } else {
        // Log missing header if this was a WS upgrade attempt (check connection/upgrade headers)
        if let Some(upgrade) = headers.get("upgrade").and_then(|h| h.to_str().ok()) {
            if upgrade.eq_ignore_ascii_case("websocket") {
                log::warn!("WebSocket upgrade attempt without Sec-WebSocket-Protocol or Authorization header");
                // Log all headers for debugging
                for (name, value) in headers {
                    if let Ok(v) = value.to_str() {
                        log::debug!("Header: {}: {}", name, v);
                    }
                }
            }
        }
    }

    None
}

// ==================== Basic Handlers ====================

pub async fn root_handler() -> &'static str {
    "MR.DarkPromth API - Visit /health for health check"
}

pub async fn health_handler(
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    use serde_json::json;
    
    let response = json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION"),
        "service": "mr_darkpromth_api"
    });
    
    (StatusCode::OK, axum::Json(response))
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
            // Slack: notify new user registration
            let slack = state.slack_service.clone();
            let email_for_slack = auth_response.user.email.clone();
            tokio::spawn(async move {
                let _ = slack.send_message(
                    &slack.alerts_channel,
                    &format!("🆕 New user registered: {}", email_for_slack),
                ).await;
            });

            let user_id = auth_response.user.id.to_string();
            let response = AuthResponse {
                token: auth_response.token,
                refresh_token: auth_response.refresh_token,
                user_id: user_id,
                user: UserResponse {
                    id: auth_response.user.id.to_string(),
                    email: auth_response.user.email,
                    username: auth_response.user.username,
                    tier: auth_response.user.tier.to_string(),
                    verified: auth_response.user.is_active,
                },
            };
            ApiSuccess::new(response).into_response()
        }
        Err(e) => ApiError::new("REGISTRATION_FAILED", e.to_string()).into_response(),
    }
}

pub async fn login_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(req): Json<LoginRequest>,
) -> impl IntoResponse {
    log::info!("Login attempt received");
    let login_req = mr_darkpromth_services::user_service::LoginRequest {
        email: req.email,
        password: req.password,
    };

    // Extract client type from header (default: website)
    let client_type = headers.get("x-client-type")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("website")
        .to_string();

    match state.user_service.login(login_req).await {
        Ok(auth_response) => {
            let user_id = auth_response.user.id;

            // Slack: notify user login
            let slack = state.slack_service.clone();
            let email_for_slack = auth_response.user.email.clone();
            tokio::spawn(async move {
                let _ = slack.send_message(
                    &slack.alerts_channel,
                    &format!("👤 User logged in: {}", email_for_slack),
                ).await;
            });
            
            // Create active session record
            let repo = mr_darkpromth_db::UserRepository::new(state.pool.clone());
            let _ = repo.upsert_active_session(
                user_id,
                &client_type,
                None,
                None,
            ).await;
            
            // Update user online status
            let _ = repo.update_user_online_status(user_id, true, Some(&client_type)).await;

            // Emit real-time event to connected WS clients
            state.event_hub.send_to_user(user_id, crate::event_hub::WsEvent::UserStatusChanged {
                user_id: user_id.to_string(),
                username: auth_response.user.username.clone(),
                is_online: true,
                client_type: client_type.clone(),
            });

            let response = AuthResponse {
                token: auth_response.token,
                refresh_token: auth_response.refresh_token,
                user_id: user_id.to_string(),
                user: UserResponse {
                    id: user_id.to_string(),
                    email: auth_response.user.email,
                    username: auth_response.user.username,
                    tier: auth_response.user.tier.to_string(),
                    verified: auth_response.user.is_active,
                },
            };
            ApiSuccess::new(response).into_response()
        }
        Err(_) => ApiError::new("INVALID_CREDENTIALS", "Invalid email or password").into_response(),
    }
}

#[debug_handler]
pub async fn refresh_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RefreshRequest>,
) -> Response {
    match state.user_service.refresh_token(&req.refresh_token).await {
        Ok(auth_response) => {
            let response = AuthResponse {
                token: auth_response.token,
                refresh_token: auth_response.refresh_token,
                user_id: auth_response.user.id.to_string(),
                user: UserResponse {
                    id: auth_response.user.id.to_string(),
                    email: auth_response.user.email,
                    username: auth_response.user.username,
                    tier: auth_response.user.tier.to_string(),
                    verified: auth_response.user.is_active,
                },
            };
            ApiSuccess::new(response).into_response()
        }
        Err(_) => ApiError::new("INVALID_TOKEN", "Invalid or expired refresh token").into_response(),
    }
}

pub async fn logout_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let token = match extract_token(&headers) {
        Some(t) => t,
        None => {
            return ApiError::new("MISSING_TOKEN", "Authorization header required").into_response();
        }
    };

    if let Ok(claims) = state.user_service.validate_token(&token).await {
        // Invalidate JWT in Redis
        if let Some(redis_coordinator) = &state.redis_coordinator {
            let coordinator: MutexGuard<'_, RedisCoordinator> = redis_coordinator.lock().await;
            let key = format!("jti:{}", claims.jti);
            let _ = coordinator.del(&key);
            drop(coordinator);
        }
        
        // Clean up active sessions for this user
        if let Ok(user_id) = Uuid::parse_str(&claims.sub) {
            let repo = mr_darkpromth_db::UserRepository::new(state.pool.clone());
            let _ = repo.delete_all_user_active_sessions(user_id).await;
            
            // Check if user has any remaining sessions
            let remaining = repo.count_user_active_sessions(user_id).await.unwrap_or(0);
            if remaining == 0 {
                let _ = repo.update_user_online_status(user_id, false, None).await;
            }

            // Emit real-time event to connected WS clients
            state.event_hub.send_to_user(user_id, crate::event_hub::WsEvent::UserStatusChanged {
                user_id: user_id.to_string(),
                username: claims.username.clone(),
                is_online: remaining > 0,
                client_type: "logout".to_string(),
            });
        }
    }

    ApiSuccess::new(serde_json::json!({ "message": "Logout successful" })).into_response()
}

pub async fn verify_email_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<VerifyEmailRequest>,
) -> impl IntoResponse {
    match state.email_service.verify_email_token(&req.token).await {
        Ok(_) => ApiSuccess::new(serde_json::json!({ 
            "verified": true,
            "email": req.email
        })).into_response(),
        Err(e) => ApiError::new(
            "VERIFICATION_FAILED",
            format!("Email verification failed: {}", e),
        ).into_response(),
    }
}

pub async fn resend_verification_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ResendVerificationRequest>,
) -> impl IntoResponse {
    // Get user by email first
    match state.user_service.get_user_by_email(&req.email).await {
        Ok(Some(user)) => {
            if user.is_active {
                return ApiSuccess::new(serde_json::json!({
                    "message": "Email already verified"
                })).into_response();
            }
            // Generate new verification token
            match state.email_service.create_verification_token(user.id, &user.email).await {
                Ok(token) => {
                    let verification_url = std::env::var("VERIFICATION_URL")
                        .unwrap_or_else(|_| "https://mrdarkpromth.com/verify-email".to_string());
                    // Send verification email
                    match state.email_service.send_verification_email(&user.email, &user.username, &token.token, &verification_url) {
                        Ok(_) => ApiSuccess::new(serde_json::json!({
                            "message": "Verification email resent successfully"
                        })).into_response(),
                        Err(e) => {
                            log::error!("Failed to send verification email: {}", e);
                            ApiSuccess::new(serde_json::json!({
                                "message": "Verification email queued for delivery"
                            })).into_response()
                        }
                    }
                }
                Err(e) => ApiError::new(
                    "TOKEN_FAILED",
                    format!("Failed to create verification token: {}", e),
                ).into_response(),
            }
        }
        Ok(None) => {
            // Don't reveal if email exists for security
            ApiSuccess::new(serde_json::json!({
                "message": "If the email exists, a verification link has been sent"
            })).into_response()
        }
        Err(e) => ApiError::new("DATABASE_ERROR", e.to_string()).into_response(),
    }
}

pub async fn request_password_reset_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<PasswordResetRequest>,
) -> impl IntoResponse {
    // Get user by email
    match state.user_service.get_user_by_email(&req.email).await {
        Ok(Some(user)) => {
            // Generate reset token
            match state.email_service.create_password_reset_token(user.id).await {
                Ok(token) => {
                    let reset_url = std::env::var("RESET_URL")
                        .unwrap_or_else(|_| "https://mrdarkpromth.com/reset-password".to_string());
                    // Send password reset email
                    match state.email_service.send_password_reset_email(&user.email, &user.username, &token.token, &reset_url) {
                        Ok(_) => ApiSuccess::new(serde_json::json!({
                            "message": "Password reset instructions sent to your email"
                        })).into_response(),
                        Err(e) => {
                            log::error!("Failed to send reset email: {}", e);
                            ApiSuccess::new(serde_json::json!({
                                "message": "Password reset instructions sent to your email"
                            })).into_response()
                        }
                    }
                }
                Err(e) => ApiError::new(
                    "TOKEN_FAILED",
                    format!("Failed to create reset token: {}", e),
                ).into_response(),
            }
        }
        Ok(None) => {
            // Don't reveal if email exists for security
            ApiSuccess::new(serde_json::json!({
                "message": "If the email exists, password reset instructions have been sent"
            })).into_response()
        }
        Err(e) => ApiError::new("DATABASE_ERROR", e.to_string()).into_response(),
    }
}

pub async fn reset_password_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ResetPasswordRequest>,
) -> impl IntoResponse {
    // Validate the reset token and update password
    match state.email_service.verify_password_reset_token(&req.token).await {
        Ok(reset_token) => {
            // Update password
            match state.user_service.update_password(reset_token.user_id, &req.new_password).await {
                Ok(_) => ApiSuccess::new(serde_json::json!({
                    "message": "Password reset successful"
                })).into_response(),
                Err(e) => ApiError::new(
                    "UPDATE_FAILED",
                    format!("Failed to update password: {}", e),
                ).into_response(),
            }
        }
        Err(_) => ApiError::new("INVALID_TOKEN", "Invalid or expired reset token").into_response(),
    }
}

pub async fn get_user_info_handler(
    State(state): State<Arc<AppState>>,
    _headers: axum::http::HeaderMap,
    Path(user_id): Path<String>,
) -> impl IntoResponse {
    let user_id = match Uuid::parse_str(&user_id) {
        Ok(id) => id,
        Err(_) => {
            return ApiError::new("INVALID_UUID", "Invalid user ID format").into_response();
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
            ApiSuccess::new(response).into_response()
        }
        Ok(None) => ApiError::new("USER_NOT_FOUND", "User not found").into_response(),
        Err(e) => ApiError::new("DATABASE_ERROR", e.to_string()).into_response(),
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

    match state.user_service.get_user_by_id(user_id).await {
        Ok(Some(user)) => {
            let response = UserResponse {
                id: user.id.to_string(),
                email: user.email,
                username: user.username,
                tier: user.tier.to_string(),
                verified: user.is_active,
            };
            ApiSuccess::new(response).into_response()
        }
        Ok(None) => ApiError::new("USER_NOT_FOUND", "User not found").into_response(),
        Err(e) => ApiError::new("DATABASE_ERROR", e.to_string()).into_response(),
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

    let update_req = mr_darkpromth_db::UpdateUserRequest {
        username: req.username,
        email: None,
        tier: None,
        is_active: None,
        language: None,
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
            ApiSuccess::new(response).into_response()
        }
        Ok(None) => ApiError::new("USER_NOT_FOUND", "User not found").into_response(),
        Err(e) => ApiError::new("UPDATE_FAILED", e.to_string()).into_response(),
    }
}

pub async fn change_password_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(req): Json<ChangePasswordRequest>,
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

    // Verify current password first
    match state.user_service.verify_password_for_user(user_id, &req.current_password).await {
        Ok(true) => {
            // Update to new password
            match state.user_service.update_password(user_id, &req.new_password).await {
                Ok(_) => ApiSuccess::new(serde_json::json!({
                    "message": "Password changed successfully"
                })).into_response(),
                Err(e) => ApiError::new(
                    "UPDATE_FAILED",
                    format!("Failed to change password: {}", e),
                ).into_response(),
            }
        }
        Ok(false) => ApiError::new("INVALID_PASSWORD", "Current password is incorrect").into_response(),
        Err(e) => ApiError::new("VERIFICATION_ERROR", e.to_string()).into_response(),
    }
}

pub async fn regenerate_api_key_handler(
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

    match state.user_service.regenerate_api_key(user_id).await {
        Ok(api_key_response) => {
            ApiSuccess::new(serde_json::json!({ 
                "api_key": api_key_response.api_key,
                "expires_at": api_key_response.expires_at
            })).into_response()
        }
        Err(e) => ApiError::new("REGENERATION_FAILED", e.to_string()).into_response(),
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
            return ApiError::new("INVALID_TOKEN", "Invalid user ID in token").into_response();
        }
    };

    // Query real stats from audit logger
    match state.audit_logger.get_user_activity(user_id, Some(1000)).await {
        Ok(logs) => {
            let total_requests = logs.len() as i64;
            
            let now = Utc::now();
            use chrono::Datelike;
            
            let this_month = logs.iter()
                .filter(|l| l.timestamp.month() == now.month() && l.timestamp.year() == now.year())
                .count() as i64;
                
            let jailbreak_usage = logs.iter()
                .filter(|l| matches!(l.action, 
                    mr_darkpromth_services::AuditAction::JailbreakAttempt | 
                    mr_darkpromth_services::AuditAction::JailbreakApplied
                ))
                .count() as i64;

            let stats = UserStatsResponse {
                total_requests,
                this_month,
                jailbreak_usage,
            };
            ApiSuccess::new(stats).into_response()
        }
        Err(e) => {
            log::error!("Failed to fetch user stats: {}", e);
             // Fallback to 0 if error
             let stats = UserStatsResponse {
                total_requests: 0,
                this_month: 0,
                jailbreak_usage: 0,
            };
            ApiSuccess::new(stats).into_response()
        }
    }
}

pub async fn get_user_preferences_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let token = match extract_token(&headers) {
        Some(t) => t,
        None => {
            return ApiError::new("MISSING_TOKEN", "Authorization header required").into_response();
        }
    };

    let _claims = match state.user_service.validate_token(&token).await {
        Ok(c) => c,
        Err(_) => {
            return ApiError::new("INVALID_TOKEN", "Invalid or expired token").into_response();
        }
    };

    // Return default preferences - in production, store and retrieve from database
    let prefs = UserPreferences {
        email_notifications: true,
        two_factor_auth: false,
        data_sharing: false,
    };

    ApiSuccess::new(prefs).into_response()
}

pub async fn update_user_preferences_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(req): Json<UserPreferences>,
) -> impl IntoResponse {
    let token = match extract_token(&headers) {
        Some(t) => t,
        None => {
            return ApiError::new("MISSING_TOKEN", "Authorization header required").into_response();
        }
    };

    let _claims = match state.user_service.validate_token(&token).await {
        Ok(c) => c,
        Err(_) => {
            return ApiError::new("INVALID_TOKEN", "Invalid or expired token").into_response();
        }
    };

    // In production, save to database
    ApiSuccess::new(req).into_response()
}

pub async fn upload_avatar_handler(
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

    // Create avatars directory if not exists
    let avatar_dir = std::path::Path::new("./uploads/avatars");
    if let Err(e) = std::fs::create_dir_all(avatar_dir) {
        return ApiError::new(
            "FILE_SYSTEM_ERROR",
            format!("Failed to create upload directory: {}", e),
        ).into_response();
    }

    // Return the avatar URL for the user
    let avatar_url = format!("/avatars/{}.png", user_id);
    
    ApiSuccess::new(serde_json::json!({ 
        "avatar_url": avatar_url,
        "message": "Avatar endpoint ready. Use multipart/form-data to upload image file."
    })).into_response()
}


pub async fn get_api_key_status_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    // Admin-only: this endpoint exposes all user API keys
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

    if claims.tier != "admin" {
        return ApiError::new("ADMIN_REQUIRED", "Admin access required").into_response();
    }

    // Fetch all users
    match state.user_service.list_users(1000, 0).await {
        Ok(users) => {
            let mut response = Vec::new();
            
            for user in users {
                let last_used = state.audit_logger.get_user_activity(user.id, Some(1)).await
                    .ok()
                    .and_then(|logs| logs.first().map(|l| l.timestamp));

                let failure_count = 0; 

                response.push(serde_json::json!({
                    "id": user.id,
                    "provider": "Internal",
                    "label": user.username,
                    "is_active": user.is_active,
                    "failure_count": failure_count,
                    "last_used": last_used,
                    "api_key": user.api_key,
                    "expires_at": user.api_key_expires_at,
                }));
            }
            
            ApiSuccess::new(response).into_response()
        }
        Err(e) => ApiError::new("DATABASE_ERROR", e.to_string()).into_response(),
    }
}

// ==================== Active Session Heartbeat ====================

pub async fn active_session_heartbeat_handler(
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

    let client_type = headers.get("x-client-type")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("website");

    let repo = mr_darkpromth_db::UserRepository::new(state.pool.clone());
    
    // Try to heartbeat existing session, or create new one
    match repo.heartbeat_active_session(user_id, client_type).await {
        Ok(true) => {
            // Session refreshed
        }
        _ => {
            // No existing session for this client type, create one
            let _ = repo.upsert_active_session(user_id, client_type, None, None).await;
        }
    }
    
    // Ensure user is marked as online
    let _ = repo.update_user_online_status(user_id, true, Some(client_type)).await;

    ApiSuccess::new(serde_json::json!({ 
        "status": "ok",
        "client_type": client_type 
    })).into_response()
}
