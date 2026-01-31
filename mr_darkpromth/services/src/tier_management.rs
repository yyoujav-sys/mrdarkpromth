use anyhow::Result;
use chrono::{DateTime, Utc};
use mr_darkpromth_db::{User, UserRepository, UserTier, UpdateUserRequest};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct TierUpgradeRequest {
    pub user_id: Uuid,
    pub target_tier: UserTier,
    pub payment_method: Option<String>,
    pub promo_code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TierUpgradeResponse {
    pub user_id: Uuid,
    pub previous_tier: UserTier,
    pub new_tier: UserTier,
    pub upgraded_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub billing_cycle: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TierStats {
    pub total_users: i64,
    pub free_tier_users: i64,
    pub premium_tier_users: i64,
    pub ultra_tier_users: i64,
    pub recent_upgrades: i64,
    pub upgrade_rate: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TierBenefits {
    pub tier: UserTier,
    pub features: Vec<String>,
    pub limits: TierLimits,
    pub pricing: Option<TierPricing>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TierLimits {
    pub api_requests_per_day: i32,
    pub jailbreak_access: bool,
    pub concurrent_sessions: i32,
    pub storage_mb: i32,
    pub support_level: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TierPricing {
    pub monthly_price: f64,
    pub yearly_price: f64,
    pub currency: String,
    pub trial_days: i32,
}

#[derive(Debug, thiserror::Error)]
pub enum TierManagementError {
    #[error("User not found")]
    UserNotFound,
    #[error("Invalid tier transition")]
    InvalidTierTransition,
    #[error("Payment required")]
    PaymentRequired,
    #[error("Promo code invalid")]
    PromoCodeInvalid,
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Already at target tier")]
    AlreadyAtTargetTier,
    #[error("Generic error: {0}")]
    GenericError(String),
}

pub struct TierManagementService {
    repository: UserRepository,
}

impl TierManagementService {
    pub fn new(repository: UserRepository) -> Self {
        Self { repository }
    }

    pub async fn get_user_tier(&self, user_id: &str) -> Result<UserTier, TierManagementError> {
        let user_id = Uuid::parse_str(user_id).map_err(|_| TierManagementError::UserNotFound)?;
        let user = self
            .repository
            .get_user_by_id(user_id)
            .await
            .map_err(|e| TierManagementError::GenericError(e.to_string()))?
            .ok_or(TierManagementError::UserNotFound)?;
        Ok(user.tier)
    }

    pub async fn upgrade_tier(&self, request: TierUpgradeRequest) -> Result<TierUpgradeResponse, TierManagementError> {
        // Get current user
        let user = self.repository.get_user_by_id(request.user_id)
            .await
            .map_err(|e| TierManagementError::GenericError(e.to_string()))?
            .ok_or(TierManagementError::UserNotFound)?;

        // Validate tier transition
        if !self.validate_tier_transition(&user.tier, &request.target_tier) {
            return Err(TierManagementError::InvalidTierTransition);
        }

        // Check if already at target tier
        if user.tier == request.target_tier {
            return Err(TierManagementError::AlreadyAtTargetTier);
        }

        // Process payment if required (for Premium or Ultra tier)
        if matches!(request.target_tier, UserTier::Premium | UserTier::Ultra) {
            self.process_payment(&request).await?;
        }

        // Update user tier
        let update_request = UpdateUserRequest {
            username: None,
            email: None,
            tier: Some(request.target_tier.clone()),
            is_active: None,
        };

        let updated_user = self.repository.update_user(request.user_id, update_request)
            .await
            .map_err(|e| TierManagementError::GenericError(e.to_string()))?
            .ok_or(TierManagementError::UserNotFound)?;

        Ok(TierUpgradeResponse {
            user_id: request.user_id,
            previous_tier: user.tier,
            new_tier: request.target_tier,
            upgraded_at: Utc::now(),
            expires_at: None, // Ultra tier doesn't expire unless explicitly downgraded
            billing_cycle: Some("monthly".to_string()),
        })
    }

    pub async fn downgrade_tier(&self, user_id: Uuid, target_tier: UserTier) -> Result<TierUpgradeResponse, TierManagementError> {
        // Get current user
        let user = self.repository.get_user_by_id(user_id)
            .await
            .map_err(|e| TierManagementError::GenericError(e.to_string()))?
            .ok_or(TierManagementError::UserNotFound)?;

        // Validate tier transition (only allow downgrading)
        if !self.validate_downgrade_transition(&user.tier, &target_tier) {
            return Err(TierManagementError::InvalidTierTransition);
        }

        // Update user tier
        let update_request = UpdateUserRequest {
            username: None,
            email: None,
            tier: Some(target_tier.clone()),
            is_active: None,
        };

        let updated_user = self.repository.update_user(user_id, update_request)
            .await
            .map_err(|e| TierManagementError::GenericError(e.to_string()))?
            .ok_or(TierManagementError::UserNotFound)?;

        Ok(TierUpgradeResponse {
            user_id,
            previous_tier: user.tier,
            new_tier: target_tier,
            upgraded_at: Utc::now(),
            expires_at: None,
            billing_cycle: None,
        })
    }

    pub async fn get_tier_stats(&self) -> Result<TierStats, TierManagementError> {
        let all_users = self.repository.list_users(10000, 0)
            .await
            .map_err(|e| TierManagementError::GenericError(e.to_string()))?;
        
        let total_users = all_users.len() as i64;
        let free_tier_users = all_users.iter().filter(|u| u.tier == UserTier::Free).count() as i64;
        let premium_tier_users = all_users.iter().filter(|u| u.tier == UserTier::Premium).count() as i64;
        let ultra_tier_users = all_users.iter().filter(|u| u.tier == UserTier::Ultra).count() as i64;
        
        // Calculate recent upgrades (last 30 days)
        let thirty_days_ago = Utc::now() - chrono::Duration::days(30);
        let recent_upgrades = all_users.iter()
            .filter(|u| u.updated_at > thirty_days_ago && u.tier != UserTier::Free)
            .count() as i64;

        let paid_tier_users = premium_tier_users + ultra_tier_users;
        let upgrade_rate = if paid_tier_users > 0 {
            recent_upgrades as f64 / paid_tier_users as f64
        } else {
            0.0
        };

        Ok(TierStats {
            total_users,
            free_tier_users,
            premium_tier_users,
            ultra_tier_users,
            recent_upgrades,
            upgrade_rate,
        })
    }

    pub async fn get_tier_benefits(&self, tier: UserTier) -> TierBenefits {
        match tier {
            UserTier::Free => TierBenefits {
                tier,
                features: vec![
                    "Basic AI chat".to_string(),
                    "100 API requests per day".to_string(),
                    "Email support".to_string(),
                    "Standard response time".to_string(),
                ],
                limits: TierLimits {
                    api_requests_per_day: 100,
                    jailbreak_access: false,
                    concurrent_sessions: 1,
                    storage_mb: 100,
                    support_level: "Email".to_string(),
                },
                pricing: None,
            },
            UserTier::Premium => TierBenefits {
                tier,
                features: vec![
                    "Advanced AI chat".to_string(),
                    "1,000 API requests per day".to_string(),
                    "Priority email support".to_string(),
                    "Faster response time".to_string(),
                    "Advanced tools access".to_string(),
                ],
                limits: TierLimits {
                    api_requests_per_day: 1000,
                    jailbreak_access: false,
                    concurrent_sessions: 3,
                    storage_mb: 1000,
                    support_level: "Priority Email".to_string(),
                },
                pricing: Some(TierPricing {
                    monthly_price: 9.99,
                    yearly_price: 99.99,
                    currency: "USD".to_string(),
                    trial_days: 7,
                }),
            },
            UserTier::Ultra => TierBenefits {
                tier,
                features: vec![
                    "Unlimited AI chat".to_string(),
                    "Unlimited API requests".to_string(),
                    "Jailbreak access".to_string(),
                    "Priority support".to_string(),
                    "Advanced AI models".to_string(),
                    "Custom prompts".to_string(),
                    "API key management".to_string(),
                    "Concurrent sessions".to_string(),
                ],
                limits: TierLimits {
                    api_requests_per_day: -1, // Unlimited
                    jailbreak_access: true,
                    concurrent_sessions: 5,
                    storage_mb: 10000,
                    support_level: "Priority".to_string(),
                },
                pricing: Some(TierPricing {
                    monthly_price: 29.99,
                    yearly_price: 299.99,
                    currency: "USD".to_string(),
                    trial_days: 7,
                }),
            },
        }
    }

    pub async fn get_user_tier_history(&self, user_id: Uuid) -> Result<Vec<TierUpgradeResponse>, TierManagementError> {
        // This would typically query a separate tier_history table
        // For now, return current tier as the only entry
        let user = self.repository.get_user_by_id(user_id)
            .await
            .map_err(|e| TierManagementError::GenericError(e.to_string()))?
            .ok_or(TierManagementError::UserNotFound)?;

        Ok(vec![TierUpgradeResponse {
            user_id,
            previous_tier: UserTier::Free, // Assume started as Free
            new_tier: user.tier,
            upgraded_at: user.created_at,
            expires_at: None,
            billing_cycle: None,
        }])
    }

    pub async fn check_tier_eligibility(&self, user_id: Uuid, target_tier: UserTier) -> Result<bool, TierManagementError> {
        let user = self.repository.get_user_by_id(user_id)
            .await
            .map_err(|e| TierManagementError::GenericError(e.to_string()))?
            .ok_or(TierManagementError::UserNotFound)?;

        // Check if user is already at or above target tier
        match (&user.tier, &target_tier) {
            (UserTier::Ultra, _) => Ok(user.is_active), // Ultra tier has access to everything
            (UserTier::Premium, _) => Ok(user.is_active),
            (UserTier::Free, UserTier::Free) => Ok(user.is_active),
            (UserTier::Free, UserTier::Premium | UserTier::Ultra) => Ok(user.is_active),
        }
    }

    fn validate_tier_transition(&self, current_tier: &UserTier, target_tier: &UserTier) -> bool {
        match (current_tier, target_tier) {
            (UserTier::Free, UserTier::Premium) => true,
            (UserTier::Free, UserTier::Ultra) => true,
            (UserTier::Premium, UserTier::Free) => true,
            (UserTier::Premium, UserTier::Ultra) => true,
            (UserTier::Ultra, UserTier::Premium) => true,
            (UserTier::Ultra, UserTier::Free) => true,
            _ if current_tier == target_tier => false,
            _ => false,
        }
    }

    fn validate_downgrade_transition(&self, current_tier: &UserTier, target_tier: &UserTier) -> bool {
        match (current_tier, target_tier) {
            (UserTier::Ultra, UserTier::Premium) => true,
            (UserTier::Ultra, UserTier::Free) => true,
            (UserTier::Premium, UserTier::Free) => true,
            _ => false,
        }
    }

    async fn process_payment(&self, request: &TierUpgradeRequest) -> Result<(), TierManagementError> {
        // In a real implementation, this would integrate with a payment processor
        // For now, we'll simulate payment processing
        
        // Check for promo code
        if let Some(promo_code) = &request.promo_code {
            if !self.validate_promo_code(promo_code).await {
                return Err(TierManagementError::PromoCodeInvalid);
            }
        }

        // Simulate payment processing delay
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // In a real implementation, you would:
        // 1. Create payment intent with Stripe/PayPal
        // 2. Handle payment confirmation webhook
        // 3. Store payment record
        // 4. Apply promo code discount if applicable

        Ok(())
    }

    async fn validate_promo_code(&self, promo_code: &str) -> bool {
        // In a real implementation, this would check against a database of valid promo codes
        match promo_code.to_uppercase().as_str() {
            "LAUNCH2026" | "BETA50" | "EARLYADOPTER" => true,
            _ => false,
        }
    }

    pub async fn bulk_tier_update(&self, user_ids: Vec<Uuid>, target_tier: UserTier) -> Result<Vec<TierUpgradeResponse>, TierManagementError> {
        let mut results = Vec::new();

        for user_id in user_ids {
            match self.upgrade_tier(TierUpgradeRequest {
                user_id,
                target_tier: target_tier.clone(),
                payment_method: None,
                promo_code: None,
            }).await {
                Ok(response) => results.push(response),
                Err(_) => {
                    // Log error but continue with other users
                    // In a real implementation, you'd want to collect errors separately
                }
            }
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_tier_transition() {
        let service = TierManagementService::new(
            UserRepository::new(sqlx::PgPool::connect_lazy("postgresql://test").unwrap())
        );

        assert!(service.validate_tier_transition(&UserTier::Free, &UserTier::Ultra));
        assert!(service.validate_tier_transition(&UserTier::Ultra, &UserTier::Free));
        assert!(!service.validate_tier_transition(&UserTier::Free, &UserTier::Free));
        assert!(!service.validate_tier_transition(&UserTier::Ultra, &UserTier::Ultra));
    }

    #[tokio::test]
    async fn test_validate_promo_code() {
        let service = TierManagementService::new(
            UserRepository::new(sqlx::PgPool::connect_lazy("postgresql://test").unwrap())
        );

        assert!(service.validate_promo_code("LAUNCH2026").await);
        assert!(service.validate_promo_code("BETA50").await);
        assert!(!service.validate_promo_code("INVALID").await);
        assert!(!service.validate_promo_code("").await);
    }

    #[tokio::test]
    async fn test_get_tier_benefits() {
        let service = TierManagementService::new(
            UserRepository::new(sqlx::PgPool::connect_lazy("postgresql://test").unwrap())
        );

        let free_benefits = service.get_tier_benefits(UserTier::Free).await;
        assert_eq!(free_benefits.tier, UserTier::Free);
        assert!(!free_benefits.limits.jailbreak_access);
        assert_eq!(free_benefits.limits.api_requests_per_day, 100);

        let ultra_benefits = service.get_tier_benefits(UserTier::Ultra).await;
        assert_eq!(ultra_benefits.tier, UserTier::Ultra);
        assert!(ultra_benefits.limits.jailbreak_access);
        assert_eq!(ultra_benefits.limits.api_requests_per_day, -1);
        assert!(ultra_benefits.pricing.is_some());
    }
}
