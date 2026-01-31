// GitHub OAuth Routes
// For GitHub login integration

use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use std::sync::Arc;
use mr_darkpromth_core::github_oauth::{GitHubOAuthClient, GitHubUserProfile};
use crate::auth_middleware::AuthenticatedUser;
use log::{info, error, warn};

#[derive(Debug, Deserialize, ToSchema)]
pub struct GitHubAuthRequest {
    pub code: String,
    pub state: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct GitHubAuthUrlResponse {
    pub authorization_url: String,
    pub state: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct GitHubAuthResponse {
    pub user: GitHubUserResponse,
    pub token: String,
    pub expires_in: i64,
    pub github_profile: GitHubProfileResponse,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct GitHubUserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub tier: String,
    pub avatar_url: Option<String>,
    pub github_username: String,
    pub github_followers: u32,
    pub github_repos: u32,
    pub created_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct GitHubProfileResponse {
    pub github_id: u64,
    pub username: String,
    pub name: Option<String>,
    pub email: String,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub location: Option<String>,
    pub company: Option<String>,
    pub blog: Option<String>,
    pub public_repos: u32,
    pub followers: u32,
    pub following: u32,
    pub tier: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct GitHubAuthErrorResponse {
    pub error: String,
    pub message: String,
}

/// Get GitHub OAuth authorization URL
#[utoipa::path(
    get,
    path = "/api/auth/github/url",
    responses(
        (status = 200, description = "GitHub authorization URL generated", body = GitHubAuthUrlResponse),
        (status = 500, description = "Internal server error", body = GitHubAuthErrorResponse)
    ),
    tag = "github-auth"
)]
pub async fn get_github_auth_url() -> impl Responder {
    let oauth_client = GitHubOAuthClient::new_from_env();
    let state = Uuid::new_v4().to_string();
    
    let authorization_url = oauth_client.get_authorization_url(&state);
    
    HttpResponse::Ok().json(GitHubAuthUrlResponse {
        authorization_url,
        state,
    })
}

/// Complete GitHub OAuth authentication
#[utoipa::path(
    post,
    path = "/api/auth/github/callback",
    request_body = GitHubAuthRequest,
    responses(
        (status = 200, description = "GitHub authentication successful", body = GitHubAuthResponse),
        (status = 400, description = "Invalid GitHub credentials", body = GitHubAuthErrorResponse),
        (status = 500, description = "Internal server error", body = GitHubAuthErrorResponse)
    ),
    tag = "github-auth"
)]
pub async fn github_auth_callback(
    request: web::Json<GitHubAuthRequest>,
) -> impl Responder {
    let oauth_client = GitHubOAuthClient::new_from_env();
    
    info!("GitHub OAuth callback received with code length: {}", request.code.len());
    
    match oauth_client.complete_oauth_flow(&request.code, &request.state).await {
        Ok((github_user, access_token)) => {
            info!("Successfully authenticated GitHub user: {}", github_user.login);
            
            // Determine user tier based on GitHub profile
            let tier = oauth_client.determine_user_tier(&github_user);
            
            // Create GitHub profile
            let mut github_profile = GitHubUserProfile::from((github_user.clone(), access_token.clone()));
            github_profile.tier = tier.clone();
            
            // Create or update user in database (this would need to be implemented)
            let user_id = create_or_update_github_user(&github_profile).await?;
            
            // Generate JWT token
            let token = generate_jwt_token(user_id, &tier)?;
            
            let user_response = GitHubUserResponse {
                id: user_id,
                username: github_user.login.clone(),
                email: github_user.email.clone().unwrap_or_default(),
                tier: tier.clone(),
                avatar_url: github_user.avatar_url,
                github_username: github_user.login,
                github_followers: github_user.followers,
                github_repos: github_user.public_repos,
                created_at: chrono::Utc::now().to_rfc3339(),
            };
            
            let profile_response = GitHubProfileResponse {
                github_id: github_profile.github_id,
                username: github_profile.username,
                name: github_profile.name,
                email: github_profile.email,
                avatar_url: github_profile.avatar_url,
                bio: github_profile.bio,
                location: github_profile.location,
                company: github_profile.company,
                blog: github_profile.blog,
                public_repos: github_profile.public_repos,
                followers: github_profile.followers,
                following: github_profile.following,
                tier: github_profile.tier,
            };
            
            HttpResponse::Ok().json(GitHubAuthResponse {
                user: user_response,
                token,
                expires_in: 86400, // 24 hours
                github_profile: profile_response,
            })
        }
        Err(e) => {
            error!("GitHub OAuth failed: {}", e);
            HttpResponse::BadRequest().json(GitHubAuthErrorResponse {
                error: "GitHub Authentication Error".to_string(),
                message: format!("Failed to authenticate with GitHub: {}", e),
            })
        }
    }
}

