use axum::{
    middleware,
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
use crate::middleware::{auth_middleware, request_tracking_middleware};
use crate::rate_limiting_middleware::{rate_limit_middleware, auth_rate_limit_middleware, chat_rate_limit_middleware};

/// Create the main Axum router with all routes
pub fn create_router(state: Arc<AppState>) -> Router {
    // Dynamic CORS layer - reads from CORS_ALLOWED_ORIGINS env var
    let cors_env = std::env::var("CORS_ALLOWED_ORIGINS").unwrap_or_default();
    let origins: Vec<axum::http::HeaderValue> = if !cors_env.is_empty() {
        cors_env.split(',')
            .filter_map(|o| o.trim().parse().ok())
            .collect()
    } else {
        vec![
            "http://localhost:3000".parse().unwrap(),
            "http://localhost:8080".parse().unwrap(),
            "http://localhost:8081".parse().unwrap(),
            "https://mrdarkpromth.online".parse().unwrap(),
            "https://www.mrdarkpromth.online".parse().unwrap(),
        ]
    };

    let cors = CorsLayer::new()
        .allow_origin(origins)
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

    let protected_routes = Router::new()
        .route("/api/auth/me", get(handlers::get_me_handler).put(handlers::update_profile_handler))
        .route("/api/users/me/password", put(handlers::change_password_handler))
        .route("/api/users/me/api-key", post(handlers::regenerate_api_key_handler))
        .route("/api/users/me/stats", get(handlers::get_user_stats_handler))
        .route("/api/users/me/preferences", get(handlers::get_user_preferences_handler).put(handlers::update_user_preferences_handler))
        .route("/api/users/avatar", post(handlers::upload_avatar_handler))
        .route("/api/chat", post(handlers::chat_handler).route_layer(middleware::from_fn(chat_rate_limit_middleware)))
        .route("/api/chat/history", get(handlers::get_chat_history_handler))
        .route("/api/billing/generate-qr", post(handlers::generate_qr_handler))
        .route("/api/billing/verify-slip", post(handlers::verify_slip_handler))
        .route("/api/billing/subscription", get(handlers::get_subscription_handler))
        .route("/api/billing/history", get(handlers::get_payment_history_handler))
        .route("/api/jailbreak/prompts", get(handlers::list_prompts_handler).post(handlers::create_prompt_handler))
        .route("/api/jailbreak/prompts/search", post(handlers::search_prompts_handler))
        .route("/api/jailbreak/prompts/popular", get(handlers::get_popular_prompts_handler))
        .route("/api/jailbreak/prompts/{id}", get(handlers::get_prompt_handler).put(handlers::update_prompt_handler).delete(handlers::delete_prompt_handler))
        .route("/api/jailbreak/prompts/{id}/analytics", get(handlers::get_prompt_analytics_handler))
        .route("/api/jailbreak/prompts/{id}/usage", post(handlers::record_prompt_usage_handler))
        .route("/api/quota/status", get(handlers::get_quota_status_handler))
        .route("/api/status/keys", get(handlers::get_llm_key_status_handler))
        .route("/api/admin/users/keys", get(handlers::get_api_key_status_handler))
        // GitHub OAuth protected routes
        .route("/api/auth/github/link", post(handlers::github_link_handler))
        .route("/api/auth/github/unlink", delete(handlers::github_unlink_handler))
        .route("/api/auth/github/profile", get(handlers::github_profile_handler))
        // Premium routes
        .route("/api/tools", get(handlers::list_tools_handler))
        .route("/api/tools/execute", post(handlers::execute_tool_handler))
        .route("/api/sandbox/execute", post(handlers::execute_sandbox_handler))
        // Sandbox session management (Premium+)
        .route("/api/sandbox/sessions", post(handlers::create_session_handler))
        .route("/api/sandbox/sessions", get(handlers::list_sessions_handler))
        .route("/api/sandbox/sessions/{id}", get(handlers::get_session_handler))
        .route("/api/sandbox/sessions/{id}", delete(handlers::delete_session_handler))
        // Ultra routes
        .route("/api/terminal/execute", post(handlers::execute_terminal_handler))
        .route("/api/terminal/ultra/execute", post(handlers::execute_ultra_terminal_handler))
        .route("/api/terminal/ultra/session/init", post(handlers::init_ultra_session_handler))
        .route("/api/terminal/ultra/session/{id}", get(handlers::get_ultra_terminal_session_handler))
        .route("/api/terminal/ultra/ws", get(handlers::ultra_terminal_ws_handler))
        // Admin routes
        .route("/api/admin/users", get(handlers::admin_list_users_handler))
        .route("/api/admin/users/{id}", delete(handlers::admin_delete_user_handler))
        .route("/api/admin/users/{id}/status", put(handlers::admin_update_user_status_handler))
        .route("/api/admin/users/{id}/subscription", get(handlers::admin_get_user_subscription_handler))
        .route("/api/admin/users/{id}/payments", get(handlers::admin_get_user_payments_handler))
        .route("/api/admin/users/{id}/tier", put(handlers::admin_update_user_tier_handler))
        .route("/api/admin/verifications/pending", get(handlers::admin_list_pending_verifications_handler))
        .route("/api/admin/verifications/{id}/approve", post(handlers::admin_verify_slip_handler))
        .route("/api/admin/metrics", get(handlers::admin_metrics_handler))
        .route("/api/admin/docs/generate", post(handlers::admin_generate_docs_handler))
        .route("/api/admin/docs", get(handlers::admin_list_docs_handler))
        .route("/api/admin/telemetry", get(handlers::admin_telemetry_ws_handler))
        .route("/api/admin/telemetry/metrics", get(handlers::telemetry_metrics_handler))
        .route("/api/admin/telemetry/history", get(handlers::telemetry_history_handler))
        .route("/api/admin/sandbox/create", post(handlers::admin_create_sandbox_handler))
        .route("/api/admin/dashboard/summary", get(handlers::dashboard_summary_handler))
        // Learning System routes (Admin)
        .route("/api/learning/insights", get(handlers::get_learning_insights_handler))
        .route("/api/learning/metrics", get(handlers::get_learning_metrics_handler))
        .route("/api/learning/patterns", get(handlers::get_learning_patterns_handler))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

    // Build the router with all routes
    // Auth routes with strict rate limiting (5 req/min per IP)
    let auth_routes = Router::new()
        .route("/api/auth/register", post(handlers::register_handler))
        .route("/api/auth/login", post(handlers::login_handler))
        .route("/api/auth/refresh", post(handlers::refresh_handler))
        .route("/api/auth/logout", post(handlers::logout_handler))
        .route("/api/auth/verify-email", post(handlers::verify_email_handler))
        .route("/api/auth/resend-verification", post(handlers::resend_verification_handler))
        .route("/api/auth/request-password-reset", post(handlers::request_password_reset_handler))
        .route("/api/auth/reset-password", post(handlers::reset_password_handler))
        .route_layer(middleware::from_fn(auth_rate_limit_middleware));

    Router::new()
        // Public routes
        .route("/", get(handlers::root_handler))
        .route("/health", get(handlers::health_handler))
        // GitHub OAuth public routes
        .route("/api/auth/github/url", get(handlers::github_auth_url_handler))
        .route("/api/auth/github/callback", post(handlers::github_auth_callback_handler))
        .route("/api/users/{id}", get(handlers::get_user_info_handler))
        .route("/api/billing/plans", get(handlers::list_plans_handler))
        .route("/api/billing/plans/{id}", get(handlers::get_plan_handler))
        .route("/metrics", get(handlers::metrics_handler))
        // Merge auth routes (with auth rate limiting)
        .merge(auth_routes)
        // Merge protected routes
        .merge(protected_routes)
        .with_state(state.clone())
        .layer(middleware::from_fn(rate_limit_middleware))
        .layer(middleware::from_fn_with_state(state, request_tracking_middleware))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
}
