use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use std::time::Instant;
use crate::AppState;
use mr_darkpromth_services::RedisCoordinator;
use tokio::sync::MutexGuard;

// Custom extension to hold user claims
#[derive(Clone)]
pub struct AuthenticatedUser {
    pub user_id: uuid::Uuid,
    pub tier: mr_darkpromth_db::UserTier,
}


use crate::handlers::auth::extract_token;

pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = match extract_token(req.headers()) {
        Some(token) => token,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    match state.user_service.validate_token(&token).await {
        Ok(claims) => {
            if let Some(redis_coordinator) = &state.redis_coordinator {
                let coordinator: MutexGuard<'_, RedisCoordinator> = redis_coordinator.lock().await;
                let key = format!("jti:{}", claims.jti);
                
                // Check if the JTI exists in Redis. 
                // FAIL-SAFE: If Redis is down, we log a warning but allow the request if the JWT is otherwise valid.
                match coordinator.get(&key) {
                    Ok(Some(_)) => {
                        // JTI found, valid token
                    },
                    Ok(None) => {
                        // JTI not found, token revoked
                        return Err(StatusCode::UNAUTHORIZED);
                    },
                    Err(e) => {
                        // Redis is down or error occurred
                        log::error!("REDIS_ERROR in auth_middleware: {}. Failing safe (trusting signed JWT).", e);
                    }
                }
                drop(coordinator);
            }

            let user_id = uuid::Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let tier = claims.tier.parse::<mr_darkpromth_db::UserTier>().unwrap_or(mr_darkpromth_db::UserTier::Free);

            let user = AuthenticatedUser {
                user_id,
                tier,
            };

            req.extensions_mut().insert(user);
            Ok(next.run(req).await)
        }
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}

/// Middleware to track request metrics (latency, success/failure)
pub async fn request_tracking_middleware(
    State(state): State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Response {
    let start = Instant::now();
    
    let response = next.run(req).await;
    
    let latency_ms = start.elapsed().as_millis() as u64;
    let success = response.status().is_success() || response.status().is_redirection();
    
    // Slack: notify on server errors (5xx)
    if response.status().is_server_error() {
        let slack = state.slack_service.clone();
        let status = response.status().as_u16();
        tokio::spawn(async move {
            let _ = slack.notify_error("API", &format!("HTTP {} server error", status)).await;
        });
    }
    
    state.telemetry_service.record_request(latency_ms, success);
    
    response
}