/// Link GitHub account to existing user
#[utoipa::path(
    post,
    path = "/api/auth/github/link",
    request_body = GitHubAuthRequest,
    responses(
        (status = 200, description = "GitHub account linked successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 400, description = "Invalid GitHub credentials")
    ),
    tag = "github-auth"
)]
pub async fn link_github_account(
    auth_user: AuthenticatedUser,
    request: web::Json<GitHubAuthRequest>,
) -> impl Responder {
    let oauth_client = GitHubOAuthClient::new_from_env();
    
    match oauth_client.complete_oauth_flow(&request.code, &request.state).await {
        Ok((github_user, access_token)) => {
            // Link GitHub account to existing user
            let result = link_github_to_user(auth_user.user_id, &github_user, &access_token).await?;
            
            if result {
                HttpResponse::Ok().json(serde_json::json!({
                    "message": "GitHub account linked successfully",
                    "github_username": github_user.login
                }))
            } else {
                HttpResponse::BadRequest().json(GitHubAuthErrorResponse {
                    error: "Link Failed".to_string(),
                    message: "Failed to link GitHub account".to_string(),
                })
            }
        }
        Err(e) => {
            error!("GitHub linking failed: {}", e);
            HttpResponse::BadRequest().json(GitHubAuthErrorResponse {
                error: "GitHub Link Error".to_string(),
                message: format!("Failed to link GitHub account: {}", e),
            })
        }
    }
}

/// Unlink GitHub account
#[utoipa::path(
    delete,
    path = "/api/auth/github/unlink",
    responses(
        (status = 200, description = "GitHub account unlinked successfully"),
        (status = 401, description = "Unauthorized")
    ),
    tag = "github-auth"
)]
pub async fn unlink_github_account(
    auth_user: AuthenticatedUser,
) -> impl Responder {
    match unlink_github_from_user(auth_user.user_id).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "message": "GitHub account unlinked successfully"
        })),
        Err(e) => {
            error!("GitHub unlinking failed: {}", e);
            HttpResponse::BadRequest().json(GitHubAuthErrorResponse {
                error: "Unlink Failed".to_string(),
                message: format!("Failed to unlink GitHub account: {}", e),
            })
        }
    }
}

/// Get GitHub profile information
#[utoipa::path(
    get,
    path = "/api/auth/github/profile",
    responses(
        (status = 200, description = "GitHub profile retrieved successfully", body = GitHubProfileResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "GitHub account not linked")
    ),
    tag = "github-auth"
)]
pub async fn get_github_profile(
    auth_user: AuthenticatedUser,
) -> impl Responder {
    match get_user_github_profile(auth_user.user_id).await {
        Ok(profile) => HttpResponse::Ok().json(profile),
        Err(_) => HttpResponse::NotFound().json(GitHubAuthErrorResponse {
            error: "Profile Not Found".to_string(),
            message: "GitHub account not linked".to_string(),
        }),
    }
}

// Helper functions (these would need to be implemented with actual database operations)

async fn create_or_update_github_user(profile: &GitHubUserProfile) -> Result<Uuid, anyhow::Error> {
    // This would create or update user in database
    // For now, return a mock UUID
    Ok(Uuid::new_v4())
}

fn generate_jwt_token(user_id: Uuid, tier: &str) -> Result<String, anyhow::Error> {
    use jsonwebtoken::{encode, EncodingKey, Header};
    
    let claims = serde_json::json!({
        "sub": user_id.to_string(),
        "exp": chrono::Utc::now().timestamp() + 86400,
        "iat": chrono::Utc::now().timestamp(),
        "tier": tier,
        "permissions": get_permissions_for_tier(tier)
    });
    
    let secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "your-super-secret-jwt-key".to_string());
    
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref()))
        .map_err(|e| anyhow::anyhow!("Failed to generate token: {}", e))
}

fn get_permissions_for_tier(tier: &str) -> Vec<String> {
    match tier {
        "ultra" => vec![
            "chat:unlimited".to_string(),
            "jailbreak:access".to_string(),
            "code:generate".to_string(),
            "admin:access".to_string(),
        ],
        "premium" => vec![
            "chat:extended".to_string(),
            "code:generate".to_string(),
        ],
        _ => vec![
            "chat:basic".to_string(),
        ],
    }
}

async fn link_github_to_user(user_id: Uuid, github_user: &mr_darkpromth_core::github_oauth::GitHubUser, access_token: &str) -> Result<bool, anyhow::Error> {
    // This would link GitHub account to existing user in database
    // For now, return true
    Ok(true)
}

async fn unlink_github_from_user(user_id: Uuid) -> Result<(), anyhow::Error> {
    // This would unlink GitHub account from user in database
    // For now, do nothing
    Ok(())
}

async fn get_user_github_profile(user_id: Uuid) -> Result<GitHubProfileResponse, anyhow::Error> {
    // This would get GitHub profile from database
    // For now, return error
    Err(anyhow::anyhow!("Not implemented"))
}
