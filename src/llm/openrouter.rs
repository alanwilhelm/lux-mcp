use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, warn};

use super::client::{ChatMessage, LLMClient, LLMResponse, Role, TokenUsage};

// OpenRouter uses the same format as OpenAI
#[derive(Debug, Serialize)]
struct OpenRouterRequest {
    model: String,
    messages: Vec<OpenRouterMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenRouterMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenRouterResponse {
    id: String,
    model: String,
    choices: Vec<OpenRouterChoice>,
    usage: Option<OpenRouterUsage>,
}

#[derive(Debug, Deserialize)]
struct OpenRouterChoice {
    message: OpenRouterMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenRouterUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct OpenRouterError {
    error: OpenRouterErrorDetail,
}

#[derive(Debug, Deserialize)]
struct OpenRouterErrorDetail {
    message: String,
    code: Option<i32>,
}

pub struct OpenRouterClient {
    client: Client,
    api_key: String,
    base_url: String,
    model: String,
    max_retries: u32,
}

impl OpenRouterClient {
    pub fn new(api_key: String, model: String, base_url: Option<String>) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(300))  // 5 minute timeout to match OpenAI client
            .build()
            .context("Failed to build HTTP client")?;
        
        let base_url = base_url.unwrap_or_else(|| "https://openrouter.ai/api/v1".to_string());
        
        Ok(Self {
            client,
            api_key,
            base_url,
            model,
            max_retries: 3,
        })
    }
    
    fn convert_role(role: &Role) -> String {
        match role {
            Role::System => "system".to_string(),
            Role::User => "user".to_string(),
            Role::Assistant => "assistant".to_string(),
        }
    }
    
    fn convert_messages(messages: &[ChatMessage]) -> Vec<OpenRouterMessage> {
        messages
            .iter()
            .map(|msg| OpenRouterMessage {
                role: Self::convert_role(&msg.role),
                content: msg.content.clone(),
            })
            .collect()
    }
    
    async fn make_request(&self, request: &OpenRouterRequest) -> Result<OpenRouterResponse> {
        let url = format!("{}/chat/completions", self.base_url);
        
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .header("HTTP-Referer", "https://github.com/lux-mcp") // OpenRouter requires this
            .header("X-Title", "Lux MCP") // Optional but recommended
            .json(request)
            .send()
            .await
            .context("Failed to send request to OpenRouter")?;
        
        let status = response.status();
        
        if status.is_success() {
            response
                .json::<OpenRouterResponse>()
                .await
                .context("Failed to parse OpenRouter response")
        } else {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            
            // Try to parse as OpenRouter error format
            if let Ok(error) = serde_json::from_str::<OpenRouterError>(&error_text) {
                anyhow::bail!(
                    "OpenRouter API error ({}): {}",
                    status,
                    error.error.message
                );
            } else {
                anyhow::bail!("OpenRouter API error ({}): {}", status, error_text);
            }
        }
    }
}

#[async_trait]
impl LLMClient for OpenRouterClient {
    async fn complete(
        &self,
        messages: Vec<ChatMessage>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<LLMResponse> {
        let request = OpenRouterRequest {
            model: self.model.clone(),
            messages: Self::convert_messages(&messages),
            temperature,
            max_tokens,
        };
        
        let mut last_error = None;
        
        for attempt in 0..self.max_retries {
            if attempt > 0 {
                let delay = Duration::from_millis(1000 * (attempt as u64 + 1));
                debug!("Retry attempt {} after {:?}", attempt + 1, delay);
                tokio::time::sleep(delay).await;
            }
            
            match self.make_request(&request).await {
                Ok(response) => {
                    let choice = response
                        .choices
                        .first()
                        .context("No choices in OpenRouter response")?;
                    
                    let usage = response.usage.map(|u| TokenUsage {
                        prompt_tokens: u.prompt_tokens,
                        completion_tokens: u.completion_tokens,
                        total_tokens: u.total_tokens,
                    });
                    
                    return Ok(LLMResponse {
                        content: choice.message.content.clone(),
                        model: response.model,
                        usage,
                        finish_reason: choice.finish_reason.clone(),
                    });
                }
                Err(e) => {
                    warn!("OpenRouter request failed (attempt {}): {}", attempt + 1, e);
                    last_error = Some(e);
                    
                    // Don't retry on certain errors
                    if let Some(err_str) = last_error.as_ref().map(|e| e.to_string()) {
                        if err_str.contains("invalid_api_key") 
                            || err_str.contains("insufficient_quota")
                            || err_str.contains("credits") {
                            break;
                        }
                    }
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("All retry attempts failed")))
    }
    
    fn get_model_name(&self) -> &str {
        &self.model
    }
}