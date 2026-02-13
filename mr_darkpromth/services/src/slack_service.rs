use serde::{Deserialize, Serialize};
use std::process::Command;
use anyhow::Result;
use log::{info, error};

#[derive(Debug, Clone)]
pub struct SlackService {
    pub enabled: bool,
    pub alerts_channel: String,
    pub metrics_channel: String,
}

impl SlackService {
    pub fn new_from_env() -> Self {
        let enabled = std::env::var("SLACK_ENABLED").unwrap_or_default() == "true";
        let alerts_channel = std::env::var("SLACK_ALERTS_CHANNEL_ID").unwrap_or_default();
        let metrics_channel = std::env::var("SLACK_METRICS_CHANNEL_ID").unwrap_or_default();

        Self {
            enabled,
            alerts_channel,
            metrics_channel,
        }
    }

    /// Send a message to a Slack channel using manus-mcp-cli
    pub async fn send_message(&self, channel_id: &str, text: &str) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let input = serde_json::json!({
            "channel_id": channel_id,
            "text": text
        });

        let output = Command::new("manus-mcp-cli")
            .args(&[
                "tool", 
                "call", 
                "slack_send_message", 
                "--server", 
                "slack", 
                "--input", 
                &input.to_string()
            ])
            .output();

        match output {
            Ok(out) => {
                if out.status.success() {
                    info!("Slack message sent to channel {}", channel_id);
                    Ok(())
                } else {
                    let err = String::from_utf8_lossy(&out.stderr);
                    error!("Failed to send Slack message: {}", err);
                    Err(anyhow::anyhow!("Slack API error: {}", err))
                }
            }
            Err(e) => {
                error!("Failed to execute manus-mcp-cli: {}", e);
                Err(anyhow::anyhow!("Command execution error: {}", e))
            }
        }
    }

    pub async fn notify_alert(&self, message: &str) -> Result<()> {
        self.send_message(&self.alerts_channel, &format!("🚨 *System Alert*\n{}", message)).await
    }

    pub async fn notify_tier_upgrade(&self, email: &str, tier: &str) -> Result<()> {
        let msg = format!("🎉 *Tier Upgrade*\nUser: `{}`\nNew Tier: *{}*", email, tier);
        self.send_message(&self.alerts_channel, &msg).await
    }

    pub async fn notify_metrics(&self, report: &str) -> Result<()> {
        self.send_message(&self.metrics_channel, &format!("📊 *Hourly Metrics*\n{}", report)).await
    }
}
