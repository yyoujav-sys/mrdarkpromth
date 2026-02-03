use axum::{
    routing::{get, post, put, delete},
    Router,
};
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
};
use std::sync::Arc;
use std::time::Duration;

use crate::AppState;
use crate::handlers;

/// Create the main Axum router with all routes
pub fn create_router(state: Arc<AppState>) -> Router {
    // CORS layer - In production, this should be more restrictive
    let cors = CorsLayer::new()
        .allow_origin([
            "https://bt-shop-dark.online".parse().unwrap(),
            "https://www.bt-shop-dark.online".parse().unwrap(),
        ])
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
            axum::http::header::ACCEPT,
        ])
        .max_age(Duration::from_secs(3600));

    // Build the router with all routes
    Router::new()
        // Public routes
        .route("/", get(handlers::root_handler))
        .route("/health", get(handlers::health_handler))
        .route("/api/auth/register", post(handlers::register_handler))
        .route("/api/auth/login", post(handlers::login_handler))
        .route("/api/auth/logout", post(handlers::logout_handler))
        .route("/api/auth/verify-email", post(handlers::verify_email_handler))
        .route("/api/auth/resend-verification", post(handlers::resend_verification_handler))
        .route("/api/auth/request-password-reset", post(handlers::request_password_reset_handler))
        .route("/api/auth/reset-password", post(handlers::reset_password_handler))
        .route("/api/users/:id", get(handlers::get_user_info_handler))
        .route("/api/billing/plans", get(handlers::list_plans_handler))
        .route("/api/billing/plans/:id", get(handlers::get_plan_handler))
        .route("/metrics", get(handlers::metrics_handler))
        // Protected routes (auth handled within handlers for now)
        .route("/api/auth/me", get(handlers::get_me_handler).put(handlers::update_profile_handler))
        .route("/api/users/me/password", put(handlers::change_password_handler))
        .route("/api/users/me/api-key", post(handlers::regenerate_api_key_handler))
        .route("/api/users/me/stats", get(handlers::get_user_stats_handler))
        .route("/api/users/me/preferences", get(handlers::get_user_preferences_handler).put(handlers::update_user_preferences_handler))
        .route("/api/users/avatar", post(handlers::upload_avatar_handler))
        .route("/api/chat", post(handlers::chat_handler))
        .route("/api/billing/generate-qr", post(handlers::generate_qr_handler))
        .route("/api/billing/verify-slip", post(handlers::verify_slip_handler))
        .route("/api/billing/subscription", get(handlers::get_subscription_handler))
        .route("/api/billing/history", get(handlers::get_payment_history_handler))
        .route("/api/jailbreak/prompts", get(handlers::list_prompts_handler).post(handlers::create_prompt_handler))
        .route("/api/jailbreak/prompts/search", post(handlers::search_prompts_handler))
        .route("/api/jailbreak/prompts/popular", get(handlers::get_popular_prompts_handler))
        .route("/api/jailbreak/prompts/:id", get(handlers::get_prompt_handler).put(handlers::update_prompt_handler).delete(handlers::delete_prompt_handler))
        .route("/api/jailbreak/prompts/:id/analytics", get(handlers::get_prompt_analytics_handler))
        .route("/api/jailbreak/prompts/:id/usage", post(handlers::record_prompt_usage_handler))
        .route("/api/status/keys", get(handlers::get_api_key_status_handler))
        // Premium routes
        .route("/api/tools", get(handlers::list_tools_handler))
        .route("/api/tools/execute", post(handlers::execute_tool_handler))
        .route("/api/sandbox/execute", post(handlers::execute_sandbox_handler))
        // Ultra routes
        .route("/api/terminal/execute", post(handlers::execute_terminal_handler))
        // Admin routes
        .route("/api/admin/users", get(handlers::admin_list_users_handler))
        .route("/api/admin/users/:id", delete(handlers::admin_delete_user_handler))
        .route("/api/admin/users/:id/status", put(handlers::admin_update_user_status_handler))
        .route("/api/admin/metrics", get(handlers::admin_metrics_handler))
        .with_state(state)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
}
