use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, warn, info, error};

use super::client::{ChatMessage, LLMClient, LLMResponse, Role, TokenUsage};

#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    id: String,
    model: String,
    choices: Vec<OpenAIChoice>,
    usage: Option<OpenAIUsage>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct OpenAIError {
    error: OpenAIErrorDetail,
}

#[derive(Debug, Deserialize)]
struct OpenAIErrorDetail {
    message: String,
    #[serde(rename = "type")]
    error_type: String,
    code: Option<String>,
}

pub struct OpenAIClient {
    client: Client,
    api_key: String,
    base_url: String,
    model: String,
    max_retries: u32,
}

impl OpenAIClient {
    pub fn new(api_key: String, model: String, base_url: Option<String>) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .context("Failed to build HTTP client")?;
        
        let base_url = base_url.unwrap_or_else(|| "https://api.openai.com/v1".to_string());
        
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
    
    fn convert_messages(messages: &[ChatMessage]) -> Vec<OpenAIMessage> {
        messages
            .iter()
            .map(|msg| OpenAIMessage {
                role: Self::convert_role(&msg.role),
                content: msg.content.clone(),
            })
            .collect()
    }
    
    async fn make_request(&self, request: &OpenAIRequest) -> Result<OpenAIResponse> {
        let url = format!("{}/chat/completions", self.base_url);
        
        info!("OpenAI request - Model: {}, Messages: {}, Temperature: {:?}", 
            request.model, request.messages.len(), request.temperature);
        debug!("Request URL: {}", url);
        debug!("Request body: {}", serde_json::to_string_pretty(request).unwrap_or_default());
        
        let response = match self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(request)
            .send()
            .await {
            Ok(resp) => resp,
            Err(e) => {
                error!("Failed to send request to OpenAI: {}", e);
                return Err(anyhow::anyhow!("Failed to send request to OpenAI: {}", e));
            }
        };
        
        let status = response.status();
        info!("OpenAI response status: {}", status);
        
        if status.is_success() {
            let response_text = response.text().await?;
            debug!("OpenAI response: {}", response_text);
            
            match serde_json::from_str::<OpenAIResponse>(&response_text) {
                Ok(parsed) => {
                    info!("OpenAI request successful - Model: {}, Choices: {}", 
                        parsed.model, parsed.choices.len());
                    Ok(parsed)
                },
                Err(e) => {
                    error!("Failed to parse OpenAI response: {} - Response: {}", e, response_text);
                    Err(anyhow::anyhow!("Failed to parse OpenAI response: {}", e))
                }
            }
        } else {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            error!("OpenAI API error - Status: {}, Response: {}", status, error_text);
            
            // Try to parse as OpenAI error format
            if let Ok(error) = serde_json::from_str::<OpenAIError>(&error_text) {
                error!("OpenAI error details - Type: {}, Message: {}", 
                    error.error.error_type, error.error.message);
                anyhow::bail!(
                    "OpenAI API error ({}): {} - {}",
                    status,
                    error.error.error_type,
                    error.error.message
                );
            } else {
                anyhow::bail!("OpenAI API error ({}): {}", status, error_text);
            }
        }
    }
}

#[async_trait]
impl LLMClient for OpenAIClient {
    async fn complete(
        &self,
        messages: Vec<ChatMessage>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<LLMResponse> {
        let request = OpenAIRequest {
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
                        .context("No choices in OpenAI response")?;
                    
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
                    warn!("OpenAI request failed (attempt {}): {}", attempt + 1, e);
                    last_error = Some(e);
                    
                    // Don't retry on certain errors
                    if let Some(err_str) = last_error.as_ref().map(|e| e.to_string()) {
                        if err_str.contains("invalid_api_key") 
                            || err_str.contains("insufficient_quota") {
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