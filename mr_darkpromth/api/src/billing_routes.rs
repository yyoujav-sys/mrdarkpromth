use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::auth_middleware::AuthenticatedUser;
use mr_darkpromth_services::{BillingService, UserService};

#[derive(Debug, Clone)]
pub struct BillingState {
    pub billing_service: BillingService,
    pub user_service: UserService,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateQRRequest {
    pub plan_id: String,
    pub amount: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifySlipRequest {
    pub payment_id: String,
    pub reference: String,
    pub amount: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionResponse {
    pub id: String,
    pub user_id: String,
    pub plan_id: String,
    pub tier: String,
    pub status: String,
    pub start_date: String,
    pub end_date: String,
    pub auto_renew: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentHistoryItem {
    pub id: String,
    pub plan_id: String,
    pub amount: f64,
    pub status: String,
    pub created_at: String,
}

pub fn billing_routes(state: BillingState) -> Router {
    Router::new()
        .route("/plans", get(get_plans))
        .route("/plans/:id", get(get_plan))
        .route("/generate-qr", post(generate_qr_code))
        .route("/verify-slip", post(verify_payment_slip))
        .route("/subscription", get(get_subscription))
        .route("/history", get(get_payment_history))
        .with_state(state)
}

async fn get_plans(State(state): State<BillingState>) -> impl IntoResponse {
    let plans = state.billing_service.get_plans();
    Json(json!({
        "plans": plans
    }))
}

async fn get_plan(
    State(state): State<BillingState>,
    Path(plan_id): Path<String>,
) -> impl IntoResponse {
    match state.billing_service.get_plan(&plan_id) {
        Ok(plan) => (StatusCode::OK, Json(json!({ "plan": plan }))).into_response(),
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Plan not found" })),
        )
            .into_response(),
    }
}

async fn generate_qr_code(
    State(state): State<BillingState>,
    user: AuthenticatedUser,
    Json(payload): Json<GenerateQRRequest>,
) -> impl IntoResponse {
    // Validate plan exists
    if let Err(_) = state.billing_service.get_plan(&payload.plan_id) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Invalid plan ID" })),
        )
            .into_response();
    }

    // Validate amount
    if payload.amount <= 0.0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Invalid amount" })),
        )
            .into_response();
    }

    // Generate QR code
    match state.billing_service.generate_qr_code(&payload.plan_id, payload.amount) {
        Ok(qr_data) => (
            StatusCode::OK,
            Json(json!({
                "qr_code": qr_data.qr_code,
                "payment_id": qr_data.payment_id,
                "amount": qr_data.amount,
                "reference": qr_data.reference,
                "expires_at": qr_data.expires_at.to_rfc3339()
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Failed to generate QR code: {}", e) })),
        )
            .into_response(),
    }
}

async fn verify_payment_slip(
    State(state): State<BillingState>,
    user: AuthenticatedUser,
    Json(payload): Json<VerifySlipRequest>,
) -> impl IntoResponse {
    // Validate payment amount
    if let Err(e) = state
        .billing_service
        .validate_payment_amount(&payload.payment_id, payload.amount)
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": format!("Amount validation failed: {}", e) })),
        )
            .into_response();
    }

    // Verify payment slip
    match state.billing_service.verify_user_payment_slip(
        &payload.payment_id,
        payload.amount,
        &payload.reference,
    ).await {
        Ok(result) => {
            // Create subscription for user
            if let Ok(plan_id) = extract_plan_id_from_payment(&payload.payment_id) {
                if let Ok(subscription) =
                    state.billing_service.create_subscription(&user.id, &plan_id).await
                {
                    return (
                        StatusCode::OK,
                        Json(json!({
                            "verified": result.verified,
                            "payment_id": result.payment_id,
                            "amount": result.amount,
                            "reference": result.reference,
                            "timestamp": result.timestamp.to_rfc3339(),
                            "subscription": {
                                "id": subscription.id,
                                "tier": subscription.tier,
                                "start_date": subscription.start_date.to_rfc3339(),
                                "end_date": subscription.end_date.to_rfc3339()
                            }
                        })),
                    )
                        .into_response();
                }
            }

            (
                StatusCode::OK,
                Json(json!({
                    "verified": result.verified,
                    "payment_id": result.payment_id,
                    "amount": result.amount,
                    "reference": result.reference,
                    "timestamp": result.timestamp.to_rfc3339()
                })),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": format!("Verification failed: {}", e) })),
        )
            .into_response(),
    }
}

async fn get_subscription(
    State(state): State<BillingState>,
    user: AuthenticatedUser,
) -> impl IntoResponse {
    // This would query the database in a real implementation
    // For now, return a mock response
    (
        StatusCode::OK,
        Json(json!({
            "subscription": {
                "id": Uuid::new_v4().to_string(),
                "user_id": user.id,
                "plan_id": "free",
                "tier": "free",
                "status": "active",
                "start_date": chrono::Utc::now().to_rfc3339(),
                "end_date": (chrono::Utc::now() + chrono::Duration::days(30)).to_rfc3339(),
                "auto_renew": false
            }
        })),
    )
        .into_response()
}

async fn get_payment_history(
    State(state): State<BillingState>,
    user: AuthenticatedUser,
) -> impl IntoResponse {
    match state.billing_service.get_payment_history(&user.id) {
        Ok(payments) => {
            let history: Vec<PaymentHistoryItem> = payments
                .iter()
                .map(|p| PaymentHistoryItem {
                    id: p.id.clone(),
                    plan_id: p.plan_id.clone(),
                    amount: p.amount,
                    status: format!("{:?}", p.status),
                    created_at: p.created_at.to_rfc3339(),
                })
                .collect();

            (StatusCode::OK, Json(json!({ "history": history }))).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Failed to fetch payment history: {}", e) })),
        )
            .into_response(),
    }
}

// Helper function to extract plan ID from payment ID
// In a real implementation, this would query the database
fn extract_plan_id_from_payment(payment_id: &str) -> Result<String, String> {
    // Default to premium-monthly for demo purposes
    Ok("premium-monthly".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_qr_request_serialization() {
        let req = GenerateQRRequest {
            plan_id: "premium-monthly".to_string(),
            amount: 9.99,
        };

        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("premium-monthly"));
        assert!(json.contains("9.99"));
    }

    #[test]
    fn test_verify_slip_request_serialization() {
        let req = VerifySlipRequest {
            payment_id: "pay-123".to_string(),
            reference: "PAY-123456".to_string(),
            amount: 9.99,
        };

        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("pay-123"));
        assert!(json.contains("PAY-123456"));
    }
}
