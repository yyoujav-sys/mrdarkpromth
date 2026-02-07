use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use crate::AppState;

// Custom extension to hold user claims
#[derive(Clone)]
pub struct AuthenticatedUser {
    pub user_id: uuid::Uuid,
    pub tier: mr_darkpromth_db::UserTier,
}


pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|header| header.strip_prefix("Bearer "));

    let token = if let Some(token) = token {
        token
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    match state.user_service.validate_token(token).await {
        Ok(claims) => {
            if let Some(redis_coordinator) = &state.redis_coordinator {
                let mut coordinator = redis_coordinator.lock().unwrap();
                let key = format!("jti:{}", claims.jti);
                if coordinator.get(&key).unwrap_or(None).is_none() {
                    // Token jti not found in Redis, so it's considered revoked/invalid
                    return Err(StatusCode::UNAUTHORIZED);
                }
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
