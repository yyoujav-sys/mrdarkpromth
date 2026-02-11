use async_trait::async_trait;
use anyhow::Result;
use uuid::Uuid;
use mr_darkpromth_core::{
    tier::{UserTier, TierLimits, PromptRequest},
};
use crate::audit::AuditLogger;

#[derive(Debug, Clone)]
pub struct TierValidationError {
    pub message: String,
    pub required_tier: Option<UserTier>,
    pub current_tier: UserTier,
}

impl std::fmt::Display for TierValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Tier validation failed: {}", self.message)
    }
}

impl std::error::Error for TierValidationError {}

#[async_trait]
pub trait TierValidator: Send + Sync {
    async fn validate_prompt_request(&self, request: &PromptRequest) -> Result<(), TierValidationError>;
    async fn validate_jailbreak_access(&self, user_id: Uuid, user_tier: UserTier) -> Result<(), TierValidationError>;
    async fn validate_concurrent_requests(&self, user_id: Uuid, user_tier: UserTier) -> Result<(), TierValidationError>;
    async fn get_user_tier(&self, user_id: Uuid) -> Result<UserTier, Box<dyn std::error::Error + Send + Sync>>;
}

pub struct DefaultTierValidator {
    audit_logger: AuditLogger,
}

impl DefaultTierValidator {
    pub fn new(audit_logger: AuditLogger) -> Self {
        Self { audit_logger }
    }

    fn validate_prompt_length(&self, prompt: &str, tier_limits: &TierLimits) -> Result<(), TierValidationError> {
        if prompt.len() > tier_limits.max_prompt_length {
            return Err(TierValidationError {
                message: format!(
                    "Prompt length {} exceeds maximum allowed {} for current tier",
                    prompt.len(),
                    tier_limits.max_prompt_length
                ),
                required_tier: None,
                current_tier: UserTier::Free, // This would be set by caller
            });
        }
        Ok(())
    }

    fn validate_jailbreak_permission(&self, jailbreak_enabled: bool, tier_limits: &TierLimits) -> Result<(), TierValidationError> {
        if jailbreak_enabled && !tier_limits.jailbreak_access {
            return Err(TierValidationError {
                message: "Jailbreak functionality requires Ultra tier".to_string(),
                required_tier: Some(UserTier::Ultra),
                current_tier: UserTier::Free, // This would be set by caller
            });
        }
        Ok(())
    }

    #[allow(dead_code)]
    fn validate_advanced_tools(&self, requires_advanced_tools: bool, tier_limits: &TierLimits) -> Result<(), TierValidationError> {
        if requires_advanced_tools && !tier_limits.advanced_tools {
            return Err(TierValidationError {
                message: "Advanced tools require Premium tier or higher".to_string(),
                required_tier: Some(UserTier::Premium),
                current_tier: UserTier::Free, // This would be set by caller
            });
        }
        Ok(())
    }
}

#[async_trait]
impl TierValidator for DefaultTierValidator {
    async fn validate_prompt_request(&self, request: &PromptRequest) -> Result<(), TierValidationError> {
        let tier_limits: TierLimits = request.user_tier.into();

        // Validate prompt length
        self.validate_prompt_length(&request.prompt, &tier_limits)?;

        // Validate jailbreak permission
        self.validate_jailbreak_permission(request.jailbreak_enabled, &tier_limits)?;

        // Log the validation attempt
        if let Err(e) = self.audit_logger.log_prompt_request(
            request.user_id,
            request.user_tier,
            request.id,
            &request.prompt,
            request.jailbreak_enabled,
            None, // IP address would be set by middleware
            None, // User agent would be set by middleware
        ).await {
            eprintln!("Failed to log audit: {}", e);
        }

        Ok(())
    }

    async fn validate_jailbreak_access(&self, user_id: Uuid, user_tier: UserTier) -> Result<(), TierValidationError> {
        let tier_limits: TierLimits = user_tier.into();

        if !tier_limits.jailbreak_access {
            let error = TierValidationError {
                message: "Jailbreak access is only available for Ultra tier users".to_string(),
                required_tier: Some(UserTier::Ultra),
                current_tier: user_tier,
            };

            // Log unauthorized jailbreak attempt
            if let Err(e) = self.audit_logger.log_unauthorized_access(
                Some(user_id),
                "jailbreak_access",
                None,
                None,
            ).await {
                eprintln!("Failed to log unauthorized access: {}", e);
            }

            return Err(error);
        }

        Ok(())
    }

    async fn validate_concurrent_requests(&self, _user_id: Uuid, user_tier: UserTier) -> Result<(), TierValidationError> {
        let tier_limits: TierLimits = user_tier.into();

        // In a real implementation, you would check the current number of concurrent requests
        // for this user from Redis or a database. For now, we'll just validate the tier limits.
        
        // This is a placeholder - actual implementation would check current concurrent requests
        let current_concurrent = 0; // This would be fetched from a cache/database

        if current_concurrent >= tier_limits.max_concurrent_requests {
            return Err(TierValidationError {
                message: format!(
                    "Concurrent request limit {} exceeded for current tier",
                    tier_limits.max_concurrent_requests
                ),
                required_tier: None,
                current_tier: user_tier,
            });
        }

        Ok(())
    }

    async fn get_user_tier(&self, _user_id: Uuid) -> Result<UserTier, Box<dyn std::error::Error + Send + Sync>> {
        // This would typically fetch from a database
        // For now, return Free as default
        Ok(UserTier::Free)
    }
}

pub struct TierMiddleware {
    validator: Box<dyn TierValidator>,
}

impl TierMiddleware {
    pub fn new(validator: Box<dyn TierValidator>) -> Self {
        Self { validator }
    }

    pub async fn validate_request(&self, request: &PromptRequest) -> Result<(), TierValidationError> {
        self.validator.validate_prompt_request(request).await
    }

    pub async fn validate_jailbreak(&self, user_id: Uuid, user_tier: UserTier) -> Result<(), TierValidationError> {
        self.validator.validate_jailbreak_access(user_id, user_tier).await
    }

    pub async fn validate_concurrent(&self, user_id: Uuid, user_tier: UserTier) -> Result<(), TierValidationError> {
        self.validator.validate_concurrent_requests(user_id, user_tier).await
    }

    pub async fn get_tier(&self, user_id: Uuid) -> Result<UserTier, Box<dyn std::error::Error + Send + Sync>> {
        self.validator.get_user_tier(user_id).await
    }
}

#[derive(Debug, Clone)]
pub struct TierConfig {
    pub enable_strict_validation: bool,
    pub log_all_validations: bool,
    pub cache_user_tiers: bool,
    pub tier_cache_ttl_seconds: u64,
}

impl Default for TierConfig {
    fn default() -> Self {
        TierConfig {
            enable_strict_validation: true,
            log_all_validations: true,
            cache_user_tiers: true,
            tier_cache_ttl_seconds: 300, // 5 minutes
        }
    }
}

pub fn create_tier_middleware(audit_logger: AuditLogger, _config: TierConfig) -> TierMiddleware {
    let validator = Box::new(DefaultTierValidator::new(audit_logger));
    TierMiddleware::new(validator)
}
