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

    pub async fn send_verification_email(&self, to_email: &str, username: &str, token: &str) -> Result<()> {
        let verification_url = format!("https://mrdarkpromth.ai/verify-email?token={}", token);
        let html = format!(r#"<!DOCTYPE html><html><body><h2>Hello {}</h2><p>Click to verify: <a href="{}">{}</a></p></body></html>"#, username, verification_url, verification_url);
        
        if let Some(smtp) = &self.smtp {
            let email = Message::builder()
                .from(format!("{} <{}>", self.config.from_name, self.config.from_email).parse()?)
                .to(to_email.parse()?)
                .subject("Verify your email - Mr.DarkPromth")
                .header(lettre::message::header::ContentType::TEXT_HTML)
                .body(html)?;
            
            smtp.send(&email)?;
        }
        Ok(())
    }

    pub async fn send_password_reset_email(&self, to_email: &str, username: &str, token: &str) -> Result<()> {
        let reset_url = format!("https://mrdarkpromth.ai/reset-password?token={}", token);
        let html = format!(r#"<!DOCTYPE html><html><body><h2>Hello {}</h2><p>Reset password: <a href="{}">{}</a></p><p>Expires in 1 hour.</p></body></html>"#, username, reset_url, reset_url);
        
        if let Some(smtp) = &self.smtp {
            let email = Message::builder()
                .from(format!("{} <{}>", self.config.from_name, self.config.from_email).parse()?)
                .to(to_email.parse()?)
                .subject("Password Reset - Mr.DarkPromth")
                .header(lettre::message::header::ContentType::TEXT_HTML)
                .body(html)?;
            
            smtp.send(&email)?;
        }
        Ok(())
    }
}
