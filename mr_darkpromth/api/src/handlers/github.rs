// GitHub OAuth Handlers for Axum
// Provides GitHub authentication integration

use axum::{
    extract::{State, Extension},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use mr_darkpromth_core::github_oauth::GitHubOAuthClient;
use mr_darkpromth_services::user_service::UserResponse;

use crate::AppState;
use crate::middleware::AuthenticatedUser;

#[derive(Debug, Deserialize)]
pub struct GitHubAuthRequest {
    pub code: String,
    pub state: String,
}

#[derive(Debug, Serialize)]
pub struct GitHubAuthUrlResponse {
    pub authorization_url: String,
    pub state: String,
}

#[derive(Debug, Serialize)]
pub struct GitHubCallbackResponse {
    pub status: String,
    pub user: UserResponse,
    pub github_profile: GitHubUserInfo,
    pub tier: String,
    pub token: Option<String>,
    pub refresh_token: Option<String>,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct GitHubUserInfo {
    pub login: String,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
    pub followers: u32,
    pub public_repos: u32,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

/// Get GitHub OAuth authorization URL
pub async fn github_auth_url_handler(
    State(state): State<Arc<AppState>>,
) -> Json<GitHubAuthUrlResponse> {
    let oauth_client = GitHubOAuthClient::new_from_env();
    let oauth_state = Uuid::new_v4().to_string();
    
    // Store state in Redis for CSRF validation (10 minute TTL)
    if let Some(redis_coordinator) = &state.redis_coordinator {
        let coordinator = redis_coordinator.lock().await;
        let key = format!("oauth_state:{}", oauth_state);
        let _ = coordinator.set(&key, "pending", Some(600)); // 10 min TTL
    }
    
    let authorization_url = oauth_client.get_authorization_url(&oauth_state);
    
    Json(GitHubAuthUrlResponse {
        authorization_url,
        state: oauth_state,
    })
}

/// Complete GitHub OAuth authentication callback
pub async fn github_auth_callback_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<GitHubAuthRequest>,
) -> Response {
    // Validate OAuth state parameter (CSRF protection)
    if let Some(redis_coordinator) = &state.redis_coordinator {
        let coordinator = redis_coordinator.lock().await;
        let key = format!("oauth_state:{}", request.state);
        match coordinator.get(&key) {
            Ok(Some(_)) => {
                // Valid state — consume it (one-time use)
                let _ = coordinator.del(&key);
            }
            _ => {
                log::warn!("Invalid or expired OAuth state parameter");
                let error = ErrorResponse {
                    error: "INVALID_STATE".to_string(),
                    message: "Invalid or expired OAuth state. Please try again.".to_string(),
                };
                return (StatusCode::BAD_REQUEST, Json(error)).into_response();
            }
        }
    }

    let oauth_client = GitHubOAuthClient::new_from_env();
    
    log::info!("GitHub OAuth callback received with code length: {}", request.code.len());
    
    match oauth_client.complete_oauth_flow(&request.code, &request.state).await {
        Ok((github_user, _access_token)) => {
            log::info!("Successfully authenticated GitHub user: {}", github_user.login);
            
            // Login or Register via UserService
            let email = github_user.email.clone().unwrap_or_else(|| format!("{}@github.com", github_user.login));
            
            match state.user_service.login_or_register_oauth(&email, &github_user.login).await {
                Ok(auth_response) => {
                     let tier = auth_response.user.tier.to_string();
                     let response = GitHubCallbackResponse {
                        status: "success".to_string(),
                        user: auth_response.user,
                        github_profile: GitHubUserInfo {
                            login: github_user.login,
                            email: github_user.email,
                            avatar_url: github_user.avatar_url,
                            followers: github_user.followers,
                            public_repos: github_user.public_repos,
                        },
                        tier,
                        token: Some(auth_response.token),
                        refresh_token: Some(auth_response.refresh_token),
                        message: "GitHub authentication successful".to_string(),
                    };
                    (StatusCode::OK, Json(response)).into_response()
                },
                Err(e) => {
                    log::error!("Failed to login/register GitHub user: {}", e);
                    let error = ErrorResponse {
                        error: "LOGIN_FAILED".to_string(),
                        message: format!("Failed to process user login: {}", e),
                    };
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
                }
            }
        }
        Err(e) => {
            log::error!("GitHub OAuth failed: {}", e);
            let error = ErrorResponse {
                error: "GITHUB_AUTH_FAILED".to_string(),
                message: format!("Failed to authenticate with GitHub: {}", e),
            };
            (StatusCode::BAD_REQUEST, Json(error)).into_response()
        }
    }
}

/// Link GitHub account to existing user (requires auth)
pub async fn github_link_handler(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(request): Json<GitHubAuthRequest>,
) -> Response {
    // Validate OAuth state parameter (CSRF protection)
    if let Some(redis_coordinator) = &state.redis_coordinator {
        let coordinator = redis_coordinator.lock().await;
        // Note: For linking, the state might be different or same flow. 
        // Assuming the frontend initiates a new OAuth flow for linking, it should use the same state mechanism.
        // If the frontend re-uses the same get_authorization_url, the state is stored in redis.
        let key = format!("oauth_state:{}", request.state);
        match coordinator.get(&key) {
            Ok(Some(_)) => {
                let _ = coordinator.del(&key);
            }
            _ => {
                let error = ErrorResponse {
                    error: "INVALID_STATE".to_string(),
                    message: "Invalid or expired OAuth state.".to_string(),
                };
                return (StatusCode::BAD_REQUEST, Json(error)).into_response();
            }
        }
    }

    let oauth_client = GitHubOAuthClient::new_from_env();
    
    match oauth_client.complete_oauth_flow(&request.code, &request.state).await {
        Ok((github_user, _access_token)) => {
            log::info!("Linking GitHub account: {} for user: {}", github_user.login, user.user_id);
            
            match state.user_service.link_github(user.user_id, github_user.login.clone()).await {
                Ok(_) => {
                    (StatusCode::OK, Json(serde_json::json!({
                        "message": "GitHub account linked successfully",
                        "github_username": github_user.login
                    }))).into_response()
                },
                Err(e) => {
                    log::error!("Failed to link GitHub account: {}", e);
                    let error = ErrorResponse {
                        error: "LINK_FAILED".to_string(),
                        message: format!("Failed to link GitHub account: {}", e),
                    };
                    (StatusCode::BAD_REQUEST, Json(error)).into_response()
                }
            }
        }
        Err(e) => {
            log::error!("GitHub linking failed: {}", e);
            let error = ErrorResponse {
                error: "GITHUB_LINK_FAILED".to_string(),
                message: format!("Failed to link GitHub account: {}", e),
            };
            (StatusCode::BAD_REQUEST, Json(error)).into_response()
        }
    }
}

/// Unlink GitHub account (requires auth)
pub async fn github_unlink_handler(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Response {
    log::info!("Unlinking GitHub account for user: {}", user.user_id);
    
    match state.user_service.unlink_github(user.user_id).await {
        Ok(_) => {
             Json(serde_json::json!({
                "message": "GitHub account unlinked successfully"
            })).into_response()
        },
        Err(e) => {
            log::error!("Failed to unlink GitHub account: {}", e);
            let error = ErrorResponse {
                error: "UNLINK_FAILED".to_string(),
                message: format!("Failed to unlink GitHub account: {}", e),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
    }
}

/// Get GitHub profile (requires auth)
pub async fn github_profile_handler(
    State(_state): State<Arc<AppState>>,
) -> Response {
    // TODO: Implement actual profile retrieval from database
    let error = ErrorResponse {
        error: "PROFILE_NOT_FOUND".to_string(),
        message: "GitHub account not linked".to_string(),
    };
    (StatusCode::NOT_FOUND, Json(error)).into_response()
}
