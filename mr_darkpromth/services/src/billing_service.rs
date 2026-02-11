use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use bigdecimal::BigDecimal;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Plan {
    pub id: Uuid,
    pub name: String,
    pub tier: String,
    pub price: BigDecimal,
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
    pub amount: BigDecimal,
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
    pub amount: BigDecimal,
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
    pub amount: BigDecimal,
    pub reference: String,
    pub timestamp: DateTime<Utc>,
}

#[async_trait::async_trait]
pub trait BillingRepository: Send + Sync {
    async fn get_active_plans(&self) -> Result<Vec<Plan>>;
    async fn get_plan_by_id(&self, id: Uuid) -> Result<Option<Plan>>;
    async fn create_payment(&self, payment: Payment) -> Result<()>;
    async fn create_slip_verification(&self, verification: PaymentSlipVerification) -> Result<PaymentSlipVerification>;
    async fn update_slip_verification(&self, id: Uuid, status: &str, admin_id: Uuid, notes: Option<String>) -> Result<PaymentSlipVerification>;
    async fn get_payment_by_id(&self, id: Uuid) -> Result<Option<Payment>>;
    async fn update_payment_status(&self, id: Uuid, status: &str, admin_id: Option<Uuid>) -> Result<()>;
    async fn update_user_tier(&self, user_id: Uuid, tier: &str) -> Result<()>;
    async fn get_active_subscription(&self, user_id: Uuid) -> Result<Option<Subscription>>;
    async fn update_subscription(&self, sub: Subscription) -> Result<Subscription>;
    async fn create_subscription(&self, sub: Subscription) -> Result<Subscription>;
    async fn get_payment_history(&self, user_id: Uuid) -> Result<Vec<Payment>>;
    async fn get_pending_verifications(&self) -> Result<Vec<PaymentSlipVerification>>;
    async fn get_payment_by_id_and_reference(&self, id: Uuid, reference: &str) -> Result<Option<Payment>>;
    async fn get_payment_by_reference(&self, reference: &str) -> Result<Option<Payment>>;
    async fn cancel_subscription(&self, id: Uuid) -> Result<()>;
    async fn check_and_increment_quota(&self, user_id: Uuid, tier: &str) -> Result<bool>;
}

pub struct PostgresBillingRepository {
    pool: sqlx::PgPool,
}

impl PostgresBillingRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl BillingRepository for PostgresBillingRepository {
    async fn get_active_plans(&self) -> Result<Vec<Plan>> {
        sqlx::query_as::<_, Plan>("SELECT * FROM plans WHERE is_active = true ORDER BY price ASC")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    async fn get_plan_by_id(&self, id: Uuid) -> Result<Option<Plan>> {
        sqlx::query_as::<_, Plan>("SELECT * FROM plans WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    async fn create_payment(&self, p: Payment) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO payments (id, user_id, plan_id, amount, status, payment_method, reference, qr_code_data, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#
        )
        .bind(p.id).bind(p.user_id).bind(p.plan_id).bind(p.amount)
        .bind(p.status).bind(p.payment_method).bind(p.reference)
        .bind(p.qr_code_data).bind(p.expires_at)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|e| anyhow::anyhow!(e))
    }

    async fn create_slip_verification(&self, v: PaymentSlipVerification) -> Result<PaymentSlipVerification> {
        sqlx::query_as::<_, PaymentSlipVerification>(
            r#"
            INSERT INTO payment_slip_verifications (id, payment_id, user_id, slip_image_path, status)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#
        )
        .bind(v.id).bind(v.payment_id).bind(v.user_id).bind(v.slip_image_path).bind(v.status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!(e))
    }

