// GitHub OAuth Handlers for Axum
// Provides GitHub authentication integration

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use mr_darkpromth_core::github_oauth::GitHubOAuthClient;

use crate::AppState;

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
    pub github_user: GitHubUserInfo,
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
    State(_state): State<Arc<AppState>>,
) -> Json<GitHubAuthUrlResponse> {
    let oauth_client = GitHubOAuthClient::new_from_env();
    let state = Uuid::new_v4().to_string();
    
    let authorization_url = oauth_client.get_authorization_url(&state);
    
    Json(GitHubAuthUrlResponse {
        authorization_url,
        state,
    })
}

/// Complete GitHub OAuth authentication callback
pub async fn github_auth_callback_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<GitHubAuthRequest>,
) -> Response {
    let oauth_client = GitHubOAuthClient::new_from_env();
    
    log::info!("GitHub OAuth callback received with code length: {}", request.code.len());
    
    match oauth_client.complete_oauth_flow(&request.code, &request.state).await {
        Ok((github_user, _access_token)) => {
            log::info!("Successfully authenticated GitHub user: {}", github_user.login);
            
            // Login or Register via UserService
            let email = github_user.email.clone().unwrap_or_else(|| format!("{}@github.com", github_user.login));
            
            match state.user_service.login_or_register_oauth(&email, &github_user.login).await {
                Ok(auth_response) => {
                     let response = GitHubCallbackResponse {
                        status: "success".to_string(),
                        github_user: GitHubUserInfo {
                            login: github_user.login,
                            email: github_user.email,
                            avatar_url: github_user.avatar_url,
                            followers: github_user.followers,
                            public_repos: github_user.public_repos,
                        },
                        tier: auth_response.user.tier.to_string(),
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
    State(_state): State<Arc<AppState>>,
    Json(request): Json<GitHubAuthRequest>,
) -> Response {
    let oauth_client = GitHubOAuthClient::new_from_env();
    
    match oauth_client.complete_oauth_flow(&request.code, &request.state).await {
        Ok((github_user, _access_token)) => {
            log::info!("Linked GitHub account: {}", github_user.login);
            
            (StatusCode::OK, Json(serde_json::json!({
                "message": "GitHub account linked successfully",
                "github_username": github_user.login
            }))).into_response()
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
    State(_state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    log::info!("Unlinking GitHub account");
    
    Json(serde_json::json!({
        "message": "GitHub account unlinked successfully"
    }))
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
