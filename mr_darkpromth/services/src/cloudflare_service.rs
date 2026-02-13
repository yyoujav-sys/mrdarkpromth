use serde::{Deserialize, Serialize};
use std::process::Command;
use anyhow::Result;
use log::{info, error};

#[derive(Debug, Clone)]
pub struct CloudflareService {
    pub enabled: bool,
    pub account_id: String,
    pub r2_bucket: String,
}

impl CloudflareService {
    pub fn new_from_env() -> Self {
        let enabled = std::env::var("CLOUDFLARE_ENABLED").unwrap_or_default() == "true";
        let account_id = std::env::var("CLOUDFLARE_ACCOUNT_ID").unwrap_or_default();
        let r2_bucket = std::env::var("CLOUDFLARE_R2_BUCKET").unwrap_or_default();

        Self {
            enabled,
            account_id,
            r2_bucket,
        }
    }

    /// List R2 buckets using manus-mcp-cli
    pub async fn list_buckets(&self) -> Result<serde_json::Value> {
        if !self.enabled {
            return Err(anyhow::anyhow!("Cloudflare service is disabled"));
        }

        let output = Command::new("manus-mcp-cli")
            .args(&[
                "tool", 
                "call", 
                "r2_buckets_list", 
                "--server", 
                "cloudflare", 
                "--input", 
                "{}"
            ])
            .output()?;

        if output.status.success() {
            let val = serde_json::from_slice(&output.stdout)?;
            Ok(val)
        } else {
            let err = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("Cloudflare API error: {}", err))
        }
    }

    /// Query D1 database using manus-mcp-cli
    pub async fn query_d1(&self, database_id: &str, query: &str) -> Result<serde_json::Value> {
        if !self.enabled {
            return Err(anyhow::anyhow!("Cloudflare service is disabled"));
        }

        let input = serde_json::json!({
            "databaseId": database_id,
            "query": query
        });

        let output = Command::new("manus-mcp-cli")
            .args(&[
                "tool", 
                "call", 
                "d1_database_query", 
                "--server", 
                "cloudflare", 
                "--input", 
                &input.to_string()
            ])
            .output()?;

        if output.status.success() {
            let val = serde_json::from_slice(&output.stdout)?;
            Ok(val)
        } else {
            let err = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("Cloudflare D1 error: {}", err))
        }
    }

    /// Get KV value using manus-mcp-cli
    pub async fn get_kv(&self, namespace_id: &str, key: &str) -> Result<String> {
        if !self.enabled {
            return Err(anyhow::anyhow!("Cloudflare service is disabled"));
        }

        let input = serde_json::json!({
            "namespaceId": namespace_id,
            "key": key
        });

        let output = Command::new("manus-mcp-cli")
            .args(&[
                "tool", 
                "call", 
                "kv_namespace_get", 
                "--server", 
                "cloudflare", 
                "--input", 
                &input.to_string()
            ])
            .output()?;

        if output.status.success() {
            let val: serde_json::Value = serde_json::from_slice(&output.stdout)?;
            Ok(val["value"].as_str().unwrap_or_default().to_string())
        } else {
            let err = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("Cloudflare KV error: {}", err))
        }
    }
}
