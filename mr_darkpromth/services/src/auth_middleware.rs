use axum::{
    extract::State,
    http::{header, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use mr_darkpromth_db::{User, UserTier};
use serde_json::json;
use std::sync::Arc;

use crate::{UserService, Claims, AuthError};

#[derive(Clone)]
pub struct AuthState {
    pub user_service: Arc<UserService>,
}

#[derive(Debug)]
pub struct AuthenticatedUser {
    pub id: uuid::Uuid,
    pub username: String,
    pub email: String,
    pub tier: UserTier,
}

pub async fn auth_middleware(
    State(state): State<AuthState>,
    mut request: Request<axum::body::Body>,
    next: Next<axum::body::Body>,
) -> Result<Response, StatusCode> {
    // Extract token from Authorization header
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|header| {
            if header.starts_with("Bearer ") {
                Some(&header[7..])
            } else {
                None
            }
        });

    let token = match auth_header {
        Some(token) => token,
        None => {
            return Ok((StatusCode::UNAUTHORIZED, Json(json!({
                "error": "Missing authorization header"
            }))).into_response());
        }
    };

    // Validate token
    let claims = match state.user_service.validate_token(token).await {
        Ok(claims) => claims,
        Err(AuthError::InvalidToken | AuthError::TokenExpired) => {
            return Ok((StatusCode::UNAUTHORIZED, Json(json!({
                "error": "Invalid or expired token"
            }))).into_response());
        }
        Err(_) => {
            return Ok((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "error": "Authentication failed"
            }))).into_response());
        }
    };

    // Parse user ID from claims
    let user_id = match uuid::Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return Ok((StatusCode::UNAUTHORIZED, Json(json!({
                "error": "Invalid user ID in token"
            }))).into_response());
        }
    };

    // Parse user tier from claims
    let user_tier = match claims.tier.as_str() {
        "free" => UserTier::Free,
        "ultra" => UserTier::Ultra,
        _ => UserTier::Free,
    };

    // Add authenticated user to request extensions
    let auth_user = AuthenticatedUser {
        id: user_id,
        username: claims.username,
        email: claims.email,
        tier: user_tier,
    };

    request.extensions_mut().insert(auth_user);

    Ok(next.run(request).await)
}

pub async fn api_key_middleware(
    State(state): State<AuthState>,
    mut request: Request<axum::body::Body>,
    next: Next<axum::body::Body>,
) -> Result<Response, StatusCode> {
    // Extract API key from X-API-Key header
    let api_key = request
        .headers()
        .get("X-API-Key")
        .and_then(|header| header.to_str().ok());

    let api_key = match api_key {
        Some(key) => key,
        None => {
            return Ok((StatusCode::UNAUTHORIZED, Json(json!({
                "error": "Missing API key header"
            }))).into_response());
        }
    };

    // Validate API key
    let user = match state.user_service.validate_api_key(api_key).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Ok((StatusCode::UNAUTHORIZED, Json(json!({
                "error": "Invalid or expired API key"
            }))).into_response());
        }
        Err(_) => {
            return Ok((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "error": "API key validation failed"
            }))).into_response());
        }
    };

    // Add authenticated user to request extensions
    let auth_user = AuthenticatedUser {
        id: user.id,
        username: user.username,
        email: user.email,
        tier: user.tier,
    };

    request.extensions_mut().insert(auth_user);

    Ok(next.run(request).await)
}

pub fn require_tier(required_tier: UserTier) -> impl Fn(Request<axum::body::Body>, Next<axum::body::Body>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, StatusCode>> + Send>> + Clone {
    move |request: Request<axum::body::Body>, next: Next<axum::body::Body>| {
        let required_tier = required_tier.clone();
        Box::pin(async move {
            // Extract authenticated user from request extensions
            let auth_user = match request.extensions().get::<AuthenticatedUser>() {
                Some(user) => user,
                None => {
                    return Ok((StatusCode::UNAUTHORIZED, Json(json!({
                        "error": "Authentication required"
                    }))).into_response());
                }
            };

            // Check tier permissions
            if !matches!((auth_user.tier.clone(), required_tier.clone()),
                (UserTier::Ultra, _) | // Ultra tier has access to everything
                (UserTier::Free, UserTier::Free) // Free tier can access free features
            ) {
                return Ok((StatusCode::FORBIDDEN, Json(json!({
                    "error": "Insufficient permissions",
                    "required_tier": format!("{:?}", required_tier),
                    "current_tier": format!("{:?}", auth_user.tier)
                }))).into_response());
            }

            Ok(next.run(request).await)
        })
    }
}

pub fn require_ultra_tier() -> impl Fn(Request<axum::body::Body>, Next<axum::body::Body>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, StatusCode>> + Send>> + Clone {
    require_tier(UserTier::Ultra)
}

pub fn require_free_tier() -> impl Fn(Request<axum::body::Body>, Next<axum::body::Body>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, StatusCode>> + Send>> + Clone {
    require_tier(UserTier::Free)
}

// Helper function to extract authenticated user from request
pub fn extract_auth_user(request: &Request<axum::body::Body>) -> Result<&AuthenticatedUser, StatusCode> {
    request.extensions().get::<AuthenticatedUser>()
        .ok_or(StatusCode::UNAUTHORIZED)
}

// Helper function to check if user has specific tier
pub fn check_user_tier(request: &Request<axum::body::Body>, required_tier: UserTier) -> Result<bool, StatusCode> {
    let auth_user = extract_auth_user(request)?;
    Ok(matches!((auth_user.tier.clone(), required_tier),
        (UserTier::Ultra, _) | // Ultra tier has access to everything
        (UserTier::Free, UserTier::Free) // Free tier can access free features
    ))
}

// Middleware for optional authentication (doesn't fail if no auth)
pub async fn optional_auth_middleware(
    State(state): State<AuthState>,
    mut request: Request<axum::body::Body>,
    next: Next<axum::body::Body>,
) -> Response {
    // Try to extract token from Authorization header
    if let Some(auth_header) = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|header| {
            if header.starts_with("Bearer ") {
                Some(&header[7..])
            } else {
                None
            }
        })
    {
        // Try to validate token
        if let Ok(claims) = state.user_service.validate_token(auth_header).await {
            if let Ok(user_id) = uuid::Uuid::parse_str(&claims.sub) {
                let user_tier = match claims.tier.as_str() {
                    "free" => UserTier::Free,
                    "ultra" => UserTier::Ultra,
                    _ => UserTier::Free,
                };

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
