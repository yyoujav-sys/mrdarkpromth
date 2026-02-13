use serde::{Deserialize, Serialize};
use anyhow::Result;
use log::{info, error};
use reqwest::Client;

#[derive(Debug, Clone)]
pub struct CloudflareService {
    pub enabled: bool,
    pub api_token: String,
    pub account_id: String,
    pub r2_bucket: String,
    client: Client,
}

#[derive(Debug, Serialize)]
struct CloudflareRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    query: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CloudflareResponse {
    success: bool,
    #[serde(default)]
    errors: Vec<CloudflareError>,
    #[serde(default)]
    result: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct CloudflareError {
    code: u32,
    message: String,
}

impl CloudflareService {
    pub fn new_from_env() -> Self {
        let enabled = std::env::var("CLOUDFLARE_ENABLED").unwrap_or_default() == "true";
        let api_token = std::env::var("CLOUDFLARE_API_TOKEN").unwrap_or_default();
        let account_id = std::env::var("CLOUDFLARE_ACCOUNT_ID").unwrap_or_default();
        let r2_bucket = std::env::var("CLOUDFLARE_R2_BUCKET").unwrap_or_default();

        Self {
            enabled,
            api_token,
            account_id,
            r2_bucket,
            client: Client::new(),
        }
    }

    /// List R2 buckets using Direct API
    pub async fn list_buckets(&self) -> Result<serde_json::Value> {
        if !self.enabled {
            return Err(anyhow::anyhow!("Cloudflare service is disabled"));
        }

        if self.api_token.is_empty() || self.account_id.is_empty() {
            return Err(anyhow::anyhow!("Cloudflare credentials not configured"));
        }

        let url = format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/r2/buckets",
            self.account_id
        );

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .send()
            .await?;

        let cf_response: CloudflareResponse = response.json().await?;

        if cf_response.success {
            info!("Successfully listed R2 buckets");
            Ok(cf_response.result)
        } else {
            let error_msg = cf_response
                .errors
                .first()
                .map(|e| e.message.clone())
                .unwrap_or_else(|| "Unknown error".to_string());
            error!("Cloudflare API error: {}", error_msg);
            Err(anyhow::anyhow!("Cloudflare API error: {}", error_msg))
        }
    }

    /// Upload file to R2 bucket
    pub async fn upload_to_r2(
        &self,
        file_path: &str,
        file_content: &[u8],
    ) -> Result<String> {
        if !self.enabled {
            return Err(anyhow::anyhow!("Cloudflare service is disabled"));
        }

        if self.api_token.is_empty() || self.account_id.is_empty() || self.r2_bucket.is_empty() {
            return Err(anyhow::anyhow!("Cloudflare credentials not configured"));
        }

        let url = format!(
            "https://{}.r2.cloudflarestorage.com/{}",
            self.account_id, file_path
        );

        let response = self.client
            .put(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .body(file_content.to_vec())
            .send()
            .await?;

        if response.status().is_success() {
            info!("Successfully uploaded file to R2: {}", file_path);
            Ok(format!("https://{}.r2.cloudflarestorage.com/{}", self.account_id, file_path))
        } else {
            let error_text = response.text().await.unwrap_or_default();
            error!("Failed to upload to R2: {}", error_text);
            Err(anyhow::anyhow!("R2 upload failed: {}", error_text))
        }
    }

    /// Get file from R2 bucket
    pub async fn get_from_r2(&self, file_path: &str) -> Result<Vec<u8>> {
        if !self.enabled {
            return Err(anyhow::anyhow!("Cloudflare service is disabled"));
        }

        if self.api_token.is_empty() || self.account_id.is_empty() {
            return Err(anyhow::anyhow!("Cloudflare credentials not configured"));
        }

        let url = format!(
            "https://{}.r2.cloudflarestorage.com/{}",
            self.account_id, file_path
        );

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .send()
            .await?;

        if response.status().is_success() {
            let content = response.bytes().await?.to_vec();
            info!("Successfully retrieved file from R2: {}", file_path);
            Ok(content)
        } else {
            error!("Failed to get file from R2: {}", response.status());
            Err(anyhow::anyhow!("R2 get failed: {}", response.status()))
        }
    }

    /// Delete file from R2 bucket
    pub async fn delete_from_r2(&self, file_path: &str) -> Result<()> {
        if !self.enabled {
            return Err(anyhow::anyhow!("Cloudflare service is disabled"));
        }

        if self.api_token.is_empty() || self.account_id.is_empty() {
            return Err(anyhow::anyhow!("Cloudflare credentials not configured"));
        }

        let url = format!(
            "https://{}.r2.cloudflarestorage.com/{}",
            self.account_id, file_path
        );

        let response = self.client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .send()
            .await?;

        if response.status().is_success() {
            info!("Successfully deleted file from R2: {}", file_path);
            Ok(())
        } else {
            error!("Failed to delete file from R2: {}", response.status());
            Err(anyhow::anyhow!("R2 delete failed: {}", response.status()))
        }
    }

    /// Query D1 database using Direct API
    pub async fn query_d1(&self, database_id: &str, query: &str) -> Result<serde_json::Value> {
        if !self.enabled {
            return Err(anyhow::anyhow!("Cloudflare service is disabled"));
        }

        if self.api_token.is_empty() || self.account_id.is_empty() {
            return Err(anyhow::anyhow!("Cloudflare credentials not configured"));
        }

        let url = format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/d1/database/{}/query",
            self.account_id, database_id
        );

        let request_body = serde_json::json!({
            "sql": query
        });

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .json(&request_body)
            .send()
            .await?;

        let cf_response: CloudflareResponse = response.json().await?;

        if cf_response.success {
            info!("Successfully queried D1 database");
            Ok(cf_response.result)
        } else {
            let error_msg = cf_response
                .errors
                .first()
                .map(|e| e.message.clone())
                .unwrap_or_else(|| "Unknown error".to_string());
            error!("D1 query error: {}", error_msg);
            Err(anyhow::anyhow!("D1 query error: {}", error_msg))
        }
    }

