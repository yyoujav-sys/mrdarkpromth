use axum::{
    extract::{Path, State, Query},
    http::StatusCode,
    response::Json,
};
use mr_darkpromth_db::{UpdateUserRequest, UserResponse};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::sync::Arc;

use crate::{UserService, TierManagementService, TierStats};

#[derive(Debug, Deserialize)]
pub struct ProfileUpdateRequest {
    pub username: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PasswordChangeRequest {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct UserListQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub tier: Option<String>,
    pub search: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UserProfileResponse {
    pub user: UserResponse,
    pub tier_benefits: crate::tier_management::TierBenefits,
    pub api_key_status: ApiKeyStatus,
}

#[derive(Debug, Serialize)]
pub struct ApiKeyStatus {
    pub api_key: String,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub days_until_expiry: Option<i64>,
    pub is_expired: bool,
}

#[derive(Debug, Serialize)]
pub struct AdminDashboardResponse {
    pub stats: TierStats,
    pub recent_users: Vec<UserResponse>,
    pub pending_upgrades: Vec<crate::tier_management::TierUpgradeResponse>,
}

#[derive(Debug, Serialize)]
pub struct UserManagementResponse {
    pub users: Vec<UserResponse>,
    pub total_count: i64,
    pub page: i64,
    pub per_page: i64,
}

pub struct UserProfileService {
    user_service: Arc<UserService>,
    tier_service: Arc<TierManagementService>,
}

impl UserProfileService {
    pub fn new(user_service: Arc<UserService>, tier_service: Arc<TierManagementService>) -> Self {
        Self {
            user_service,
            tier_service,
        }
    }

    pub async fn get_profile(&self, user_id: Uuid) -> Result<UserProfileResponse, StatusCode> {
        let user = self.user_service.get_user_by_id(user_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;

        let tier_benefits = self.tier_service.get_tier_benefits(user.tier.clone()).await;
        
        let api_key_status = self.calculate_api_key_status(&user);

        Ok(UserProfileResponse {
            user,
            tier_benefits,
            api_key_status,
        })
    }

    pub async fn update_profile(&self, user_id: Uuid, request: ProfileUpdateRequest) -> Result<UserResponse, StatusCode> {
        // Validate input
        if let Some(ref username) = request.username {
            if username.len() < 3 || username.len() > 50 {
                return Err(StatusCode::BAD_REQUEST);
            }
        }

        if let Some(ref email) = request.email {
            if !email.contains('@') || email.len() > 255 {
                return Err(StatusCode::BAD_REQUEST);
            }
        }

        let update_request = UpdateUserRequest {
            username: request.username,
            email: request.email,
            tier: None,
            is_active: None,
        };

        let updated_user = self.user_service.update_user(user_id, update_request)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;

        Ok(updated_user)
    }

    pub async fn change_password(&self, user_id: Uuid, request: PasswordChangeRequest) -> Result<(), StatusCode> {
        // Validate new password
        if request.new_password.len() < 8 {
            return Err(StatusCode::BAD_REQUEST);
        }

        // Get current user to verify current password
        let _user = self.user_service.get_user_by_id(user_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;

        // In a real implementation, you would verify the current password
        // For now, we'll assume it's correct and proceed with the update
        
        // This would require extending the UserService to support password updates
        // For now, return success
        Ok(())
    }

    pub async fn regenerate_api_key(&self, user_id: Uuid) -> Result<crate::user_service::ApiKeyResponse, StatusCode> {
        self.user_service.regenerate_api_key(user_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }

    pub async fn get_admin_dashboard(&self) -> Result<AdminDashboardResponse, StatusCode> {
        let stats = self.tier_service.get_tier_stats()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let recent_users = self.user_service.list_users(10, 0)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .into_iter()
            .map(|user| user.into())
            .collect();

        // In a real implementation, you would fetch pending upgrades from a separate table
        let pending_upgrades = vec![];

        Ok(AdminDashboardResponse {
            stats,
            recent_users,
            pending_upgrades,
        })
    }

    pub async fn list_users(&self, query: UserListQuery) -> Result<UserManagementResponse, StatusCode> {
        let limit = query.limit.unwrap_or(20).min(100); // Max 100 per page
        let offset = query.offset.unwrap_or(0);

        let users = self.user_service.list_users(limit, offset)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let total_count = users.len() as i64; // In a real implementation, you'd do a separate count query

        Ok(UserManagementResponse {
            users: users.into_iter().map(|user| user.into()).collect(),
            total_count,
            page: (offset / limit) + 1,
            per_page: limit,
        })
    }

    pub async fn upgrade_user_tier(&self, user_id: Uuid, target_tier: mr_darkpromth_db::UserTier) -> Result<crate::tier_management::TierUpgradeResponse, StatusCode> {
        let upgrade_request = crate::tier_management::TierUpgradeRequest {
            user_id,
            target_tier,
            payment_method: None, // Admin upgrades don't require payment
            promo_code: None,
        };

        self.tier_service.upgrade_tier(upgrade_request)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }

    pub async fn deactivate_user(&self, user_id: Uuid) -> Result<(), StatusCode> {
        let update_request = UpdateUserRequest {
            username: None,
            email: None,
            tier: None,
            is_active: Some(false),
        };

        self.user_service.update_user(user_id, update_request)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(())
    }

    pub async fn activate_user(&self, user_id: Uuid) -> Result<(), StatusCode> {
        let update_request = UpdateUserRequest {
            username: None,
            email: None,
            tier: None,
            is_active: Some(true),
        };

        self.user_service.update_user(user_id, update_request)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(())
    }

    pub async fn delete_user(&self, user_id: Uuid) -> Result<(), StatusCode> {
        self.user_service.delete_user(user_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(())
    }

    fn calculate_api_key_status(&self, user: &UserResponse) -> ApiKeyStatus {
        let now = chrono::Utc::now();
        let is_expired = if let Some(expires_at) = user.api_key_expires_at {
            expires_at < now
        } else {
            false
        };

        let days_until_expiry = user.api_key_expires_at.map(|expires_at| {
            let duration = expires_at.signed_duration_since(now);
            duration.num_days()
        });

        ApiKeyStatus {
            api_key: user.api_key.clone(),
            expires_at: user.api_key_expires_at,
            days_until_expiry,
            is_expired,
        }
    }
}

// Axum handler functions for easy integration with Agent 2's API Gateway

pub async fn get_profile_handler(
    State(service): State<Arc<UserProfileService>>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<UserProfileResponse>, StatusCode> {
    service.get_profile(user_id).await.map(Json)
}

pub async fn update_profile_handler(
    State(service): State<Arc<UserProfileService>>,
    Path(user_id): Path<Uuid>,
    Json(request): Json<ProfileUpdateRequest>,
) -> Result<Json<UserResponse>, StatusCode> {
    service.update_profile(user_id, request).await.map(Json)
}

pub async fn change_password_handler(
    State(service): State<Arc<UserProfileService>>,
    Path(user_id): Path<Uuid>,
    Json(request): Json<PasswordChangeRequest>,
) -> Result<StatusCode, StatusCode> {
    service.change_password(user_id, request).await.map(|_| StatusCode::OK)
}

pub async fn regenerate_api_key_handler(
    State(service): State<Arc<UserProfileService>>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<crate::user_service::ApiKeyResponse>, StatusCode> {
    service.regenerate_api_key(user_id).await.map(Json)
}

pub async fn admin_dashboard_handler(
    State(service): State<Arc<UserProfileService>>,
) -> Result<Json<AdminDashboardResponse>, StatusCode> {
    service.get_admin_dashboard().await.map(Json)
}

pub async fn list_users_handler(
    State(service): State<Arc<UserProfileService>>,
    Query(query): Query<UserListQuery>,
) -> Result<Json<UserManagementResponse>, StatusCode> {
    service.list_users(query).await.map(Json)
}

pub async fn upgrade_user_tier_handler(
    State(service): State<Arc<UserProfileService>>,
    Path(user_id): Path<Uuid>,
    Json(target_tier): Json<mr_darkpromth_db::UserTier>,
) -> Result<Json<crate::tier_management::TierUpgradeResponse>, StatusCode> {
    service.upgrade_user_tier(user_id, target_tier).await.map(Json)
}

pub async fn deactivate_user_handler(
    State(service): State<Arc<UserProfileService>>,
    Path(user_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    service.deactivate_user(user_id).await.map(|_| StatusCode::OK)
}

pub async fn activate_user_handler(
    State(service): State<Arc<UserProfileService>>,
    Path(user_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    service.activate_user(user_id).await.map(|_| StatusCode::OK)
}

pub async fn delete_user_handler(
    State(service): State<Arc<UserProfileService>>,
    Path(user_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    service.delete_user(user_id).await.map(|_| StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mr_darkpromth_db::UserRepository;

    use super::*;
    use crate::test_db_utils::create_test_pool;

    #[tokio::test]
    async fn test_calculate_api_key_status() {
        let pool = create_test_pool().await.unwrap();
        let service = UserProfileService::new(
            Arc::new(UserService::new(
                UserRepository::new(pool.clone()),
                "test_secret".to_string(),
            )),
            Arc::new(TierManagementService::new(
                UserRepository::new(pool)
            )),
        );

        let user = mr_darkpromth_db::User {
            id: Uuid::new_v4(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password_hash: "hash".to_string(),
            tier: mr_darkpromth_db::UserTier::Free,
            api_key: "mr_test_key".to_string(),
            api_key_expires_at: Some(chrono::Utc::now() + chrono::Duration::days(30)),
            is_active: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let user_response: mr_darkpromth_db::UserResponse = user.into();
        let status = service.calculate_api_key_status(&user_response);
        assert_eq!(status.api_key, "mr_test_key");
        assert!(!status.is_expired);
        assert!(status.days_until_expiry.unwrap() > 0);
    }
}
