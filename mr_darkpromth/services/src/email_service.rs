use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use lettre::transport::smtp::SmtpTransport;
use lettre::{Message, Transport};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailVerificationToken {
    pub id: String,
    pub user_id: String,
    pub email: String,
    pub token: String,
    pub verified: bool,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub verified_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordResetToken {
    pub id: String,
    pub user_id: String,
    pub email: String,
    pub token: String,
    pub used: bool,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
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
}

impl EmailService {
    pub fn new(config: EmailConfig) -> Result<Self> {
        // Initialize SMTP transport
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

        Ok(Self { config, smtp })
    }

    pub fn new_from_env() -> Result<Self> {
        Self::new(EmailConfig::default())
    }

    pub fn generate_verification_token(&self, user_id: &str, email: &str) -> EmailVerificationToken {
        let token = Uuid::new_v4().to_string();
        let now = Utc::now();

        EmailVerificationToken {
            id: Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            email: email.to_string(),
            token,
            verified: false,
            created_at: now,
            expires_at: now + Duration::hours(self.config.verification_expiry_hours),
            verified_at: None,
        }
    }

    pub fn generate_password_reset_token(&self, user_id: &str, email: &str) -> PasswordResetToken {
        let token = Uuid::new_v4().to_string();
        let now = Utc::now();

        PasswordResetToken {
            id: Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            email: email.to_string(),
            token,
            used: false,
            created_at: now,
            expires_at: now + Duration::hours(self.config.reset_expiry_hours),
            used_at: None,
        }
    }

    pub fn verify_token(&self, token: &EmailVerificationToken) -> Result<()> {
        if token.verified {
            return Err(anyhow!("Token already verified"));
        }

        if Utc::now() > token.expires_at {
            return Err(anyhow!("Token has expired"));
        }

        Ok(())
    }

    pub fn verify_reset_token(&self, token: &PasswordResetToken) -> Result<()> {
        if token.used {
            return Err(anyhow!("Token has already been used"));
        }

        if Utc::now() > token.expires_at {
            return Err(anyhow!("Token has expired"));
        }

        Ok(())
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

    #[test]
    fn test_generate_verification_token() {
        let config = EmailConfig::default();
        let service = EmailService::new(config).unwrap();
        let token = service.generate_verification_token("user123", "user@example.com");

        assert_eq!(token.user_id, "user123");
        assert_eq!(token.email, "user@example.com");
        assert!(!token.verified);
        assert!(token.expires_at > Utc::now());
    }

    #[test]
    fn test_generate_password_reset_token() {
        let config = EmailConfig::default();
        let service = EmailService::new(config).unwrap();
        let token = service.generate_password_reset_token("user123", "user@example.com");

        assert_eq!(token.user_id, "user123");
        assert_eq!(token.email, "user@example.com");
        assert!(!token.used);
        assert!(token.expires_at > Utc::now());
    }

    #[test]
    fn test_verify_token() {
        let config = EmailConfig::default();
        let service = EmailService::new(config).unwrap();
        let token = service.generate_verification_token("user123", "user@example.com");

        assert!(service.verify_token(&token).is_ok());
    }

    #[test]
    fn test_verify_expired_token() {
        let config = EmailConfig::default();
        let service = EmailService::new(config).unwrap();
        let mut token = service.generate_verification_token("user123", "user@example.com");
        token.expires_at = Utc::now() - Duration::hours(1);

        assert!(service.verify_token(&token).is_err());
    }
}
