use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Plan {
    pub id: Uuid,
    pub name: String,
    pub tier: String,
    pub price: f64,
    pub duration_days: i32,
    pub features: serde_json::Value,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Payment {
    pub id: Uuid,
    pub user_id: Uuid,
    pub plan_id: Uuid,
    pub amount: f64,
    pub status: String,
    pub payment_method: String,
    pub reference: String,
    pub qr_code_data: Option<String>,
    pub slip_image_path: Option<String>,
    pub verified_by: Option<Uuid>,
    pub verified_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PaymentStatus {
    Pending,
    Verified,
    Failed,
    Expired,
    Refunded,
}

impl PaymentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            PaymentStatus::Pending => "pending",
            PaymentStatus::Verified => "verified",
            PaymentStatus::Failed => "failed",
            PaymentStatus::Expired => "expired",
            PaymentStatus::Refunded => "refunded",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaymentMethod {
    QRCode,
    CreditCard,
    BankTransfer,
}

impl PaymentMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            PaymentMethod::QRCode => "qr_code",
            PaymentMethod::CreditCard => "credit_card",
            PaymentMethod::BankTransfer => "bank_transfer",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Subscription {
    pub id: Uuid,
    pub user_id: Uuid,
    pub plan_id: Uuid,
    pub payment_id: Option<Uuid>,
    pub tier: String,
    pub status: String,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub auto_renew: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SubscriptionStatus {
    Active,
    Inactive,
    Cancelled,
    Expired,
}

impl SubscriptionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            SubscriptionStatus::Active => "active",
            SubscriptionStatus::Inactive => "inactive",
            SubscriptionStatus::Cancelled => "cancelled",
            SubscriptionStatus::Expired => "expired",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PaymentSlipVerification {
    pub id: Uuid,
    pub payment_id: Uuid,
    pub user_id: Uuid,
    pub slip_image_path: String,
    pub status: String,
    pub submitted_at: DateTime<Utc>,
    pub reviewed_by: Option<Uuid>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub review_notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QRCodePaymentData {
    pub qr_code: String,
    pub payment_id: Uuid,
    pub amount: f64,
    pub reference: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlipVerificationRequest {
    pub payment_id: Uuid,
    pub slip_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlipVerificationResult {
    pub verified: bool,
    pub payment_id: Uuid,
    pub amount: f64,
    pub reference: String,
    pub timestamp: DateTime<Utc>,
}

pub struct BillingService {
    pool: sqlx::PgPool,
}

impl BillingService {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_plans(&self) -> Result<Vec<Plan>> {
        let plans = sqlx::query_as::<_, Plan>(
            "SELECT * FROM plans WHERE is_active = true ORDER BY price ASC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(plans)
    }

    pub async fn get_plan(&self, plan_id: Uuid) -> Result<Plan> {
        let plan = sqlx::query_as::<_, Plan>(
            "SELECT * FROM plans WHERE id = $1"
        )
        .bind(plan_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| anyhow!("Plan not found: {}", plan_id))?;

        Ok(plan)
    }

    pub async fn generate_qr_code(&self, user_id: Uuid, plan_id: Uuid) -> Result<QRCodePaymentData> {
        let plan = self.get_plan(plan_id).await?;
        let payment_id = Uuid::new_v4();
        let reference = self.generate_reference_number();

        // Generate QR code data (simplified - in production use proper QR library)
        let qr_data = format!(
            "PAYMENT|{}|{}|{}|{}",
            payment_id, reference, plan.price, plan.name
        );

        // Placeholder QR code - base64 encoded placeholder
        // In production, use a proper QR code generation library or service
        let placeholder_qr = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";

        let expires_at = Utc::now() + Duration::minutes(15);

        // Save payment record
        sqlx::query(
            r#"
            INSERT INTO payments (id, user_id, plan_id, amount, status, payment_method, reference, qr_code_data, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#
        )
        .bind(payment_id)
        .bind(user_id)
        .bind(plan_id)
        .bind(plan.price)
        .bind(PaymentStatus::Pending.as_str())
        .bind(PaymentMethod::QRCode.as_str())
        .bind(&reference)
        .bind(&placeholder_qr)
        .bind(expires_at)
        .execute(&self.pool)
        .await?;

        Ok(QRCodePaymentData {
            qr_code: format!("data:image/png;base64,{}", placeholder_qr),
            payment_id,
            amount: plan.price,
            reference,
            expires_at,
        })
    }

    pub async fn submit_slip_for_verification(
        &self,
        payment_id: Uuid,
        user_id: Uuid,
        slip_path: &str,
    ) -> Result<PaymentSlipVerification> {
        let verification_id = Uuid::new_v4();

        let verification = sqlx::query_as::<_, PaymentSlipVerification>(
            r#"
            INSERT INTO payment_slip_verifications (id, payment_id, user_id, slip_image_path, status)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#
        )
        .bind(verification_id)
        .bind(payment_id)
        .bind(user_id)
        .bind(slip_path)
        .bind("pending")
        .fetch_one(&self.pool)
        .await?;

        Ok(verification)
    }

    pub async fn verify_payment_slip(
        &self,
        verification_id: Uuid,
        admin_id: Uuid,
        approved: bool,
        notes: Option<&str>,
    ) -> Result<SlipVerificationResult> {
        let status = if approved { "approved" } else { "rejected" };

        // Update verification record
        let verification: PaymentSlipVerification = sqlx::query_as(
            r#"
            UPDATE payment_slip_verifications
            SET status = $1, reviewed_by = $2, reviewed_at = NOW(), review_notes = $3
            WHERE id = $4
            RETURNING *
            "#
        )
        .bind(status)
        .bind(admin_id)
        .bind(notes)
        .bind(verification_id)
        .fetch_one(&self.pool)
        .await?;

        // Get payment details
        let payment: Payment = sqlx::query_as(
            "SELECT * FROM payments WHERE id = $1"
        )
        .bind(verification.payment_id)
        .fetch_one(&self.pool)
        .await?;

        if approved {
            // Update payment status
            sqlx::query(
                r#"
                UPDATE payments
                SET status = $1, verified_by = $2, verified_at = NOW()
                WHERE id = $3
                "#
            )
            .bind(PaymentStatus::Verified.as_str())
            .bind(admin_id)
            .bind(verification.payment_id)
            .execute(&self.pool)
            .await?;

            // Get plan tier and create/update subscription
            let plan: Plan = sqlx::query_as(
                "SELECT * FROM plans WHERE id = $1"
            )
            .bind(payment.plan_id)
            .fetch_one(&self.pool)
            .await?;

            self.create_or_update_subscription(
                verification.user_id,
                payment.plan_id,
                verification.payment_id,
                &plan.tier,
            ).await?;

            // Update user tier
            sqlx::query("UPDATE users SET tier = $1 WHERE id = $2")
                .bind(&plan.tier)
                .bind(verification.user_id)
                .execute(&self.pool)
                .await?;
        }

        Ok(SlipVerificationResult {
            verified: approved,
            payment_id: verification.payment_id,
            amount: payment.amount,
            reference: payment.reference,
            timestamp: Utc::now(),
        })
    }

    pub async fn validate_payment_amount(&self, payment_id: &str, amount: f64) -> Result<()> {
        let id = Uuid::parse_str(payment_id)
            .map_err(|e| anyhow!("Invalid payment ID format: {}", e))?;
        
        let payment: Payment = sqlx::query_as(
            "SELECT * FROM payments WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| anyhow!("Payment not found"))?;
        
        if (payment.amount - amount).abs() > 0.01 {
            return Err(anyhow!("Payment amount mismatch. Expected: {}, Got: {}", payment.amount, amount));
        }
        
        Ok(())
    }

    /// Simple payment verification for user-submitted slips
    pub async fn verify_user_payment_slip(
        &self,
        payment_id_str: &str,
        amount: f64,
        reference: &str,
    ) -> Result<SlipVerificationResult> {
        let payment_id = Uuid::parse_str(payment_id_str)
            .map_err(|e| anyhow!("Invalid payment ID format: {}", e))?;
        
        // Find payment by ID and reference
        let payment: Payment = sqlx::query_as(
            r#"
            SELECT * FROM payments 
            WHERE id = $1 AND reference = $2
            "#
        )
        .bind(payment_id)
        .bind(reference)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| anyhow!("Payment not found with given ID and reference"))?;
        
        // Validate amount matches
        if (payment.amount - amount).abs() > 0.01 {
            return Err(anyhow!("Payment amount mismatch"));
        }
        
        // Check payment is still pending
        if payment.status != PaymentStatus::Pending.as_str() {
            return Err(anyhow!("Payment is not in pending status"));
        }
        
        // For demo purposes, auto-approve if reference matches pattern
        // In production, this would involve actual slip image analysis
        let verified = reference.starts_with("PAY-") && payment.reference == reference;
        
        if verified {
            // Update payment status
            sqlx::query(
                r#"
                UPDATE payments
                SET status = $1, verified_at = NOW()
                WHERE id = $2
                "#
            )
            .bind(PaymentStatus::Verified.as_str())
            .bind(payment_id)
            .execute(&self.pool)
            .await?;
            
            // Get plan and create subscription
            let plan: Plan = sqlx::query_as(
                "SELECT * FROM plans WHERE id = $1"
            )
            .bind(payment.plan_id)
            .fetch_one(&self.pool)
            .await?;
            
            self.create_or_update_subscription(
                payment.user_id,
                payment.plan_id,
                payment_id,
                &plan.tier,
            ).await?;
            
            // Update user tier
            sqlx::query("UPDATE users SET tier = $1 WHERE id = $2")
                .bind(&plan.tier)
                .bind(payment.user_id)
                .execute(&self.pool)
                .await?;
        }
        
        Ok(SlipVerificationResult {
            verified,
            payment_id,
            amount: payment.amount,
            reference: payment.reference,
            timestamp: Utc::now(),
        })
    }

    pub async fn create_subscription(&self, user_id_str: &str, plan_id_str: &str) -> Result<Subscription> {
        let user_id = Uuid::parse_str(user_id_str)
            .map_err(|e| anyhow!("Invalid user ID: {}", e))?;
        let plan_id = Uuid::parse_str(plan_id_str)
            .map_err(|e| anyhow!("Invalid plan ID: {}", e))?;
        
        let plan = self.get_plan(plan_id).await?;
        self.create_or_update_subscription(user_id, plan_id, Uuid::new_v4(), &plan.tier).await
    }

    pub async fn create_or_update_subscription(
        &self,
        user_id: Uuid,
        plan_id: Uuid,
        payment_id: Uuid,
        tier: &str,
    ) -> Result<Subscription> {
        let plan = self.get_plan(plan_id).await?;
        let now = Utc::now();
        let end_date = now + Duration::days(plan.duration_days as i64);

        // Check for existing active subscription
        let existing: Option<Subscription> = sqlx::query_as(
            r#"
            SELECT * FROM subscriptions 
            WHERE user_id = $1 AND status = 'active'
            ORDER BY end_date DESC
            LIMIT 1
            "#
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        let subscription = if let Some(existing) = existing {
            // Extend existing subscription
            let new_end_date = if existing.end_date > now {
                existing.end_date + Duration::days(plan.duration_days as i64)
            } else {
                end_date
            };

            sqlx::query_as(
                r#"
                UPDATE subscriptions
                SET plan_id = $1, payment_id = $2, tier = $3, end_date = $4, updated_at = NOW()
                WHERE id = $5
                RETURNING *
                "#
            )
            .bind(plan_id)
            .bind(payment_id)
            .bind(tier)
            .bind(new_end_date)
            .bind(existing.id)
            .fetch_one(&self.pool)
            .await?
        } else {
            // Create new subscription
            let subscription_id = Uuid::new_v4();
            sqlx::query_as(
                r#"
                INSERT INTO subscriptions (id, user_id, plan_id, payment_id, tier, status, start_date, end_date, auto_renew)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                RETURNING *
                "#
            )
            .bind(subscription_id)
            .bind(user_id)
            .bind(plan_id)
            .bind(payment_id)
            .bind(tier)
            .bind(SubscriptionStatus::Active.as_str())
            .bind(now)
            .bind(end_date)
            .bind(false)
            .fetch_one(&self.pool)
            .await?
        };

        Ok(subscription)
    }

    pub async fn get_user_subscription(&self, user_id: Uuid) -> Result<Option<Subscription>> {
        let subscription = sqlx::query_as::<_, Subscription>(
            r#"
            SELECT * FROM subscriptions 
            WHERE user_id = $1 AND status = 'active'
            ORDER BY end_date DESC
            LIMIT 1
            "#
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(subscription)
    }

    pub async fn get_payment_history(&self, user_id: Uuid) -> Result<Vec<Payment>> {
        let payments = sqlx::query_as::<_, Payment>(
            r#"
            SELECT * FROM payments 
            WHERE user_id = $1 
            ORDER BY created_at DESC
            "#
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(payments)
    }

    pub async fn get_pending_verifications(&self) -> Result<Vec<PaymentSlipVerification>> {
        let verifications = sqlx::query_as::<_, PaymentSlipVerification>(
            r#"
            SELECT * FROM payment_slip_verifications 
            WHERE status = 'pending'
            ORDER BY submitted_at ASC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(verifications)
    }

    pub fn generate_reference_number(&self) -> String {
        let timestamp = Utc::now().timestamp_millis();
        let random = Uuid::new_v4().to_string();
        format!("PAY-{}-{}", timestamp, &random[..8].to_uppercase())
    }

    pub async fn cancel_subscription(&self, subscription_id: Uuid) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE subscriptions
            SET status = $1, updated_at = NOW()
            WHERE id = $2
            "#
        )
        .bind(SubscriptionStatus::Cancelled.as_str())
        .bind(subscription_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_pool() -> sqlx::PgPool {
        sqlx::PgPool::connect_lazy("postgres://postgres:postgres@localhost:5432/mr_darkpromth")
            .expect("Failed to connect to database")
    }

    #[tokio::test]
    async fn test_generate_reference_number() {
        let billing = BillingService::new(get_test_pool());
        let ref1 = billing.generate_reference_number();
        let ref2 = billing.generate_reference_number();

        assert!(ref1.starts_with("PAY-"));
        assert!(ref2.starts_with("PAY-"));
        assert_ne!(ref1, ref2);
    }

    #[tokio::test]
    async fn test_get_plans() {
        let billing = BillingService::new(get_test_pool());
        let plans = billing.get_plans().await.expect("Failed to get plans");

        assert!(!plans.is_empty());
    }
}
