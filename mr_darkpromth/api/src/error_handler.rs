// Standardized error handling for the API
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use chrono::Utc;
use uuid::Uuid;

/// Standardized API error response
#[derive(Debug, Clone, Serialize)]
pub struct ApiError {
    /// Unique request ID for tracing
    #[serde(rename = "request_id")]
    pub request_id: String,
    
    /// Error code for client-side handling
    #[serde(rename = "error_code")]
    pub error_code: String,
    
    /// Human-readable error message
    pub message: String,
    
    /// Additional context (only in development)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    
    /// When the error occurred
    pub timestamp: String,
}

impl ApiError {
    /// Create a new API error
    pub fn new(error_code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            request_id: Uuid::new_v4().to_string(),
            error_code: error_code.into(),
            message: message.into(),
            details: None,
            timestamp: Utc::now().to_rfc3339(),
        }
    }

    /// Add details to the error
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    /// Get the HTTP status code for this error
    pub fn status_code(&self) -> StatusCode {
        match self.error_code.as_str() {
            "AUTH_FAILED" | "INVALID_CREDENTIALS" | "INVALID_PASSWORD" => StatusCode::UNAUTHORIZED,
            "AUTH_REQUIRED" | "MISSING_TOKEN" | "INVALID_TOKEN" => StatusCode::UNAUTHORIZED,
            "PERMISSION_DENIED" | "PREMIUM_REQUIRED" | "ULTRA_OR_ADMIN_REQUIRED" | "FORBIDDEN" => StatusCode::FORBIDDEN,
            "NOT_FOUND" | "USER_NOT_FOUND" => StatusCode::NOT_FOUND,
            "VALIDATION_ERROR" | "INVALID_INPUT" | "INVALID_UUID" => StatusCode::BAD_REQUEST,
            "REGISTRATION_FAILED" | "VERIFICATION_FAILED" | "WEAK_PASSWORD" => StatusCode::BAD_REQUEST,
            "CONFLICT" => StatusCode::CONFLICT,
            "RATE_LIMITED" => StatusCode::TOO_MANY_REQUESTS,
            "INTERNAL_ERROR" | "UPDATE_FAILED" | "TOKEN_FAILED" | "REGENERATION_FAILED" 
            | "VERIFICATION_ERROR" | "FILE_SYSTEM_ERROR" | "INVALID_USER_ID" => StatusCode::INTERNAL_SERVER_ERROR,
            "SERVICE_UNAVAILABLE" => StatusCode::SERVICE_UNAVAILABLE,
            "DATABASE_ERROR" => StatusCode::INTERNAL_SERVER_ERROR,
            "EXTERNAL_SERVICE_ERROR" => StatusCode::BAD_GATEWAY,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

/// Common error codes
pub mod codes {
    pub const AUTH_FAILED: &str = "AUTH_FAILED";
    pub const AUTH_REQUIRED: &str = "AUTH_REQUIRED";
    pub const INVALID_CREDENTIALS: &str = "INVALID_CREDENTIALS";
    pub const INVALID_TOKEN: &str = "INVALID_TOKEN";
    pub const MISSING_TOKEN: &str = "MISSING_TOKEN";
    pub const PERMISSION_DENIED: &str = "PERMISSION_DENIED";
    pub const NOT_FOUND: &str = "NOT_FOUND";
    pub const USER_NOT_FOUND: &str = "USER_NOT_FOUND";
    pub const VALIDATION_ERROR: &str = "VALIDATION_ERROR";
    pub const INVALID_INPUT: &str = "INVALID_INPUT";
    pub const INVALID_UUID: &str = "INVALID_UUID";
    pub const CONFLICT: &str = "CONFLICT";
    pub const RATE_LIMITED: &str = "RATE_LIMITED";
    pub const INTERNAL_ERROR: &str = "INTERNAL_ERROR";
    pub const REGISTRATION_FAILED: &str = "REGISTRATION_FAILED";
    pub const UPDATE_FAILED: &str = "UPDATE_FAILED";
    pub const TOKEN_FAILED: &str = "TOKEN_FAILED";
    pub const REGENERATION_FAILED: &str = "REGENERATION_FAILED";
    pub const VERIFICATION_FAILED: &str = "VERIFICATION_FAILED";
    pub const VERIFICATION_ERROR: &str = "VERIFICATION_ERROR";
    pub const SERVICE_UNAVAILABLE: &str = "SERVICE_UNAVAILABLE";
    pub const DATABASE_ERROR: &str = "DATABASE_ERROR";
    pub const EXTERNAL_SERVICE_ERROR: &str = "EXTERNAL_SERVICE_ERROR";
    pub const EMAIL_VERIFICATION_FAILED: &str = "EMAIL_VERIFICATION_FAILED";
    pub const WEAK_PASSWORD: &str = "WEAK_PASSWORD";
    pub const ACCOUNT_LOCKED: &str = "ACCOUNT_LOCKED";
    pub const INVALID_PASSWORD: &str = "INVALID_PASSWORD";
    pub const FILE_SYSTEM_ERROR: &str = "FILE_SYSTEM_ERROR";
    pub const INVALID_USER_ID: &str = "INVALID_USER_ID";
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        
        // Log the error
        log::error!(
            "API Error [{}] {}: {} (request_id: {})",
            status.as_u16(),
            self.error_code,
            self.message,
            self.request_id
        );

        (status, Json(self)).into_response()
    }
}

/// Result type for API handlers
pub type ApiResult<T> = Result<T, ApiError>;

/// Create a success response with data
#[derive(Debug, Serialize)]
pub struct ApiSuccess<T: Serialize> {
    pub data: T,
    pub request_id: String,
    pub timestamp: String,
}

impl<T: Serialize> ApiSuccess<T> {
    pub fn new(data: T) -> Self {
        Self {
            data,
            request_id: Uuid::new_v4().to_string(),
            timestamp: Utc::now().to_rfc3339(),
        }
    }
}

impl<T: Serialize> IntoResponse for ApiSuccess<T> {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_status_codes() {
        assert_eq!(
            ApiError::new("AUTH_FAILED", "Invalid credentials").status_code(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            ApiError::new("NOT_FOUND", "User not found").status_code(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            ApiError::new("VALIDATION_ERROR", "Invalid input").status_code(),
            StatusCode::BAD_REQUEST
        );
    }

    #[test]
    fn test_error_with_details() {
        let error = ApiError::new("AUTH_FAILED", "Invalid credentials")
            .with_details("Password mismatch");
        assert_eq!(error.details, Some("Password mismatch".to_string()));
    }
}
