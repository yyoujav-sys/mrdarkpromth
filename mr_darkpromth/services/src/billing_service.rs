use crate::user_service::UserService;
use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use qrcode::QrCode;
use image::Luma;
use base64::Engine;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub id: String,
    pub name: String,
    pub tier: String,
    pub price: f64,
    pub duration_days: i32,
    pub features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    pub id: String,
    pub user_id: String,
    pub plan_id: String,
    pub amount: f64,
    pub status: PaymentStatus,
    pub payment_method: PaymentMethod,
    pub reference: String,
    pub qr_code: Option<String>,
    pub slip_path: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub verified_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PaymentStatus {
    Pending,
    Verified,
    Failed,
    Expired,
    Refunded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaymentMethod {
    QRCode,
    CreditCard,
    BankTransfer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub id: String,
    pub user_id: String,
    pub plan_id: String,
    pub tier: String,
    pub status: SubscriptionStatus,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QRCodePaymentData {
    pub qr_code: String,
    pub payment_id: String,
    pub amount: f64,
    pub reference: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlipVerificationRequest {
    pub payment_id: String,
    pub slip_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlipVerificationResult {
    pub verified: bool,
    pub payment_id: String,
    pub amount: f64,
    pub reference: String,
    pub timestamp: DateTime<Utc>,
}

pub struct BillingService {
    user_service: UserService,
    plans: Vec<Plan>,
}

impl BillingService {
    pub fn new(user_service: UserService) -> Self {
        let plans = vec![
            Plan {
                id: "premium-monthly".to_string(),
                name: "Premium".to_string(),
                tier: "premium".to_string(),
                price: 9.99,
                duration_days: 30,
                features: vec![
                    "Advanced AI features".to_string(),
                    "Higher rate limits".to_string(),
                    "Priority support".to_string(),
                    "Custom models".to_string(),
                    "10 concurrent requests".to_string(),
                ],
            },
            Plan {
                id: "premium-yearly".to_string(),
                name: "Premium (Yearly)".to_string(),
                tier: "premium".to_string(),
                price: 99.99,
                duration_days: 365,
                features: vec![
                    "Advanced AI features".to_string(),
                    "Higher rate limits".to_string(),
                    "Priority support".to_string(),
                    "Custom models".to_string(),
                    "10 concurrent requests".to_string(),
                    "2 months free".to_string(),
                ],
            },
            Plan {
                id: "ultra-monthly".to_string(),
                name: "Ultra".to_string(),
                tier: "ultra".to_string(),
                price: 29.99,
                duration_days: 30,
                features: vec![
                    "All Premium features".to_string(),
                    "Jailbreak prompt access".to_string(),
                    "Unlimited rate limits".to_string(),
                    "50 concurrent requests".to_string(),
                    "API access".to_string(),
                    "Dedicated support".to_string(),
                ],
            },
            Plan {
                id: "ultra-yearly".to_string(),
                name: "Ultra (Yearly)".to_string(),
                tier: "ultra".to_string(),
                price: 299.99,
                duration_days: 365,
                features: vec![
                    "All Premium features".to_string(),
                    "Jailbreak prompt access".to_string(),
                    "Unlimited rate limits".to_string(),
                    "50 concurrent requests".to_string(),
                    "API access".to_string(),
                    "Dedicated support".to_string(),
                    "3 months free".to_string(),
                ],
            },
        ];

        Self { user_service, plans }
    }

    pub fn get_plans(&self) -> Vec<Plan> {
        self.plans.clone()
    }

    pub fn get_plan(&self, plan_id: &str) -> Result<Plan> {
        self.plans
            .iter()
            .find(|p| p.id == plan_id)
            .cloned()
            .ok_or_else(|| anyhow!("Plan not found: {}", plan_id))
    }

    pub fn generate_qr_code(&self, plan_id: &str, amount: f64) -> Result<QRCodePaymentData> {
        let plan = self.get_plan(plan_id)?;
        let payment_id = Uuid::new_v4().to_string();
        let reference = self.generate_reference_number();

        // Generate QR code data
        let qr_data = format!(
            "PAYMENT|{}|{}|{}|{}",
            payment_id, reference, amount, plan.name
        );

        // Create QR code
        let qr = QrCode::new(&qr_data)?;
        let image = qr.render::<Luma<u8>>().min_dimensions(300, 300).build();

        // Convert to base64
        let mut buffer = Vec::new();
        image.write_png(&mut buffer)?;
        let base64_qr = base64::engine::general_purpose::STANDARD.encode(&buffer);

        let expires_at = Utc::now() + Duration::minutes(15);

        Ok(QRCodePaymentData {
            qr_code: format!("data:image/png;base64,{}", base64_qr),
            payment_id,
            amount,
            reference,
            expires_at,
        })
    }

    pub fn verify_payment_slip(
        &self,
        payment_id: &str,
        amount: f64,
        reference: &str,
    ) -> Result<SlipVerificationResult> {
        // In a real implementation, this would use OCR or manual verification
        // For now, we'll do basic validation

        if reference.len() < 10 {
            return Err(anyhow!("Invalid reference number"));
        }

        if amount <= 0.0 {
            return Err(anyhow!("Invalid amount"));
        }

        Ok(SlipVerificationResult {
            verified: true,
            payment_id: payment_id.to_string(),
            amount,
            reference: reference.to_string(),
            timestamp: Utc::now(),
        })
    }

    pub fn create_subscription(
        &self,
        user_id: &str,
        plan_id: &str,
    ) -> Result<Subscription> {
        let plan = self.get_plan(plan_id)?;
        let now = Utc::now();
        let end_date = now + Duration::days(plan.duration_days as i64);

        Ok(Subscription {
            id: Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            plan_id: plan_id.to_string(),
            tier: plan.tier,
            status: SubscriptionStatus::Active,
            start_date: now,
            end_date,
            auto_renew: true,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn check_subscription_expiry(&self, subscription: &Subscription) -> SubscriptionStatus {
        let now = Utc::now();

        if subscription.end_date < now {
            SubscriptionStatus::Expired
        } else if subscription.status == SubscriptionStatus::Cancelled {
            SubscriptionStatus::Cancelled
        } else {
            SubscriptionStatus::Active
        }
    }

    pub fn renew_subscription(&self, subscription: &mut Subscription) -> Result<()> {
        if subscription.auto_renew {
            subscription.start_date = subscription.end_date;
            subscription.end_date = subscription.end_date + Duration::days(30);
            subscription.status = SubscriptionStatus::Active;
            subscription.updated_at = Utc::now();
            Ok(())
        } else {
            Err(anyhow!("Auto-renewal is disabled for this subscription"))
        }
    }

    pub fn cancel_subscription(&self, subscription: &mut Subscription) -> Result<()> {
        subscription.status = SubscriptionStatus::Cancelled;
        subscription.updated_at = Utc::now();
        Ok(())
    }

    pub fn generate_reference_number(&self) -> String {
        let timestamp = Utc::now().timestamp_millis();
        let random = Uuid::new_v4().to_string()[0..8].to_string();
        format!("PAY-{}-{}", timestamp, random.to_uppercase())
    }

    pub fn calculate_discount(&self, plan_id: &str) -> f64 {
        // Apply discount for yearly plans
        if plan_id.contains("yearly") {
            0.15 // 15% discount
        } else {
            0.0
        }
    }

    pub fn get_payment_history(&self, user_id: &str) -> Result<Vec<Payment>> {
        // This would query the database in a real implementation
        // For now, return empty vector
        Ok(Vec::new())
    }

    pub fn validate_payment_amount(&self, plan_id: &str, amount: f64) -> Result<()> {
        let plan = self.get_plan(plan_id)?;
        let discount = self.calculate_discount(plan_id);
        let expected_amount = plan.price * (1.0 - discount);

        if (amount - expected_amount).abs() > 0.01 {
            return Err(anyhow!(
                "Amount mismatch. Expected: {}, Got: {}",
                expected_amount,
                amount
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_reference_number() {
        let pool = mr_darkpromth_db::DbPool::connect_lazy("postgres://postgres:postgres@localhost:5432/mr_darkpromth").unwrap();
        let repository = mr_darkpromth_db::UserRepository::new(pool);
        let user_service = UserService::new(repository, "test_secret".to_string());
        let billing = BillingService::new(user_service);
        let ref1 = billing.generate_reference_number();
        let ref2 = billing.generate_reference_number();

        assert!(ref1.starts_with("PAY-"));
        assert!(ref2.starts_with("PAY-"));
        assert_ne!(ref1, ref2);
    }

    #[test]
    fn test_get_plans() {
        let pool = mr_darkpromth_db::DbPool::connect_lazy("postgres://postgres:postgres@localhost:5432/mr_darkpromth").unwrap();
        let repository = mr_darkpromth_db::UserRepository::new(pool);
        let user_service = UserService::new(repository, "test_secret".to_string());
        let billing = BillingService::new(user_service);
        let plans = billing.get_plans();

        assert_eq!(plans.len(), 4);
        assert_eq!(plans[0].id, "premium-monthly");
        assert_eq!(plans[3].id, "ultra-yearly");
    }

    #[test]
    fn test_calculate_discount() {
        let pool = mr_darkpromth_db::DbPool::connect_lazy("postgres://postgres:postgres@localhost:5432/mr_darkpromth").unwrap();
        let repository = mr_darkpromth_db::UserRepository::new(pool);
        let user_service = UserService::new(repository, "test_secret".to_string());
        let billing = BillingService::new(user_service);

        assert_eq!(billing.calculate_discount("premium-monthly"), 0.0);
        assert_eq!(billing.calculate_discount("premium-yearly"), 0.15);
        assert_eq!(billing.calculate_discount("ultra-yearly"), 0.15);
    }

    #[test]
    fn test_create_subscription() {
        let pool = mr_darkpromth_db::DbPool::connect_lazy("postgres://postgres:postgres@localhost:5432/mr_darkpromth").unwrap();
        let repository = mr_darkpromth_db::UserRepository::new(pool);
        let user_service = UserService::new(repository, "test_secret".to_string());
        let billing = BillingService::new(user_service);
        let subscription = billing.create_subscription("user123", "premium-monthly").unwrap();

        assert_eq!(subscription.user_id, "user123");
        assert_eq!(subscription.tier, "premium");
        assert_eq!(subscription.status, SubscriptionStatus::Active);
        assert!(subscription.auto_renew);
    }

    #[test]
    fn test_check_subscription_expiry() {
        let pool = mr_darkpromth_db::DbPool::connect_lazy("postgres://postgres:postgres@localhost:5432/mr_darkpromth").unwrap();
        let repository = mr_darkpromth_db::UserRepository::new(pool);
        let user_service = UserService::new(repository, "test_secret".to_string());
        let billing = BillingService::new(user_service);
        let mut subscription = billing.create_subscription("user123", "premium-monthly").unwrap();

        // Subscription should be active
        assert_eq!(billing.check_subscription_expiry(&subscription), SubscriptionStatus::Active);

        // Set end date to past
        subscription.end_date = Utc::now() - Duration::days(1);
        assert_eq!(billing.check_subscription_expiry(&subscription), SubscriptionStatus::Expired);
    }
}
