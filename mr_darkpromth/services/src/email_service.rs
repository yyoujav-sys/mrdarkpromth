use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use lettre::transport::smtp::SmtpTransport;
use lettre::{Message, Transport};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EmailVerificationToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: String,
    pub email: String,
    pub used: bool,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PasswordResetToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: String,
    pub used: bool,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub from_email: String,
    pub from_name: String,
    pub verification_expiry_hours: i64,
    pub reset_expiry_hours: i64,
}

impl Default for EmailConfig {
    fn default() -> Self {
        Self {
            smtp_host: std::env::var("SMTP_HOST").unwrap_or_else(|_| "localhost".to_string()),
            smtp_port: std::env::var("SMTP_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(587),
            smtp_username: std::env::var("SMTP_USERNAME").unwrap_or_default(),
            smtp_password: std::env::var("SMTP_PASSWORD").unwrap_or_default(),
            from_email: std::env::var("FROM_EMAIL")
                .unwrap_or_else(|_| "noreply@mrdarkpromth.com".to_string()),
            from_name: "Mr.DarkPromth".to_string(),
            verification_expiry_hours: 24,
            reset_expiry_hours: 1,
        }
    }
}

pub struct EmailService {
    config: EmailConfig,
    smtp: Option<SmtpTransport>,
    pool: sqlx::PgPool,
}

impl EmailService {
    pub fn new(config: EmailConfig, pool: sqlx::PgPool) -> Result<Self> {
        let smtp = if !config.smtp_host.is_empty() && !config.smtp_username.is_empty() {
            Some(
                SmtpTransport::relay(&config.smtp_host)?
                    .port(config.smtp_port)
                    .credentials(lettre::transport::smtp::authentication::Credentials::new(
                        config.smtp_username.clone().into(),
                        config.smtp_password.clone().into(),
                    ))
                    .build(),
            )
        } else {
            None
        };

        Ok(Self { config, smtp, pool })
    }

    pub fn new_from_env(pool: sqlx::PgPool) -> Result<Self> {
        Self::new(EmailConfig::default(), pool)
    }

