use serde::{Deserialize, Serialize};
use anyhow::Result;
use log::{info, error};
use reqwest::Client;

#[derive(Debug, Clone)]
pub struct SlackService {
    pub enabled: bool,
    pub bot_token: String,
    pub alerts_channel: String,
    pub metrics_channel: String,
    pub admin_user_id: String,
    client: Client,
}

#[derive(Debug, Serialize)]
struct SlackMessage {
    channel: String,
    text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    blocks: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Deserialize)]
struct SlackResponse {
    ok: bool,
    #[serde(default)]
    error: String,
    #[serde(default)]
    ts: String,
}

impl SlackService {
    pub fn new_from_env() -> Self {
        let enabled = std::env::var("SLACK_ENABLED").unwrap_or_default() == "true";
        let bot_token = std::env::var("SLACK_BOT_TOKEN").unwrap_or_default();
        let alerts_channel = std::env::var("SLACK_ALERTS_CHANNEL_ID").unwrap_or_default();
        let metrics_channel = std::env::var("SLACK_METRICS_CHANNEL_ID").unwrap_or_default();
        let admin_user_id = std::env::var("SLACK_ADMIN_USER_ID").unwrap_or_default();
        
        Self {
            enabled,
            bot_token,
            alerts_channel,
            metrics_channel,
            admin_user_id,
            client: Client::new(),
        }
    }

    /// Send a message to a Slack channel using Direct API
    pub async fn send_message(&self, channel_id: &str, text: &str) -> Result<()> {
        if !self.enabled {
            info!("Slack service is disabled, skipping message");
            return Ok(());
        }

        if self.bot_token.is_empty() {
            error!("Slack bot token is not configured");
            return Err(anyhow::anyhow!("Slack bot token not configured"));
        }

        let message = SlackMessage {
            channel: channel_id.to_string(),
            text: text.to_string(),
            blocks: None,
        };

        let response = self.client
            .post("https://slack.com/api/chat.postMessage")
            .header("Authorization", format!("Bearer {}", self.bot_token))
            .header("Content-Type", "application/json")
            .json(&message)
            .send()
            .await?;

        let slack_response: SlackResponse = response.json().await?;

        if slack_response.ok {
            info!("Slack message sent to channel {} (ts: {})", channel_id, slack_response.ts);
            Ok(())
        } else {
            error!("Failed to send Slack message: {}", slack_response.error);
            Err(anyhow::anyhow!("Slack API error: {}", slack_response.error))
        }
    }

    /// Send a formatted message with blocks
    pub async fn send_formatted_message(
        &self,
        channel_id: &str,
        text: &str,
        blocks: Vec<serde_json::Value>,
    ) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        if self.bot_token.is_empty() {
            return Err(anyhow::anyhow!("Slack bot token not configured"));
        }

        let message = SlackMessage {
            channel: channel_id.to_string(),
            text: text.to_string(),
            blocks: Some(blocks),
        };

        let response = self.client
            .post("https://slack.com/api/chat.postMessage")
            .header("Authorization", format!("Bearer {}", self.bot_token))
            .header("Content-Type", "application/json")
            .json(&message)
            .send()
            .await?;

        let slack_response: SlackResponse = response.json().await?;

        if slack_response.ok {
            info!("Slack formatted message sent to channel {}", channel_id);
            Ok(())
        } else {
            error!("Failed to send formatted message: {}", slack_response.error);
            Err(anyhow::anyhow!("Slack API error: {}", slack_response.error))
        }
    }

    pub async fn notify_alert(&self, message: &str) -> Result<()> {
        let formatted = format!("🚨 *System Alert*\n{}", message);
        self.send_message(&self.alerts_channel, &formatted).await
    }

    pub async fn notify_tier_upgrade(&self, email: &str, tier: &str) -> Result<()> {
        let msg = format!("🎉 *Tier Upgrade*\nUser: `{}`\nNew Tier: *{}*", email, tier);
        self.send_message(&self.alerts_channel, &msg).await
    }

    pub async fn notify_metrics(&self, report: &str) -> Result<()> {
        let formatted = format!("📊 *Hourly Metrics*\n{}", report);
        self.send_message(&self.metrics_channel, &formatted).await
    }

    pub async fn notify_api_key_rotation(&self, provider: &str, rotated_count: usize) -> Result<()> {
        let msg = format!(
            "🔄 *API Key Rotation*\nProvider: `{}`\nKeys Rotated: {}",
            provider, rotated_count
        );
        self.send_message(&self.alerts_channel, &msg).await
    }

    pub async fn notify_error(&self, service: &str, error: &str) -> Result<()> {
        let msg = format!("❌ *Error in {}*\n```\n{}\n```", service, error);
        self.send_message(&self.alerts_channel, &msg).await
    }

    pub async fn notify_deployment(&self, version: &str, status: &str) -> Result<()> {
        let emoji = if status == "success" { "✅" } else { "❌" };
        let msg = format!(
            "{} *Deployment {}*\nVersion: `{}`",
            emoji, status, version
        );
        self.send_message(&self.alerts_channel, &msg).await
    }

    /// Direct message to admin user
    pub async fn dm_admin(&self, message: &str) -> Result<()> {
        if self.admin_user_id.is_empty() {
            return Err(anyhow::anyhow!("Admin user ID not configured"));
        }
        self.send_message(&self.admin_user_id, message).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_slack_service_disabled() {
        let service = SlackService {
            enabled: false,
            bot_token: "test".to_string(),
            alerts_channel: "C123".to_string(),
            metrics_channel: "C456".to_string(),
            admin_user_id: "U123".to_string(),
            client: Client::new(),
        };

        let result = service.send_message("C123", "test").await;
        assert!(result.is_ok());
    }
}
