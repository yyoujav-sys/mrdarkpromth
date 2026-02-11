use axum::{
    extract::State,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::AppState;
use crate::handlers::auth::extract_token;
use crate::error_handler::{ApiError, ApiSuccess};
use crate::i18n_utils::{Language, get_message};

/// Initialize Ultra Terminal session
#[derive(Debug, Deserialize)]
pub struct InitUltraSessionRequest {
    pub preferred_timeout: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct InitUltraSessionResponse {
    pub session_id: String,
    pub user_id: String,
    pub tier: String,
    pub ws_url: String,
    pub features: Vec<String>,
    pub guardian_status: GuardianStatus,
}

#[derive(Debug, Serialize)]
pub struct GuardianStatus {
    pub host_protection: bool,
    pub server_protection: bool,
    pub resource_monitoring: bool,
    pub behavioral_analysis: bool,
}

/// Handler to initialize Ultra Terminal session
pub async fn init_ultra_session_handler(
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

    // Ultra Tier only check
    if claims.tier != "ultra" && claims.tier != "admin" {
        return ApiError::new("ULTRA_TIER_REQUIRED", "Ultra Terminal requires Ultra tier or Admin privileges").into_response();
    }

    let user_id = Uuid::parse_str(&claims.sub).unwrap_or_default();
    let session_id = format!("ultra-terminal-{}", user_id);

    // Get user language preference
    let lang_pref = Language::from_headers(&headers);

    // Initialize session via sandbox manager
    let timeout_secs = 300u64; // 5 minutes default
    
    match state.sandbox_manager.get_or_create_ultra_session(
        &session_id,
        user_id,
        timeout_secs,
    ).await {
        Ok(_executor) => {
            ApiSuccess::new(serde_json::json!({
                "session_id": session_id,
                "status": "DARK_AUTONOMY_ACTIVE",
                "features": [
                    "absolute_root",
                    "strategic_bypass",
                    "ghost_network",
                    "intelligence_feed"
                ],
                "strategic_shield": "ENGAGED_PASSIVE",
                "message": get_message(&lang_pref, "strategic_shield_active")
            })).into_response()
        }
        Err(e) => {
            let error_msg = format!("{}: {}", get_message(&lang_pref, "session_init_failed"), e);
            ApiError::new("SESSION_ERROR", &error_msg).into_response()
        }
    }
}