    pub async fn create_verification_token(&self, user_id: Uuid, email: &str) -> Result<EmailVerificationToken> {
        let token = Uuid::new_v4().to_string();
        let token_id = Uuid::new_v4();
        let expires_at = Utc::now() + Duration::hours(self.config.verification_expiry_hours);

        let verification_token = sqlx::query_as::<_, EmailVerificationToken>(
            r#"
            INSERT INTO email_verification_tokens (id, user_id, token, email, expires_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#
        )
        .bind(token_id)
        .bind(user_id)
        .bind(&token)
        .bind(email)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(verification_token)
    }

    pub async fn create_password_reset_token(&self, user_id: Uuid) -> Result<PasswordResetToken> {
        let token = Uuid::new_v4().to_string();
        let token_id = Uuid::new_v4();
        let expires_at = Utc::now() + Duration::hours(self.config.reset_expiry_hours);

        let reset_token = sqlx::query_as::<_, PasswordResetToken>(
            r#"
            INSERT INTO password_reset_tokens (id, user_id, token, expires_at)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#
        )
        .bind(token_id)
        .bind(user_id)
        .bind(&token)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(reset_token)
    }

    pub async fn verify_email_token(&self, token: &str) -> Result<EmailVerificationToken> {
        let mut verification_token: EmailVerificationToken = sqlx::query_as(
            "SELECT * FROM email_verification_tokens WHERE token = $1"
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| anyhow!("Invalid or expired token"))?;

        if verification_token.used {
            return Err(anyhow!("Token already verified"));
        }

        if Utc::now() > verification_token.expires_at {
            return Err(anyhow!("Token has expired"));
        }

        verification_token = sqlx::query_as(
            r#"
            UPDATE email_verification_tokens 
            SET used = true, used_at = NOW() 
            WHERE token = $1 
            RETURNING *
            "#
        )
        .bind(token)
        .fetch_one(&self.pool)
        .await?;

        sqlx::query("UPDATE users SET email_verified = true WHERE id = $1")
            .bind(verification_token.user_id)
            .execute(&self.pool)
            .await?;

        Ok(verification_token)
    }

    pub async fn verify_password_reset_token(&self, token: &str) -> Result<PasswordResetToken> {
        let mut reset_token: PasswordResetToken = sqlx::query_as(
            "SELECT * FROM password_reset_tokens WHERE token = $1"
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| anyhow!("Invalid or expired token"))?;

        if reset_token.used {
            return Err(anyhow!("Token has already been used"));
        }

        if Utc::now() > reset_token.expires_at {
            return Err(anyhow!("Token has expired"));
        }

        reset_token = sqlx::query_as(
            r#"
            UPDATE password_reset_tokens 
            SET used = true, used_at = NOW() 
            WHERE token = $1 
            RETURNING *
            "#
        )
        .bind(token)
        .fetch_one(&self.pool)
        .await?;

        Ok(reset_token)
    }

    pub fn send_verification_email(
        &self,
        email: &str,
        username: &str,
        token: &str,
        verification_url: &str,
    ) -> Result<()> {
        if self.smtp.is_none() {
            log::warn!("SMTP not configured, skipping email send");
            return Ok(());
        }

        let full_url = format!("{}?token={}&email={}", verification_url, token, email);
        let html_body = self.render_verification_email(username, &full_url);

        let message = Message::builder()
            .from(
                format!("{} <{}>", self.config.from_name, self.config.from_email)
                    .parse()?,
            )
            .to(email.parse()?)
            .subject("Verify Your Email - Mr.DarkPromth")
            .multipart(lettre::message::MultiPart::alternative().singlepart(
                lettre::message::SinglePart::html(html_body),
            ))?;

        if let Some(smtp) = &self.smtp {
            let _ = Transport::send(smtp, &message)?;
        }

        Ok(())
    }

    pub fn send_password_reset_email(
        &self,
        email: &str,
        username: &str,
        token: &str,
        reset_url: &str,
    ) -> Result<()> {
        if self.smtp.is_none() {
            log::warn!("SMTP not configured, skipping email send");
            return Ok(());
        }

        let full_url = format!("{}?token={}", reset_url, token);
        let html_body = self.render_password_reset_email(username, &full_url);

        let message = Message::builder()
            .from(
                format!("{} <{}>", self.config.from_name, self.config.from_email)
                    .parse()?,
            )
            .to(email.parse()?)
            .subject("Reset Your Password - Mr.DarkPromth")
            .multipart(lettre::message::MultiPart::alternative().singlepart(
                lettre::message::SinglePart::html(html_body),
            ))?;

        if let Some(smtp) = &self.smtp {
            let _ = Transport::send(smtp, &message)?;
        }

        Ok(())
    }

    pub fn send_subscription_confirmation_email(
        &self,
        email: &str,
        username: &str,
        plan_name: &str,
        amount: f64,
    ) -> Result<()> {
        if self.smtp.is_none() {
            log::warn!("SMTP not configured, skipping email send");
            return Ok(());
        }

        let html_body = self.render_subscription_email(username, plan_name, amount);

        let message = Message::builder()
            .from(
                format!("{} <{}>", self.config.from_name, self.config.from_email)
                    .parse()?,
            )
            .to(email.parse()?)
            .subject("Subscription Confirmed - Mr.DarkPromth")
            .multipart(lettre::message::MultiPart::alternative().singlepart(
                lettre::message::SinglePart::html(html_body),
            ))?;

        if let Some(smtp) = &self.smtp {
            let _ = Transport::send(smtp, &message)?;
        }

        Ok(())
    }

    fn render_verification_email(&self, username: &str, verification_url: &str) -> String {
        format!(
            r#"
            <!DOCTYPE html>
            <html>
            <head>
                <style>
                    body {{ font-family: Arial, sans-serif; }}
                    .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
                    .header {{ background-color: #a78bfa; color: white; padding: 20px; border-radius: 5px; }}
                    .content {{ padding: 20px; background-color: #f5f5f5; }}
                    .button {{ background-color: #a78bfa; color: white; padding: 10px 20px; text-decoration: none; border-radius: 5px; display: inline-block; }}
                </style>
            </head>
            <body>
                <div class="container">
                    <div class="header">
                        <h1>Welcome to Mr.DarkPromth, {}</h1>
                    </div>
                    <div class="content">
                        <p>Thank you for signing up! Please verify your email address by clicking the button below:</p>
                        <p><a href="{}" class="button">Verify Email</a></p>
                        <p>Or copy and paste this link in your browser:</p>
                        <p><code>{}</code></p>
                        <p>This link will expire in 24 hours.</p>
                        <p>Best regards,<br>Mr.DarkPromth Team</p>
                    </div>
                </div>
            </body>
            </html>
            "#,
            username, verification_url, verification_url
        )
    }

    fn render_password_reset_email(&self, username: &str, reset_url: &str) -> String {
        format!(
            r#"
            <!DOCTYPE html>
            <html>
            <head>
                <style>
                    body {{ font-family: Arial, sans-serif; }}
                    .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
                    .header {{ background-color: #a78bfa; color: white; padding: 20px; border-radius: 5px; }}
                    .content {{ padding: 20px; background-color: #f5f5f5; }}
                    .button {{ background-color: #a78bfa; color: white; padding: 10px 20px; text-decoration: none; border-radius: 5px; display: inline-block; }}
                </style>
            </head>
            <body>
                <div class="container">
                    <div class="header">
                        <h1>Password Reset Request</h1>
                    </div>
                    <div class="content">
                        <p>Hi {},</p>
                        <p>We received a request to reset your password. Click the button below to create a new password:</p>
                        <p><a href="{}" class="button">Reset Password</a></p>
                        <p>Or copy and paste this link in your browser:</p>
                        <p><code>{}</code></p>
                        <p>This link will expire in 1 hour.</p>
                        <p>If you didn't request a password reset, please ignore this email.</p>
                        <p>Best regards,<br>Mr.DarkPromth Team</p>
                    </div>
                </div>
            </body>
            </html>
            "#,
            username, reset_url, reset_url
        )
    }

    fn render_subscription_email(&self, username: &str, plan_name: &str, amount: f64) -> String {
        format!(
            r#"
            <!DOCTYPE html>
            <html>
            <head>
                <style>
                    body {{ font-family: Arial, sans-serif; }}
                    .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
                    .header {{ background-color: #a78bfa; color: white; padding: 20px; border-radius: 5px; }}
                    .content {{ padding: 20px; background-color: #f5f5f5; }}
                </style>
            </head>
            <body>
                <div class="container">
                    <div class="header">
                        <h1>Subscription Confirmed</h1>
                    </div>
                    <div class="content">
                        <p>Hi {},</p>
                        <p>Thank you for upgrading to the <strong>{}</strong> plan!</p>
                        <p><strong>Amount Charged:</strong> ${:.2}</p>
                        <p>Your subscription is now active and you can enjoy all the premium features.</p>
                        <p>You can manage your subscription at any time from your account settings.</p>
                        <p>Best regards,<br>Mr.DarkPromth Team</p>
                    </div>
                </div>
            </body>
            </html>
            "#,
            username, plan_name, amount
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_verification_token() {
        let pool = sqlx::PgPool::connect("postgresql://postgres:postgres@localhost:5432/mrdarkpromth").await.unwrap();
        
        // Use existing admin user
        let admin_user: (Uuid,) = sqlx::query_as("SELECT id FROM users WHERE username = 'admin' LIMIT 1")
            .fetch_one(&pool)
            .await
            .expect("Admin user should exist");
        
        let service = EmailService::new_from_env(pool).unwrap();
        let token = service.create_verification_token(admin_user.0, "admin@mrdarkpromth.ai").await.unwrap();

        assert_eq!(token.user_id, admin_user.0);
        assert_eq!(token.email, "admin@mrdarkpromth.ai");
        assert!(!token.used);
        assert!(token.expires_at > Utc::now());
    }

    #[tokio::test]
    async fn test_create_password_reset_token() {
        let pool = sqlx::PgPool::connect("postgresql://postgres:postgres@localhost:5432/mrdarkpromth").await.unwrap();
        
        // Use existing admin user
        let admin_user: (Uuid,) = sqlx::query_as("SELECT id FROM users WHERE username = 'admin' LIMIT 1")
            .fetch_one(&pool)
            .await
            .expect("Admin user should exist");
        
        let service = EmailService::new_from_env(pool).unwrap();
        let token = service.create_password_reset_token(admin_user.0).await.unwrap();

        assert_eq!(token.user_id, admin_user.0);
        assert!(!token.used);
        assert!(token.expires_at > Utc::now());
    }
}
