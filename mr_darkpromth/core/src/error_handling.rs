// MR.DarkPromth Comprehensive Error Handling System
// Phase 2: Safety and Security Implementation

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum SecurityError {
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    #[error("Authorization denied: {0}")]
    AuthorizationDenied(String),
    
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),
    
    #[error("Input validation failed: {0}")]
    ValidationFailed(String),
    
    #[error("API key invalid: {0}")]
    ApiKeyInvalid(String),
    
    #[error("Session expired: {0}")]
    SessionExpired(String),
    
    #[error("Security policy violation: {0}")]
    SecurityPolicyViolation(String),
    
    #[error("Suspicious activity detected: {0}")]
    SuspiciousActivity(String),
    
    #[error("DDoS protection triggered: {0}")]
    DDoSProtection(String),
    
    #[error("Jailbreak safety violation: {0}")]
    JailbreakSafetyViolation(String),
    
    #[error("Server protection triggered: {0}")]
    ServerProtection(String),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Internal server error: {0}")]
    InternalServerError(String),
    
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
    
    #[error("Request timeout: {0}")]
    RequestTimeout(String),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("External service error: {0}")]
    ExternalServiceError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    pub request_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub session_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub endpoint: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub additional_data: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityErrorResponse {
    pub error_id: Uuid,
    pub error_type: String,
    pub message: String,
    pub details: Option<String>,
    pub should_retry: bool,
    pub retry_after_seconds: Option<u64>,
    pub context: ErrorContext,
    pub security_flags: SecurityFlags,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityFlags {
    pub is_security_related: bool,
    pub is_rate_limited: bool,
    pub is_blocked: bool,
    pub requires_authentication: bool,
    pub log_level: LogLevel,
    pub notify_admins: bool,
    pub escalate_priority: EscalationPriority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EscalationPriority {
    Low,
    Medium,
    High,
    Critical,
}

impl SecurityError {
    pub fn to_error_response(&self, context: ErrorContext) -> SecurityErrorResponse {
        let error_id = Uuid::new_v4();
        let error_type = self.get_error_type();
        let message = self.get_user_message();
        let details = self.get_detailed_message();
        let should_retry = self.should_retry();
        let retry_after = self.get_retry_after();
        let security_flags = self.get_security_flags();

        SecurityErrorResponse {
            error_id,
            error_type,
            message,
            details,
            should_retry,
            retry_after_seconds: retry_after,
            context,
            security_flags,
        }
    }

    fn get_error_type(&self) -> String {
        match self {
            SecurityError::AuthenticationFailed(_) => "AUTHENTICATION_FAILED".to_string(),
            SecurityError::AuthorizationDenied(_) => "AUTHORIZATION_DENIED".to_string(),
            SecurityError::RateLimitExceeded(_) => "RATE_LIMIT_EXCEEDED".to_string(),
            SecurityError::ValidationFailed(_) => "VALIDATION_FAILED".to_string(),
            SecurityError::ApiKeyInvalid(_) => "API_KEY_INVALID".to_string(),
            SecurityError::SessionExpired(_) => "SESSION_EXPIRED".to_string(),
            SecurityError::SecurityPolicyViolation(_) => "SECURITY_POLICY_VIOLATION".to_string(),
            SecurityError::SuspiciousActivity(_) => "SUSPICIOUS_ACTIVITY".to_string(),
            SecurityError::DDoSProtection(_) => "DDOS_PROTECTION".to_string(),
            SecurityError::JailbreakSafetyViolation(_) => "JAILBREAK_SAFETY_VIOLATION".to_string(),
            SecurityError::ServerProtection(_) => "SERVER_PROTECTION".to_string(),
            SecurityError::DatabaseError(_) => "DATABASE_ERROR".to_string(),
            SecurityError::InternalServerError(_) => "INTERNAL_SERVER_ERROR".to_string(),
            SecurityError::ServiceUnavailable(_) => "SERVICE_UNAVAILABLE".to_string(),
            SecurityError::RequestTimeout(_) => "REQUEST_TIMEOUT".to_string(),
            SecurityError::ConfigurationError(_) => "CONFIGURATION_ERROR".to_string(),
            SecurityError::ExternalServiceError(_) => "EXTERNAL_SERVICE_ERROR".to_string(),
        }
    }

    fn get_user_message(&self) -> String {
        match self {
            SecurityError::AuthenticationFailed(_) => "Authentication failed. Please check your credentials.".to_string(),
            SecurityError::AuthorizationDenied(_) => "Access denied. You don't have permission to perform this action.".to_string(),
            SecurityError::RateLimitExceeded(_) => "Too many requests. Please try again later.".to_string(),
            SecurityError::ValidationFailed(_) => "Invalid input provided. Please check your request.".to_string(),
            SecurityError::ApiKeyInvalid(_) => "Invalid API key provided.".to_string(),
            SecurityError::SessionExpired(_) => "Your session has expired. Please log in again.".to_string(),
            SecurityError::SecurityPolicyViolation(_) => "Request violates security policies.".to_string(),
            SecurityError::SuspiciousActivity(_) => "Suspicious activity detected. Request blocked.".to_string(),
            SecurityError::DDoSProtection(_) => "DDoS protection activated. Please try again later.".to_string(),
            SecurityError::JailbreakSafetyViolation(_) => "Jailbreak prompt violates safety guidelines.".to_string(),
            SecurityError::ServerProtection(_) => "Request blocked by server protection rules.".to_string(),
            SecurityError::DatabaseError(_) => "Database operation failed. Please try again.".to_string(),
            SecurityError::InternalServerError(_) => "Internal server error. Please try again later.".to_string(),
            SecurityError::ServiceUnavailable(_) => "Service temporarily unavailable. Please try again later.".to_string(),
            SecurityError::RequestTimeout(_) => "Request timed out. Please try again.".to_string(),
            SecurityError::ConfigurationError(_) => "Service configuration error. Please contact support.".to_string(),
            SecurityError::ExternalServiceError(_) => "External service error. Please try again later.".to_string(),
        }
    }

    fn get_detailed_message(&self) -> Option<String> {
        // Only provide detailed messages for non-security errors
        match self {
            SecurityError::DatabaseError(msg) => Some(format!("Database error: {}", msg)),
            SecurityError::InternalServerError(msg) => Some(format!("Internal error: {}", msg)),
            SecurityError::ServiceUnavailable(msg) => Some(format!("Service unavailable: {}", msg)),
            SecurityError::RequestTimeout(msg) => Some(format!("Timeout: {}", msg)),
            SecurityError::ConfigurationError(msg) => Some(format!("Configuration error: {}", msg)),
            SecurityError::ExternalServiceError(msg) => Some(format!("External service error: {}", msg)),
            SecurityError::ValidationFailed(msg) => Some(format!("Validation error: {}", msg)),
            _ => None, // Don't expose details for security-related errors
        }
    }

    fn should_retry(&self) -> bool {
        matches!(
            self,
            SecurityError::RateLimitExceeded(_)
                | SecurityError::ServiceUnavailable(_)
                | SecurityError::RequestTimeout(_)
                | SecurityError::ExternalServiceError(_)
                | SecurityError::DatabaseError(_)
                | SecurityError::InternalServerError(_)
        )
    }

    fn get_retry_after(&self) -> Option<u64> {
        match self {
            SecurityError::RateLimitExceeded(_) => Some(60), // 1 minute
            SecurityError::ServiceUnavailable(_) => Some(300), // 5 minutes
            SecurityError::RequestTimeout(_) => Some(30), // 30 seconds
            SecurityError::DDoSProtection(_) => Some(600), // 10 minutes
            _ => None,
        }
    }

    fn get_security_flags(&self) -> SecurityFlags {
        match self {
            SecurityError::AuthenticationFailed(_) => SecurityFlags {
                is_security_related: true,
                is_rate_limited: false,
                is_blocked: false,
                requires_authentication: true,
                log_level: LogLevel::Warn,
                notify_admins: false,
                escalate_priority: EscalationPriority::Low,
            },
            SecurityError::AuthorizationDenied(_) => SecurityFlags {
                is_security_related: true,
                is_rate_limited: false,
                is_blocked: false,
                requires_authentication: true,
                log_level: LogLevel::Warn,
                notify_admins: false,
                escalate_priority: EscalationPriority::Low,
            },
            SecurityError::RateLimitExceeded(_) => SecurityFlags {
                is_security_related: true,
                is_rate_limited: true,
                is_blocked: false,
                requires_authentication: false,
                log_level: LogLevel::Info,
                notify_admins: false,
                escalate_priority: EscalationPriority::Low,
            },
            SecurityError::SuspiciousActivity(_) => SecurityFlags {
                is_security_related: true,
                is_rate_limited: false,
                is_blocked: true,
                requires_authentication: false,
                log_level: LogLevel::Error,
                notify_admins: true,
                escalate_priority: EscalationPriority::High,
            },
            SecurityError::DDoSProtection(_) => SecurityFlags {
                is_security_related: true,
                is_rate_limited: true,
                is_blocked: true,
                requires_authentication: false,
                log_level: LogLevel::Error,
                notify_admins: true,
                escalate_priority: EscalationPriority::Critical,
            },
            SecurityError::JailbreakSafetyViolation(_) => SecurityFlags {
                is_security_related: true,
                is_rate_limited: false,
                is_blocked: true,
                requires_authentication: true,
                log_level: LogLevel::Warn,
                notify_admins: false,
                escalate_priority: EscalationPriority::Medium,
            },
            SecurityError::ServerProtection(_) => SecurityFlags {
                is_security_related: true,
                is_rate_limited: false,
                is_blocked: true,
                requires_authentication: false,
                log_level: LogLevel::Error,
                notify_admins: true,
                escalate_priority: EscalationPriority::High,
            },
            SecurityError::SecurityPolicyViolation(_) => SecurityFlags {
                is_security_related: true,
                is_rate_limited: false,
                is_blocked: true,
                requires_authentication: false,
                log_level: LogLevel::Warn,
                notify_admins: false,
                escalate_priority: EscalationPriority::Medium,
            },
            SecurityError::DatabaseError(_) => SecurityFlags {
                is_security_related: false,
                is_rate_limited: false,
                is_blocked: false,
                requires_authentication: false,
                log_level: LogLevel::Error,
                notify_admins: false,
                escalate_priority: EscalationPriority::Medium,
            },
            SecurityError::InternalServerError(_) => SecurityFlags {
                is_security_related: false,
                is_rate_limited: false,
                is_blocked: false,
                requires_authentication: false,
                log_level: LogLevel::Error,
                notify_admins: false,
                escalate_priority: EscalationPriority::High,
            },
            _ => SecurityFlags {
                is_security_related: false,
                is_rate_limited: false,
                is_blocked: false,
                requires_authentication: false,
                log_level: LogLevel::Info,
                notify_admins: false,
                escalate_priority: EscalationPriority::Low,
            },
        }
    }

    pub fn get_http_status_code(&self) -> StatusCode {
        match self {
            SecurityError::AuthenticationFailed(_) => StatusCode::UNAUTHORIZED,
            SecurityError::AuthorizationDenied(_) => StatusCode::FORBIDDEN,
            SecurityError::RateLimitExceeded(_) => StatusCode::TOO_MANY_REQUESTS,
            SecurityError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            SecurityError::ApiKeyInvalid(_) => StatusCode::UNAUTHORIZED,
            SecurityError::SessionExpired(_) => StatusCode::UNAUTHORIZED,
            SecurityError::SecurityPolicyViolation(_) => StatusCode::FORBIDDEN,
            SecurityError::SuspiciousActivity(_) => StatusCode::FORBIDDEN,
            SecurityError::DDoSProtection(_) => StatusCode::SERVICE_UNAVAILABLE,
            SecurityError::JailbreakSafetyViolation(_) => StatusCode::FORBIDDEN,
            SecurityError::ServerProtection(_) => StatusCode::FORBIDDEN,
            SecurityError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            SecurityError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            SecurityError::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            SecurityError::RequestTimeout(_) => StatusCode::REQUEST_TIMEOUT,
            SecurityError::ConfigurationError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            SecurityError::ExternalServiceError(_) => StatusCode::BAD_GATEWAY,
        }
    }
}

impl IntoResponse for SecurityError {
    fn into_response(self) -> Response {
        let context = ErrorContext {
            request_id: Some(Uuid::new_v4()),
            user_id: None,
            session_id: None,
            ip_address: None,
            user_agent: None,
            endpoint: None,
            timestamp: chrono::Utc::now(),
            additional_data: HashMap::new(),
        };

        let error_response = self.to_error_response(context);
        let status_code = self.get_http_status_code();

        // Log the error based on security flags
        self.log_error(&error_response);

        (status_code, Json(json!(error_response))).into_response()
    }
}

impl SecurityError {
    fn log_error(&self, error_response: &SecurityErrorResponse) {
        let log_level = match error_response.security_flags.log_level {
            LogLevel::Debug => log::Level::Debug,
            LogLevel::Info => log::Level::Info,
            LogLevel::Warn => log::Level::Warn,
            LogLevel::Error => log::Level::Error,
            LogLevel::Critical => log::Level::Error,
        };

        let log_message = format!(
            "SecurityError [{}]: {} | RequestID: {} | UserID: {:?} | IP: {:?}",
            error_response.error_type,
            error_response.message,
            error_response.error_id,
            error_response.context.user_id,
            error_response.context.ip_address
        );

        log::log!(log_level, "{}", log_message);

        // Notify admins if required
        if error_response.security_flags.notify_admins {
            self.notify_admins(error_response);
        }
    }

    fn notify_admins(&self, error_response: &SecurityErrorResponse) {
        // In a real implementation, this would send notifications to admins
        log::warn!(
            "ADMIN NOTIFICATION: Critical security event [{}] | ErrorID: {} | Priority: {:?}",
            error_response.error_type,
            error_response.error_id,
            error_response.security_flags.escalate_priority
        );
    }
}

pub struct ErrorHandler {
    enable_detailed_errors: bool,
    enable_error_tracking: bool,
    max_error_details_length: usize,
}

impl ErrorHandler {
    pub fn new(enable_detailed_errors: bool, enable_error_tracking: bool) -> Self {
        Self {
            enable_detailed_errors,
            enable_error_tracking,
            max_error_details_length: 1000,
        }
    }

    pub fn handle_error(&self, error: SecurityError, context: ErrorContext) -> SecurityErrorResponse {
        let mut error_response = error.to_error_response(context);

        // Apply error handling policies
        if !self.enable_detailed_errors {
            error_response.details = None;
        }

        // Truncate long details
        if let Some(details) = &mut error_response.details {
            if details.len() > self.max_error_details_length {
                *details = format!("{}... [truncated]", &details[..self.max_error_details_length]);
            }
        }

        // Track error if enabled
        if self.enable_error_tracking {
            self.track_error(&error_response);
        }

        error_response
    }

    fn track_error(&self, error_response: &SecurityErrorResponse) {
        // In a real implementation, this would track errors in a monitoring system
        log::info!(
            "Error tracking: {} | Type: {} | User: {:?} | IP: {:?}",
            error_response.error_id,
            error_response.error_type,
            error_response.context.user_id,
            error_response.context.ip_address
        );
    }

    pub fn create_error_context(
        &self,
        request_id: Option<Uuid>,
        user_id: Option<Uuid>,
        session_id: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
        endpoint: Option<String>,
    ) -> ErrorContext {
        ErrorContext {
            request_id,
            user_id,
            session_id,
            ip_address,
            user_agent,
            endpoint,
            timestamp: chrono::Utc::now(),
            additional_data: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_authentication_error_response() {
        let error = SecurityError::AuthenticationFailed("Invalid credentials".to_string());
        let context = ErrorContext {
            request_id: Some(Uuid::new_v4()),
            user_id: None,
            session_id: None,
            ip_address: Some("127.0.0.1".to_string()),
            user_agent: None,
            endpoint: Some("/api/auth/login".to_string()),
            timestamp: chrono::Utc::now(),
            additional_data: HashMap::new(),
        };

        let response = error.to_error_response(context);

        assert_eq!(response.error_type, "AUTHENTICATION_FAILED");
        assert!(response.message.contains("Authentication failed"));
        assert!(response.security_flags.is_security_related);
        assert!(response.security_flags.requires_authentication);
        assert!(!response.should_retry);
    }

    #[test]
    fn test_rate_limit_error_response() {
        let error = SecurityError::RateLimitExceeded("Too many requests".to_string());
        let context = ErrorContext {
            request_id: Some(Uuid::new_v4()),
            user_id: None,
            session_id: None,
            ip_address: Some("127.0.0.1".to_string()),
            user_agent: None,
            endpoint: Some("/api/chat".to_string()),
            timestamp: chrono::Utc::now(),
            additional_data: HashMap::new(),
        };

        let response = error.to_error_response(context);

        assert_eq!(response.error_type, "RATE_LIMIT_EXCEEDED");
        assert!(response.should_retry);
        assert_eq!(response.retry_after_seconds, Some(60));
        assert!(response.security_flags.is_rate_limited);
    }

    #[test]
    fn test_suspicious_activity_error_response() {
        let error = SecurityError::SuspiciousActivity("Unusual pattern detected".to_string());
        let context = ErrorContext {
            request_id: Some(Uuid::new_v4()),
            user_id: Some(Uuid::new_v4()),
            session_id: None,
            ip_address: Some("192.168.1.100".to_string()),
            user_agent: None,
            endpoint: Some("/api/admin/users".to_string()),
            timestamp: chrono::Utc::now(),
            additional_data: HashMap::new(),
        };

        let response = error.to_error_response(context);

        assert_eq!(response.error_type, "SUSPICIOUS_ACTIVITY");
        assert!(response.security_flags.is_blocked);
        assert!(response.security_flags.notify_admins);
        assert!(matches!(response.security_flags.escalate_priority, EscalationPriority::High));
    }

    #[test]
    fn test_error_handler() {
        let handler = ErrorHandler::new(false, true);
        let error = SecurityError::InternalServerError("Database connection failed".to_string());
        let context = handler.create_error_context(
            Some(Uuid::new_v4()),
            Some(Uuid::new_v4()),
            None,
            Some("127.0.0.1".to_string()),
            None,
            Some("/api/users".to_string()),
        );

        let response = handler.handle_error(error, context);

        // Detailed errors should be disabled
        assert!(response.details.is_none());
        assert_eq!(response.error_type, "INTERNAL_SERVER_ERROR");
    }

    #[test]
    fn test_http_status_codes() {
        assert_eq!(
            SecurityError::AuthenticationFailed("".to_string()).get_http_status_code(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            SecurityError::RateLimitExceeded("".to_string()).get_http_status_code(),
            StatusCode::TOO_MANY_REQUESTS
        );
        assert_eq!(
            SecurityError::ValidationFailed("".to_string()).get_http_status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            SecurityError::DDoSProtection("".to_string()).get_http_status_code(),
            StatusCode::SERVICE_UNAVAILABLE
        );
    }
}
