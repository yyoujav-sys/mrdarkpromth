// GitHub OAuth Integration Module
// For GitHub login and user management

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use reqwest::Client;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubUser {
    pub id: u64,
    pub login: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub location: Option<String>,
    pub company: Option<String>,
    pub blog: Option<String>,
    pub public_repos: u32,
    pub followers: u32,
    pub following: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubEmail {
    pub email: String,
    pub verified: bool,
    pub primary: bool,
    pub visibility: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubOAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub scope: String,
}

impl Default for GitHubOAuthConfig {
    fn default() -> Self {
        Self {
            client_id: std::env::var("GITHUB_CLIENT_ID")
                .unwrap_or_else(|_| "your_github_client_id".to_string()),
            client_secret: std::env::var("GITHUB_CLIENT_SECRET")
                .unwrap_or_else(|_| "your_github_client_secret".to_string()),
            redirect_uri: std::env::var("GITHUB_REDIRECT_URI")
                .unwrap_or_else(|_| "http://localhost:5173/auth/github/callback".to_string()),
            scope: "user:email".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthState {
    pub state: String,
    pub code_verifier: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubAuthResponse {
    pub access_token: String,
    pub token_type: String,
    pub scope: String,
}

pub struct GitHubOAuthClient {
    config: GitHubOAuthConfig,
    client: Client,
}

impl GitHubOAuthClient {
    pub fn new(config: GitHubOAuthConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }

    pub fn new_from_env() -> Self {
        Self::new(GitHubOAuthConfig::default())
    }

    /// Generate GitHub OAuth authorization URL
    pub fn get_authorization_url(&self, state: &str) -> String {
        format!(
            "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}&scope={}&state={}",
            urlencoding::encode(&self.config.client_id),
            urlencoding::encode(&self.config.redirect_uri),
            urlencoding::encode(&self.config.scope),
            urlencoding::encode(state)
        )
    }

    /// Exchange authorization code for access token
    pub async fn exchange_code_for_token(&self, code: &str, state: &str) -> Result<GitHubAuthResponse> {
        let mut params = HashMap::new();
        params.insert("client_id", self.config.client_id.as_str());
        params.insert("client_secret", self.config.client_secret.as_str());
        params.insert("code", code);
        params.insert("redirect_uri", self.config.redirect_uri.as_str());
        params.insert("state", state);

        let response: reqwest::Response = self.client
            .post("https://github.com/login/oauth/access_token")
            .header("Accept", "application/json")
            .form(&params)
            .send()
            .await?;

        if response.status().is_success() {
            let auth_response: GitHubAuthResponse = response.json().await?;
            Ok(auth_response)
        } else {
            Err(anyhow!("Failed to exchange code for token: {}", response.status()))
        }
    }

    /// Get user information from GitHub API
    pub async fn get_user_info(&self, access_token: &str) -> Result<GitHubUser> {
        let response: reqwest::Response = self.client
            .get("https://api.github.com/user")
            .header("Authorization", format!("Bearer {}", access_token))
            .header("User-Agent", "MR.DarkPromth")
            .send()
            .await?;

        if response.status().is_success() {
            let user: GitHubUser = response.json().await?;
            Ok(user)
        } else {
            Err(anyhow!("Failed to get user info: {}", response.status()))
        }
    }

    /// Get user emails from GitHub API
    pub async fn get_user_emails(&self, access_token: &str) -> Result<Vec<GitHubEmail>> {
        let response: reqwest::Response = self.client
            .get("https://api.github.com/user/emails")
            .header("Authorization", format!("Bearer {}", access_token))
            .header("User-Agent", "MR.DarkPromth")
            .send()
            .await?;

        if response.status().is_success() {
            let emails: Vec<GitHubEmail> = response.json().await?;
            Ok(emails)
        } else {
            Err(anyhow!("Failed to get user emails: {}", response.status()))
        }
    }

    /// Get primary verified email
    pub async fn get_primary_email(&self, access_token: &str) -> Result<String> {
        let emails = self.get_user_emails(access_token).await?;
        
        let primary_email = emails
            .into_iter()
            .find(|email| email.primary && email.verified)
            .map(|email| email.email)
            .ok_or_else(|| anyhow!("No primary verified email found"))?;

        Ok(primary_email)
    }

    /// Complete GitHub OAuth flow
    pub async fn complete_oauth_flow(&self, code: &str, state: &str) -> Result<(GitHubUser, String)> {
        // Exchange code for access token
        let auth_response = self.exchange_code_for_token(code, state).await?;
        
        // Get user information
        let user = self.get_user_info(&auth_response.access_token).await?;
        
        // Get primary email
        let email = self.get_primary_email(&auth_response.access_token).await?;
        
        // Create user with email
        let mut user_with_email = user;
        user_with_email.email = Some(email);
        
        Ok((user_with_email, auth_response.access_token))
    }

    /// Validate GitHub user tier based on criteria
    pub fn determine_user_tier(&self, user: &GitHubUser) -> String {
        // Ultra Tier criteria:
        // - 1000+ followers OR
        // - 100+ public repos OR
        // - Works at known tech company
        let known_companies = vec![
            "microsoft", "google", "amazon", "apple", "meta", "netflix",
            "twitter", "github", "openai", "anthropic", "nvidia", "intel",
            "adobe", "salesforce", "oracle", "ibm", "tesla", "spacex"
        ];

        let is_ultra = user.followers >= 1000 
            || user.public_repos >= 100
            || user.company.as_ref().map_or(false, |company| {
                let company_lower = company.to_lowercase();
                known_companies.iter().any(|known| company_lower.contains(known))
            });

        if is_ultra {
            "ultra".to_string()
        } else if user.followers >= 100 || user.public_repos >= 20 {
            "premium".to_string()
        } else {
            "free".to_string()
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubUserProfile {
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
    pub github_access_token: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<(GitHubUser, String)> for GitHubUserProfile {
    fn from((user, access_token): (GitHubUser, String)) -> Self {
        let email = user.email.clone().unwrap_or_default();
        Self {
            github_id: user.id,
            username: user.login,
            name: user.name,
            email,
            avatar_url: user.avatar_url,
            bio: user.bio,
            location: user.location,
            company: user.company,
            blog: user.blog,
            public_repos: user.public_repos,
            followers: user.followers,
            following: user.following,
            tier: "free".to_string(), // Will be determined by OAuth client
            github_access_token: access_token,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_user_tier() {
        let client = GitHubOAuthClient::new_from_env();
        
        // Test Ultra Tier user
        let ultra_user = GitHubUser {
            id: 1,
            login: "test".to_string(),
            name: Some("Test User".to_string()),
            email: Some("test@example.com".to_string()),
            avatar_url: None,
            bio: None,
            location: None,
            company: Some("Microsoft".to_string()),
            blog: None,
            public_repos: 50,
            followers: 1500,
            following: 100,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert_eq!(client.determine_user_tier(&ultra_user), "ultra");

        // Test Premium Tier user
        let premium_user = GitHubUser {
            followers: 150,
            public_repos: 25,
            ..ultra_user
        };

        assert_eq!(client.determine_user_tier(&premium_user), "premium");

        // Test Free Tier user
        let free_user = GitHubUser {
            followers: 50,
            public_repos: 5,
            ..ultra_user
        };

        assert_eq!(client.determine_user_tier(&free_user), "free");
    }
}