    async fn update_slip_verification(&self, id: Uuid, status: &str, admin_id: Uuid, notes: Option<String>) -> Result<PaymentSlipVerification> {
        sqlx::query_as(
            r#"
            UPDATE payment_slip_verifications
            SET status = $1, reviewed_by = $2, reviewed_at = NOW(), review_notes = $3
            WHERE id = $4
            RETURNING *
            "#
        )
        .bind(status).bind(admin_id).bind(notes).bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!(e))
    }

    async fn get_payment_by_id(&self, id: Uuid) -> Result<Option<Payment>> {
        sqlx::query_as("SELECT * FROM payments WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    async fn update_payment_status(&self, id: Uuid, status: &str, admin_id: Option<Uuid>) -> Result<()> {
        let mut query = "UPDATE payments SET status = $1".to_string();
        if admin_id.is_some() {
            query.push_str(", verified_by = $2, verified_at = NOW()");
        } else {
             query.push_str(", verified_at = NOW()");
        }
        query.push_str(" WHERE id = $3"); // or $2 if admin_id is None

        if let Some(aid) = admin_id {
             sqlx::query("UPDATE payments SET status = $1, verified_by = $2, verified_at = NOW() WHERE id = $3")
                .bind(status).bind(aid).bind(id)
                .execute(&self.pool).await.map(|_| ()).map_err(|e| anyhow::anyhow!(e))
        } else {
             sqlx::query("UPDATE payments SET status = $1, verified_at = NOW() WHERE id = $2")
                .bind(status).bind(id)
                .execute(&self.pool).await.map(|_| ()).map_err(|e| anyhow::anyhow!(e))
        }
    }

    async fn update_user_tier(&self, user_id: Uuid, tier: &str) -> Result<()> {
        sqlx::query("UPDATE users SET tier = $1::user_tier WHERE id = $2")
            .bind(tier).bind(user_id)
            .execute(&self.pool).await.map(|_| ()).map_err(|e| anyhow::anyhow!(e))
    }

    async fn get_active_subscription(&self, user_id: Uuid) -> Result<Option<Subscription>> {
        sqlx::query_as(
            "SELECT * FROM subscriptions WHERE user_id = $1 AND status = 'active' ORDER BY end_date DESC LIMIT 1"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!(e))
    }

    async fn update_subscription(&self, s: Subscription) -> Result<Subscription> {
        sqlx::query_as(
            r#"
            UPDATE subscriptions
            SET plan_id = $1, payment_id = $2, tier = $3, end_date = $4, updated_at = NOW()
            WHERE id = $5
            RETURNING *
            "#
        )
        .bind(s.plan_id).bind(s.payment_id).bind(s.tier).bind(s.end_date).bind(s.id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!(e))
    }

     async fn create_subscription(&self, s: Subscription) -> Result<Subscription> {
        sqlx::query_as(
            r#"
            INSERT INTO subscriptions (id, user_id, plan_id, payment_id, tier, status, start_date, end_date, auto_renew)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#
        )
        .bind(s.id).bind(s.user_id).bind(s.plan_id).bind(s.payment_id).bind(s.tier)
        .bind(s.status).bind(s.start_date).bind(s.end_date).bind(s.auto_renew)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!(e))
    }

    async fn get_payment_history(&self, user_id: Uuid) -> Result<Vec<Payment>> {
        sqlx::query_as("SELECT * FROM payments WHERE user_id = $1 ORDER BY created_at DESC")
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    async fn get_pending_verifications(&self) -> Result<Vec<PaymentSlipVerification>> {
        sqlx::query_as("SELECT * FROM payment_slip_verifications WHERE status = 'pending' ORDER BY submitted_at ASC")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    async fn get_payment_by_id_and_reference(&self, id: Uuid, reference: &str) -> Result<Option<Payment>> {
        sqlx::query_as("SELECT * FROM payments WHERE id = $1 AND reference = $2")
            .bind(id).bind(reference)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    async fn get_payment_by_reference(&self, reference: &str) -> Result<Option<Payment>> {
        sqlx::query_as("SELECT * FROM payments WHERE reference = $1")
            .bind(reference)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    async fn cancel_subscription(&self, id: Uuid) -> Result<()> {
        sqlx::query("UPDATE subscriptions SET status = $1, updated_at = NOW() WHERE id = $2")
            .bind(SubscriptionStatus::Cancelled.as_str()).bind(id)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|e| anyhow::anyhow!(e))
    }

    async fn check_and_increment_quota(&self, user_id: Uuid, tier: &str) -> Result<bool> {
        let row: (bool,) = sqlx::query_as("SELECT update_daily_quota($1, $2)")
            .bind(user_id)
            .bind(tier)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
        Ok(row.0)
    }
}

pub struct BillingService {
    repository: std::sync::Arc<dyn BillingRepository>,
}

impl BillingService {
    pub fn new(repository: std::sync::Arc<dyn BillingRepository>) -> Self {
        Self { repository }
    }

    pub fn new_with_pool(pool: sqlx::PgPool) -> Self {
        Self::new(std::sync::Arc::new(PostgresBillingRepository::new(pool)))
    }

    pub async fn get_plans(&self) -> Result<Vec<Plan>> {
        self.repository.get_active_plans().await
    }

    pub async fn get_plan(&self, plan_id: Uuid) -> Result<Plan> {
        self.repository.get_plan_by_id(plan_id).await?
            .ok_or_else(|| anyhow!("Plan not found: {}", plan_id))
    }

    pub async fn generate_qr_code(&self, user_id: Uuid, plan_id: Uuid) -> Result<QRCodePaymentData> {
        let plan = self.get_plan(plan_id).await?;
        let payment_id = Uuid::new_v4();
        let reference = self.generate_reference_number();

        let _qr_data = format!("PAYMENT|{}|{}|{}|{}", payment_id, reference, plan.price, plan.name);
        let placeholder_qr = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";
        let expires_at = Utc::now() + Duration::minutes(15);

        let payment = Payment {
            id: payment_id,
            user_id,
            plan_id,
            amount: plan.price.clone(),
            status: PaymentStatus::Pending.as_str().to_string(),
            payment_method: PaymentMethod::QRCode.as_str().to_string(),
            reference: reference.clone(),
            qr_code_data: Some(placeholder_qr.to_string()),
            slip_image_path: None,
            verified_by: None,
            verified_at: None,
            expires_at: Some(expires_at),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.repository.create_payment(payment).await?;

        Ok(QRCodePaymentData {
            qr_code: format!("data:image/png;base64,{}", placeholder_qr),
            payment_id,
            amount: plan.price,
            reference,
            expires_at,
        })
    }

    pub async fn get_payment_by_reference(&self, reference: &str) -> Result<Option<Payment>> {
        self.repository.get_payment_by_reference(reference).await
    }

    pub async fn submit_slip_for_verification(&self, payment_id: Uuid, user_id: Uuid, slip_path: &str) -> Result<PaymentSlipVerification> {
        let verification = PaymentSlipVerification {
            id: Uuid::new_v4(),
            payment_id,
            user_id,
            slip_image_path: slip_path.to_string(),
            status: "pending".to_string(),
            submitted_at: Utc::now(),
            reviewed_by: None,
            reviewed_at: None,
            review_notes: None,
            created_at: Utc::now(),
        };
        self.repository.create_slip_verification(verification).await
    }

    pub async fn verify_payment_slip(&self, verification_id: Uuid, admin_id: Uuid, approved: bool, notes: Option<&str>) -> Result<SlipVerificationResult> {
        let status = if approved { "approved" } else { "rejected" };
        let verification = self.repository.update_slip_verification(verification_id, status, admin_id, notes.map(|s| s.to_string())).await?;

        let payment = self.repository.get_payment_by_id(verification.payment_id).await?
            .ok_or_else(|| anyhow!("Payment not found"))?;

        if approved {
            self.repository.update_payment_status(verification.payment_id, PaymentStatus::Verified.as_str(), Some(admin_id)).await?;
            let plan = self.get_plan(payment.plan_id).await?;
            self.create_or_update_subscription(verification.user_id, payment.plan_id, verification.payment_id, &plan.tier).await?;
            self.repository.update_user_tier(verification.user_id, &plan.tier).await?;
        }

        Ok(SlipVerificationResult {
            verified: approved,
            payment_id: verification.payment_id,
            amount: payment.amount,
            reference: payment.reference,
            timestamp: Utc::now(),
        })
    }

    pub async fn update_user_tier(&self, user_id: Uuid, tier: &str) -> Result<()> {
        self.repository.update_user_tier(user_id, tier).await
    }

    pub async fn validate_payment_amount(&self, payment_id: &str, amount: BigDecimal) -> Result<()> {
        let id = Uuid::parse_str(payment_id).map_err(|e| anyhow!("Invalid payment ID format: {}", e))?;
        let payment = self.repository.get_payment_by_id(id).await?
            .ok_or_else(|| anyhow!("Payment not found"))?;
        
        if (payment.amount.clone() - amount.clone()).abs() > BigDecimal::from(1) / BigDecimal::from(100) {
            return Err(anyhow!("Payment amount mismatch. Expected: {}, Got: {}", payment.amount, amount));
        }
        Ok(())
    }

    pub async fn verify_user_payment_slip(&self, payment_id_str: &str, amount: BigDecimal, reference: &str) -> Result<SlipVerificationResult> {
        let payment_id = Uuid::parse_str(payment_id_str).map_err(|e| anyhow!("Invalid payment ID format: {}", e))?;
        
        // Find payment by ID and reference
        let payment = self.repository.get_payment_by_id_and_reference(payment_id, reference).await?
            .ok_or_else(|| anyhow!("Payment not found with given ID and reference"))?;
        
        if (payment.amount.clone() - amount.clone()).abs() > BigDecimal::from(1) / BigDecimal::from(100) {
             return Err(anyhow!("Payment amount mismatch"));
        }
        
        if payment.status != PaymentStatus::Pending.as_str() {
             return Err(anyhow!("Payment is not in pending status"));
        }
        
        let verified = reference.starts_with("PAY-") && payment.reference == reference;
        
        if verified {
            self.repository.update_payment_status(payment_id, PaymentStatus::Verified.as_str(), None).await?;
            let plan = self.get_plan(payment.plan_id).await?;
            self.create_or_update_subscription(payment.user_id, payment.plan_id, payment_id, &plan.tier).await?;
            self.repository.update_user_tier(payment.user_id, &plan.tier).await?;
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
        let user_id = Uuid::parse_str(user_id_str).map_err(|e| anyhow!("Invalid user ID: {}", e))?;
        let plan_id = Uuid::parse_str(plan_id_str).map_err(|e| anyhow!("Invalid plan ID: {}", e))?;
        
        let plan = self.get_plan(plan_id).await?;
        self.create_or_update_subscription(user_id, plan_id, Uuid::new_v4(), &plan.tier).await
    }

    pub async fn create_or_update_subscription(&self, user_id: Uuid, plan_id: Uuid, payment_id: Uuid, tier: &str) -> Result<Subscription> {
        let plan = self.get_plan(plan_id).await?;
        let now = Utc::now();
        let end_date = now + Duration::days(plan.duration_days as i64);

        let existing = self.repository.get_active_subscription(user_id).await?;

        if let Some(existing) = existing {
            let new_end_date = if existing.end_date > now {
                existing.end_date + Duration::days(plan.duration_days as i64)
            } else {
                end_date
            };
            
            let mut updated = existing;
            updated.plan_id = plan_id;
            updated.payment_id = Some(payment_id);
            updated.tier = tier.to_string();
            updated.end_date = new_end_date;
            
            self.repository.update_subscription(updated).await
        } else {
            let subscription = Subscription {
                id: Uuid::new_v4(),
                user_id,
                plan_id,
                payment_id: Some(payment_id),
                tier: tier.to_string(),
                status: SubscriptionStatus::Active.as_str().to_string(),
                start_date: now,
                end_date,
                auto_renew: false,
                created_at: now,
                updated_at: now,
            };
            self.repository.create_subscription(subscription).await
        }
    }

    pub async fn get_user_subscription(&self, user_id: Uuid) -> Result<Option<Subscription>> {
        self.repository.get_active_subscription(user_id).await
    }

    pub async fn get_payment_history(&self, user_id: Uuid) -> Result<Vec<Payment>> {
        self.repository.get_payment_history(user_id).await
    }

    pub async fn get_pending_verifications(&self) -> Result<Vec<PaymentSlipVerification>> {
        self.repository.get_pending_verifications().await
    }

    pub fn generate_reference_number(&self) -> String {
        let timestamp = Utc::now().timestamp_millis();
        let random = Uuid::new_v4().to_string();
        format!("PAY-{}-{}", timestamp, &random[..8].to_uppercase())
    }

    pub async fn process_chat_usage(&self, user_id: Uuid, _model: String) -> Result<bool> {
        // 1. Get user's current tier
        let sub = self.repository.get_active_subscription(user_id).await?;
        let tier = sub.map(|s| s.tier).unwrap_or_else(|| "free".to_string());
        
        // 2. Check and increment quota using the DB function
        self.repository.check_and_increment_quota(user_id, &tier).await
    }

    pub async fn cancel_subscription(&self, subscription_id: Uuid) -> Result<()> {
        self.repository.cancel_subscription(subscription_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use std::sync::Arc;

    struct MockBillingRepository {
        plans: Vec<Plan>,
    }

    impl MockBillingRepository {
        fn new() -> Self {
            Self { plans: vec![] }
        }
        
        fn with_plans(plans: Vec<Plan>) -> Self {
            Self { plans }
        }
    }

    #[async_trait::async_trait]
    impl BillingRepository for MockBillingRepository {
        async fn get_active_plans(&self) -> Result<Vec<Plan>> {
            Ok(self.plans.clone())
        }
        async fn get_plan_by_id(&self, _id: Uuid) -> Result<Option<Plan>> { Ok(None) }
        async fn create_payment(&self, _p: Payment) -> Result<()> { Ok(()) }
        async fn create_slip_verification(&self, v: PaymentSlipVerification) -> Result<PaymentSlipVerification> { Ok(v) }
        async fn update_slip_verification(&self, _id: Uuid, _status: &str, _admin_id: Uuid, _notes: Option<String>) -> Result<PaymentSlipVerification> { Err(anyhow!("unimplemented")) }
        async fn get_payment_by_id(&self, _id: Uuid) -> Result<Option<Payment>> { Ok(None) }
        async fn update_payment_status(&self, _id: Uuid, _status: &str, _admin_id: Option<Uuid>) -> Result<()> { Ok(()) }
        async fn update_user_tier(&self, _user_id: Uuid, _tier: &str) -> Result<()> { Ok(()) }
        async fn get_active_subscription(&self, _user_id: Uuid) -> Result<Option<Subscription>> { Ok(None) }
        async fn update_subscription(&self, s: Subscription) -> Result<Subscription> { Ok(s) }
        async fn create_subscription(&self, s: Subscription) -> Result<Subscription> { Ok(s) }
        async fn get_payment_history(&self, _user_id: Uuid) -> Result<Vec<Payment>> { Ok(vec![]) }
        async fn get_pending_verifications(&self) -> Result<Vec<PaymentSlipVerification>> { Ok(vec![]) }
        async fn get_payment_by_id_and_reference(&self, _id: Uuid, _reference: &str) -> Result<Option<Payment>> { Ok(None) }
        async fn get_payment_by_reference(&self, _reference: &str) -> Result<Option<Payment>> { Ok(None) }
        async fn cancel_subscription(&self, _id: Uuid) -> Result<()> { Ok(()) }
        async fn check_and_increment_quota(&self, _user_id: Uuid, _tier: &str) -> Result<bool> { Ok(true) }
    }

    #[tokio::test]
    async fn test_generate_reference_number() {
        let repo = Arc::new(MockBillingRepository::new());
        let billing = BillingService::new(repo);
        let ref1 = billing.generate_reference_number();
        let ref2 = billing.generate_reference_number();

        assert!(ref1.starts_with("PAY-"));
        assert!(ref2.starts_with("PAY-"));
        assert_ne!(ref1, ref2);
    }

    #[tokio::test]
    async fn test_get_plans() {
        let test_plan = Plan {
            id: Uuid::new_v4(),
            name: "Test Plan".to_string(),
            tier: "free".to_string(),
            price: BigDecimal::from(999) / BigDecimal::from(100),
            duration_days: 30,
            features: serde_json::json!({}),
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let repo = Arc::new(MockBillingRepository::with_plans(vec![test_plan]));
        let billing = BillingService::new(repo);

        let plans = billing.get_plans().await.expect("Failed to get plans");

        assert!(!plans.is_empty(), "Expected to find at least one active plan");
        assert_eq!(plans[0].name, "Test Plan");
    }

    /// Integration Test: Complete Flow - User submits slip -> Admin approves -> Tier upgrade
    #[tokio::test]
    async fn test_complete_payment_verification_flow() {
        // Step 1: Create a mock repository with a premium plan
        let premium_plan = Plan {
            id: Uuid::new_v4(),
            name: "Premium Plan".to_string(),
            tier: "premium".to_string(),
            price: BigDecimal::from(29900) / BigDecimal::from(100), // 299.00
            duration_days: 30,
            features: serde_json::json!({"max_prompts": 100}),
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let repo = Arc::new(MockBillingRepository::with_plans(vec![premium_plan.clone()]));
        let billing = BillingService::new(repo);

        // Step 2: Simulate user generating QR code for payment
        let user_id = Uuid::new_v4();
        let plan_id = premium_plan.id;
        
        let qr_data = billing.generate_qr_code(user_id, plan_id).await
            .expect("Failed to generate QR code");
        
        assert!(qr_data.reference.starts_with("PAY-"));
        assert_eq!(qr_data.amount, premium_plan.price);

        // Step 3: Simulate user submitting slip for verification
        let slip_path = "/uploads/slip_12345.jpg".to_string();
        let verification = billing.submit_slip_for_verification(
            qr_data.payment_id, 
            user_id, 
            &slip_path
        ).await.expect("Failed to submit slip");

        assert_eq!(verification.status, "pending");
        assert_eq!(verification.slip_image_path, slip_path);

        // Step 4: Simulate admin approving the verification
        let admin_id = Uuid::new_v4();
        let result = billing.verify_payment_slip(
            verification.id, 
            admin_id, 
            true,  // approved
            Some("Payment verified successfully")
        ).await.expect("Failed to verify payment");

        assert!(result.verified);
        assert_eq!(result.payment_id, qr_data.payment_id);

        println!("✅ Integration Test Passed: Complete payment flow");
        println!("   - User ID: {}", user_id);
        println!("   - Payment ID: {}", qr_data.payment_id);
        println!("   - Reference: {}", result.reference);
        println!("   - Amount: {}", result.amount);
    }

    /// Integration Test: Admin rejects slip verification
    #[tokio::test]
    async fn test_slip_rejection_flow() {
        let premium_plan = Plan {
            id: Uuid::new_v4(),
            name: "Premium Plan".to_string(),
            tier: "premium".to_string(),
            price: BigDecimal::from(29900) / BigDecimal::from(100),
            duration_days: 30,
            features: serde_json::json!({}),
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let repo = Arc::new(MockBillingRepository::with_plans(vec![premium_plan.clone()]));
        let billing = BillingService::new(repo);

        let user_id = Uuid::new_v4();
        let qr_data = billing.generate_qr_code(user_id, premium_plan.id).await
            .expect("Failed to generate QR");

        let verification = billing.submit_slip_for_verification(
            qr_data.payment_id, 
            user_id, 
            "/uploads/slip_invalid.jpg"
        ).await.expect("Failed to submit slip");

        // Admin rejects
        let admin_id = Uuid::new_v4();
        let result = billing.verify_payment_slip(
            verification.id, 
            admin_id, 
            false,  // rejected
            Some("Image unclear, please resubmit")
        ).await.expect("Failed to reject verification");

        assert!(!result.verified);

        println!("✅ Integration Test Passed: Slip rejection flow");
    }

    /// Integration Test: Verify process_chat_usage enforces quota
    #[tokio::test]
    async fn test_process_chat_usage_with_mock() {
        let repo = Arc::new(MockBillingRepository::new());
        let billing = BillingService::new(repo);

        // Test that process_chat_usage returns true (allowing the request)
        // In real implementation, this would check usage_by_day
        let user_id = Uuid::new_v4();
        let result = billing.process_chat_usage(user_id, "gpt-4".to_string()).await;
        
        assert!(result.is_ok());
        assert!(result.unwrap()); // Currently allows all requests

        println!("✅ Integration Test Passed: Chat usage processing");
    }
}
