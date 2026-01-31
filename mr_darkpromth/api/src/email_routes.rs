use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::auth_middleware::AuthenticatedUser;
use mr_darkpromth_services::{EmailService, UserService};

#[derive(Debug, Clone)]
pub struct EmailState {
    pub email_service: EmailService,
    pub user_service: UserService,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyEmailRequest {
    pub token: String,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResendVerificationRequest {
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestPasswordResetRequest {
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

pub fn email_routes(state: EmailState) -> Router {
    Router::new()
        .route("/verify-email", post(verify_email))
        .route("/resend-verification", post(resend_verification))
        .route("/request-password-reset", post(request_password_reset))
        .route("/reset-password", post(reset_password))
        .route("/change-password", post(change_password))
        .with_state(state)
}

async fn verify_email(
    State(state): State<EmailState>,
    Json(payload): Json<VerifyEmailRequest>,
) -> impl IntoResponse {
    // In a real implementation, this would:
    // 1. Look up the verification token in the database
    // 2. Verify it hasn't expired
    // 3. Verify it matches the email
    // 4. Mark the email as verified in the user record

    // For now, we'll just validate the token format
    if payload.token.is_empty() || payload.email.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Invalid token or email" })),
        )
            .into_response();
    }

    // Mock verification success
    (
        StatusCode::OK,
        Json(json!({
            "message": "Email verified successfully",
            "email": payload.email
        })),
    )
        .into_response()
}

async fn resend_verification(
    State(state): State<EmailState>,
    Json(payload): Json<ResendVerificationRequest>,
) -> impl IntoResponse {
    if payload.email.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Email is required" })),
        )
            .into_response();
    }

    // Generate new verification token
    let token = state
        .email_service
        .generate_verification_token("user-id", &payload.email);

    // In a real implementation, send the email
    // For now, just return success
    (
        StatusCode::OK,
        Json(json!({
            "message": "Verification email sent",
            "email": payload.email,
            "expires_at": token.expires_at.to_rfc3339()
        })),
    )
        .into_response()
}

async fn request_password_reset(
    State(state): State<EmailState>,
    Json(payload): Json<RequestPasswordResetRequest>,
) -> impl IntoResponse {
    if payload.email.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Email is required" })),
        )
            .into_response();
    }

    // Generate password reset token
    let token = state
        .email_service
        .generate_password_reset_token("user-id", &payload.email);

    // In a real implementation, send the email
    // For now, just return success
    (
        StatusCode::OK,
        Json(json!({
            "message": "Password reset email sent",
            "email": payload.email,
            "expires_at": token.expires_at.to_rfc3339()
        })),
    )
        .into_response()
}

async fn reset_password(
    State(state): State<EmailState>,
    Json(payload): Json<ResetPasswordRequest>,
) -> impl IntoResponse {
    if payload.token.is_empty() || payload.password.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Token and password are required" })),
        )
            .into_response();
    }

    if payload.password.len() < 8 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Password must be at least 8 characters" })),
        )
            .into_response();
    }

    // In a real implementation, this would:
    // 1. Verify the token
    // 2. Hash the new password
    // 3. Update the user record
    // 4. Mark the token as used

    (
        StatusCode::OK,
        Json(json!({
            "message": "Password reset successfully"
        })),
    )
        .into_response()
}

async fn change_password(
    State(state): State<EmailState>,
    user: AuthenticatedUser,
    Json(payload): Json<ChangePasswordRequest>,
) -> impl IntoResponse {
    if payload.current_password.is_empty() || payload.new_password.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Current and new password are required" })),
        )
            .into_response();
    }

    if payload.new_password.len() < 8 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "New password must be at least 8 characters" })),
        )
            .into_response();
    }

    if payload.current_password == payload.new_password {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "New password must be different from current password" })),
        )
            .into_response();
    }

    // In a real implementation, this would:
    // 1. Verify the current password
    // 2. Hash the new password
    // 3. Update the user record

    (
        StatusCode::OK,
        Json(json!({
            "message": "Password changed successfully"
        })),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_email_request_serialization() {
        let req = VerifyEmailRequest {
            token: "token-123".to_string(),
            email: "user@example.com".to_string(),
        };

        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("token-123"));
        assert!(json.contains("user@example.com"));
    }

    #[test]
    fn test_reset_password_request_serialization() {
        let req = ResetPasswordRequest {
            token: "token-123".to_string(),
            password: "newpassword123".to_string(),
        };

        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("token-123"));
        assert!(json.contains("newpassword123"));
    }
}
