use axum::{
    extract::State,
    http::{header, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    body::Body,
};
use axum::Json;
use mr_darkpromth_core::tier::UserTier;
use serde_json::json;
use std::sync::Arc;

use crate::UserService;

#[derive(Clone)]
pub struct AuthState {
    pub user_service: Arc<UserService>,
}

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub id: uuid::Uuid,
    pub username: String,
    pub email: String,
    pub tier: UserTier,
}

fn parse_user_tier(raw: &str) -> UserTier {
    raw.parse().unwrap_or(UserTier::Free)
}

fn has_required_tier(current: &UserTier, required: &UserTier) -> bool {
    current >= required
}

pub async fn auth_middleware(
    State(state): State<AuthState>,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    let auth_header = request.headers().get(header::AUTHORIZATION).and_then(|h| h.to_str().ok());

    if let Some(auth_header) = auth_header {
        if let Some(token) = auth_header.strip_prefix("Bearer ") {
            if let Ok(claims) = state.user_service.validate_token(token).await {
                if let Ok(user_id) = uuid::Uuid::parse_str(&claims.sub) {
                    let auth_user = AuthenticatedUser {
                        id: user_id,
                        username: claims.username,
                        email: claims.email,
                        tier: parse_user_tier(&claims.tier),
                    };
                    request.extensions_mut().insert(auth_user);
                }
            }
        }
    }
    next.run(request).await
}

pub async fn api_key_middleware(
    State(state): State<AuthState>,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    if let Some(api_key) = request.headers().get("X-API-Key").and_then(|h| h.to_str().ok()) {
        if let Ok(Some(user)) = state.user_service.validate_api_key(api_key).await {
            let auth_user = AuthenticatedUser {
                id: user.id,
                username: user.username,
                email: user.email,
                tier: user.tier,
            };
            request.extensions_mut().insert(auth_user);
        }
    }
    next.run(request).await
}

// This is the new, simplified middleware function for checking tiers.
async fn require_tier_level(
    request: &Request<Body>,
    required_tier: &UserTier,
) -> Result<(), Response> {
    let extensions = request.extensions();
    let auth_user = extensions.get::<AuthenticatedUser>().ok_or_else(|| {
        (StatusCode::UNAUTHORIZED, Json(json!({ "error": "Authentication required" }))).into_response()
    })?;

    if !has_required_tier(&auth_user.tier, required_tier) {
        Err((StatusCode::FORBIDDEN, Json(json!({
            "error": "Insufficient permissions",
            "required_tier": format!("{:?}", required_tier),
            "current_tier": format!("{:?}", auth_user.tier)
        }))).into_response())
    } else {
        Ok(())
    }
}

pub async fn require_premium_tier(
    request: Request<Body>,
    next: Next,
) -> Response {
    match require_tier_level(&request, &UserTier::Premium).await {
        Ok(_) => next.run(request).await,
        Err(response) => response,
    }
}

pub async fn require_ultra_tier(
    request: Request<Body>,
    next: Next,
) -> Response {
    match require_tier_level(&request, &UserTier::Ultra).await {
        Ok(_) => next.run(request).await,
        Err(response) => response,
    }
}

pub async fn require_admin_tier(
    request: Request<Body>,
    next: Next,
) -> Response {
    match require_tier_level(&request, &UserTier::Admin).await {
        Ok(_) => next.run(request).await,
        Err(response) => response,
    }
}

// Helper function to extract authenticated user from request
pub fn extract_auth_user(request: &Request<Body>) -> Result<&AuthenticatedUser, StatusCode> {
    request.extensions().get::<AuthenticatedUser>()
        .ok_or(StatusCode::UNAUTHORIZED)
}

// Helper function to check if user has specific tier
pub fn check_user_tier(request: &Request<Body>, required_tier: UserTier) -> Result<bool, StatusCode> {
    let auth_user = extract_auth_user(request)?;
    Ok(has_required_tier(&auth_user.tier, &required_tier))
}

// Middleware for optional authentication (doesn't fail if no auth)
pub async fn optional_auth_middleware(
    State(state): State<AuthState>,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    // Try to extract token from Authorization header
    if let Some(auth_header) = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|header| header.strip_prefix("Bearer "))
    {
        // Try to validate token
        if let Ok(claims) = state.user_service.validate_token(auth_header).await {
            if let Ok(user_id) = uuid::Uuid::parse_str(&claims.sub) {
                let user_tier = parse_user_tier(&claims.tier);

                let auth_user = AuthenticatedUser {
                    id: user_id,
                    username: claims.username,
                    email: claims.email,
                    tier: user_tier,
                };

                request.extensions_mut().insert(auth_user);
            }
        }
    }

    next.run(request).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Method, Request},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_extract_auth_user() {
        let auth_user = AuthenticatedUser {
            id: uuid::Uuid::new_v4(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            tier: UserTier::Free,
        };

        let mut request = Request::builder()
            .method(Method::GET)
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        request.extensions_mut().insert(auth_user);

        let extracted = extract_auth_user(&request).unwrap();
        assert_eq!(extracted.username, "testuser");
        assert_eq!(extracted.tier, UserTier::Free);
    }

    #[test]
    fn test_check_user_tier() {
        let auth_user = AuthenticatedUser {
            id: uuid::Uuid::new_v4(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            tier: UserTier::Ultra,
        };

        let mut request = Request::builder()
            .method(Method::GET)
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        request.extensions_mut().insert(auth_user);

        // Ultra tier should have access to everything
        assert!(check_user_tier(&request, UserTier::Free).unwrap());
        assert!(check_user_tier(&request, UserTier::Ultra).unwrap());
    }
}
