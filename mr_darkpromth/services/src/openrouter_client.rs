use serde::{Deserialize, Serialize};
use reqwest::Client;
use anyhow::Result;
use log::error;

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenRouterMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct OpenRouterRequest {
    pub model: String,
    pub messages: Vec<OpenRouterMessage>,
}

#[derive(Debug, Deserialize)]
pub struct OpenRouterChoice {
    pub message: OpenRouterMessage,
}

#[derive(Debug, Deserialize)]
pub struct OpenRouterResponse {
    pub choices: Vec<OpenRouterChoice>,
}

pub struct OpenRouterClient {
    api_key: String,
    client: Client,
}

impl OpenRouterClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: Client::new(),
        }
    }

    pub async fn chat_completion(&self, prompt: &str, model: Option<&str>) -> Result<String> {
        let model = model.unwrap_or("openai/gpt-4o-mini");
        let url = "https://openrouter.ai/api/v1/chat/completions";
        
        let request = OpenRouterRequest {
            model: model.to_string(),
            messages: vec![OpenRouterMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
        };

        let response = self.client.post(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("HTTP-Referer", "https://mrdarkpromth.online")
            .header("X-Title", "MR.DarkPromth")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            error!("OpenRouter API error: {}", error_text);
            return Err(anyhow::anyhow!("OpenRouter API error: {}", error_text));
        }

        let result: OpenRouterResponse = response.json().await?;
        let content = result.choices.first()
            .ok_or_else(|| anyhow::anyhow!("No choices in OpenRouter response"))?
            .message.content.clone();

        Ok(content)
    }
}
