use axum::{
    extract::State,
    response::IntoResponse,
};
use std::sync::Arc;
use crate::AppState;
use crate::handlers::auth::extract_token;
use crate::error_handler::{ApiError, ApiSuccess};
use mr_darkpromth_services::UserTier;
use uuid::Uuid;

pub async fn get_quota_status_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    // Extract and validate token
    let token = match extract_token(&headers) {
        Some(t) => t,
        None => return ApiError::new("MISSING_TOKEN", "Auth required").into_response(),
    };

    let claims = match state.user_service.validate_token(&token).await {
        Ok(c) => c,
        Err(_) => return ApiError::new("INVALID_TOKEN", "Invalid token").into_response(),
    };

    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return ApiError::new("INVALID_USER", "Invalid user ID").into_response(),
    };

    let tier: UserTier = claims.tier.parse().unwrap_or(UserTier::Free);

    match state.jailbreak_service.get_quota_status(user_id, tier).await {
        Ok((used, remaining)) => {
            ApiSuccess::new(serde_json::json!({
                "tier": tier.as_str(),
                "daily_limit": tier.max_daily_messages(),
                "used_today": used,
                "remaining_today": remaining,
                "quota_exceeded": remaining <= 0,
                "reset_time": "tomorrow at midnight UTC" // Simplified
            })).into_response()
        },
        Err(e) => {
            log::error!("Failed to get quota status: {}", e);
            ApiError::new("FETCH_ERROR", "Failed to fetch quota status").into_response()
        }
    }
}