    /// Get KV namespace value using Direct API
    pub async fn get_kv(&self, namespace_id: &str, key: &str) -> Result<String> {
        if !self.enabled {
            return Err(anyhow::anyhow!("Cloudflare service is disabled"));
        }

        if self.api_token.is_empty() || self.account_id.is_empty() {
            return Err(anyhow::anyhow!("Cloudflare credentials not configured"));
        }

        let url = format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/storage/kv/namespaces/{}/values/{}",
            self.account_id, namespace_id, key
        );

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .send()
            .await?;

        if response.status().is_success() {
            let value = response.text().await?;
            info!("Successfully retrieved KV value: {}", key);
            Ok(value)
        } else {
            error!("Failed to get KV value: {}", response.status());
            Err(anyhow::anyhow!("KV get failed: {}", response.status()))
        }
    }

    /// Set KV namespace value using Direct API
    pub async fn set_kv(&self, namespace_id: &str, key: &str, value: &str) -> Result<()> {
        if !self.enabled {
            return Err(anyhow::anyhow!("Cloudflare service is disabled"));
        }

        if self.api_token.is_empty() || self.account_id.is_empty() {
            return Err(anyhow::anyhow!("Cloudflare credentials not configured"));
        }

        let url = format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/storage/kv/namespaces/{}/values/{}",
            self.account_id, namespace_id, key
        );

        let response = self.client
            .put(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .body(value.to_string())
            .send()
            .await?;

        if response.status().is_success() {
            info!("Successfully set KV value: {}", key);
            Ok(())
        } else {
            error!("Failed to set KV value: {}", response.status());
            Err(anyhow::anyhow!("KV set failed: {}", response.status()))
        }
    }

    /// Delete KV namespace value using Direct API
    pub async fn delete_kv(&self, namespace_id: &str, key: &str) -> Result<()> {
        if !self.enabled {
            return Err(anyhow::anyhow!("Cloudflare service is disabled"));
        }

        if self.api_token.is_empty() || self.account_id.is_empty() {
            return Err(anyhow::anyhow!("Cloudflare credentials not configured"));
        }

        let url = format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/storage/kv/namespaces/{}/values/{}",
            self.account_id, namespace_id, key
        );

        let response = self.client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .send()
            .await?;

        if response.status().is_success() {
            info!("Successfully deleted KV value: {}", key);
            Ok(())
        } else {
            error!("Failed to delete KV value: {}", response.status());
            Err(anyhow::anyhow!("KV delete failed: {}", response.status()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cloudflare_service_disabled() {
        let service = CloudflareService {
            enabled: false,
            api_token: "test".to_string(),
            account_id: "test".to_string(),
            r2_bucket: "test".to_string(),
            client: Client::new(),
        };

        assert!(!service.enabled);
    }
}
