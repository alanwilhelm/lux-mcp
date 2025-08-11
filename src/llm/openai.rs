use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, error, info, warn};

// Token limits for different model families - OPTIMIZED FOR QUALITY
const GPT5_DEFAULT_TOKENS: u32 = 128000; // GPT-5: Maximum supported completion tokens
const GPT5_MAX_TOKENS: u32 = 128000; // GPT-5 supports up to 128K completion tokens
const O3_DEFAULT_TOKENS: u32 = 100000; // O3: Maximum reasoning (200K - 100K input)
const STANDARD_DEFAULT_TOKENS: u32 = 20000; // Even standard models get more thinking space

use super::client::{ChatMessage, LLMClient, LLMResponse, Role, TokenUsage};

// Chat Completions API structures
#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_completion_tokens: Option<u32>, // For o4 models
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning_effort: Option<String>, // For o4 models
}

// Responses API structures (for o3 models)
#[derive(Debug, Serialize)]
struct ResponsesRequest {
    model: String,
    input: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning: Option<ReasoningConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<TextConfig>, // For GPT-5 verbosity control
}

#[derive(Debug, Serialize)]
struct ReasoningConfig {
    effort: String, // minimal, low, medium, high
}

#[derive(Debug, Serialize)]
struct TextConfig {
    verbosity: String, // low, medium, high
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

// Chat Completions response
#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    id: String,
    model: String,
    choices: Vec<ChatChoice>,
    usage: Option<OpenAIUsage>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: OpenAIMessage,
    finish_reason: Option<String>,
}

// Responses API response (for o3 models)
#[derive(Debug, Deserialize)]
struct ResponsesResponse {
    id: String,
    model: String,
    output: Vec<ResponseOutput>,
    usage: Option<ResponseUsage>,
}

#[derive(Debug, Deserialize)]
struct ResponseOutput {
    #[serde(rename = "type")]
    output_type: String,
    content: Option<Vec<ResponseContent>>,
}

#[derive(Debug, Deserialize)]
struct ResponseContent {
    #[serde(rename = "type")]
    content_type: String,
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ResponseUsage {
    input_tokens: u32,
    output_tokens: u32,
    total_tokens: u32,
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
            .timeout(Duration::from_secs(300)) // 5 minute timeout for o3 models
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

    fn is_o3_model(model: &str) -> bool {
        // O3 models use the Responses API
        model.starts_with("o3")
    }

    fn is_o4_model(model: &str) -> bool {
        // O4 models use chat completions with max_completion_tokens
        model.starts_with("o4")
    }

    fn is_gpt5_model(model: &str) -> bool {
        // GPT-5 models - available now via Responses API!
        model == "gpt-5" || model.starts_with("gpt-5-") || model == "gpt5-mini"
    }

    fn requires_default_temperature(model: &str) -> bool {
        // These models only support default temperature (1.0)
        Self::is_o4_model(model) 
            || Self::is_gpt5_model(model)  // All GPT-5 models use Responses API (no temperature)
    }

    pub fn get_optimal_tokens(model: &str) -> u32 {
        if Self::is_gpt5_model(model) {
            GPT5_DEFAULT_TOKENS
        } else if model.starts_with("o3") {
            O3_DEFAULT_TOKENS
        } else {
            STANDARD_DEFAULT_TOKENS
        }
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

    async fn make_chat_request(
        &self,
        messages: Vec<ChatMessage>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<LLMResponse> {
        let url = format!("{}/chat/completions", self.base_url);

        // Some models don't support custom temperature
        let adjusted_temperature = if Self::requires_default_temperature(&self.model) {
            if temperature.is_some() && temperature != Some(1.0) {
                info!(
                    "Model '{}' doesn't support custom temperature. Using default (1.0) instead of {:?}",
                    self.model, temperature
                );
            }
            None // These models only support default temperature
        } else {
            temperature
        };

        // O4 models require max_completion_tokens (GPT-5 uses Responses API, not here)
        let request = if Self::is_o4_model(&self.model) {
            ChatCompletionRequest {
                model: self.model.clone(),
                messages: Self::convert_messages(&messages),
                temperature: adjusted_temperature,
                max_tokens: None,
                max_completion_tokens: max_tokens,
                reasoning_effort: Some("high".to_string()), // O4 uses reasoning_effort
            }
        } else {
            ChatCompletionRequest {
                model: self.model.clone(),
                messages: Self::convert_messages(&messages),
                temperature: adjusted_temperature,
                max_tokens,
                max_completion_tokens: None,
                reasoning_effort: None,
            }
        };

        info!("OpenAI chat request - Model: {}, Messages: {}, Temperature: {:?}, Max tokens: {:?}, Reasoning effort: {:?}", 
            request.model, request.messages.len(), request.temperature,
            if Self::is_o4_model(&self.model) { request.max_completion_tokens } else { request.max_tokens },
            request.reasoning_effort);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send request to OpenAI")?;

        let status = response.status();
        info!("OpenAI response status: {}", status);

        if status.is_success() {
            let response_text = response.text().await?;
            let parsed: ChatCompletionResponse =
                serde_json::from_str(&response_text).context("Failed to parse OpenAI response")?;

            let choice = parsed
                .choices
                .first()
                .context("No choices in OpenAI response")?;

            let usage = parsed.usage.map(|u| TokenUsage {
                prompt_tokens: u.prompt_tokens,
                completion_tokens: u.completion_tokens,
                total_tokens: u.total_tokens,
            });

            Ok(LLMResponse {
                content: choice.message.content.clone(),
                model: parsed.model,
                usage,
                finish_reason: choice.finish_reason.clone(),
            })
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!(
                "OpenAI API error - Status: {}, Response: {}",
                status, error_text
            );

            if let Ok(error) = serde_json::from_str::<OpenAIError>(&error_text) {
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

    async fn make_responses_request(
        &self,
        messages: Vec<ChatMessage>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<LLMResponse> {
        let url = format!("{}/responses", self.base_url);

        // Convert messages to a single input string
        let input = messages
            .iter()
            .map(|msg| match msg.role {
                Role::System => format!("System: {}", msg.content),
                Role::User => format!("User: {}", msg.content),
                Role::Assistant => format!("Assistant: {}", msg.content),
            })
            .collect::<Vec<_>>()
            .join("\n");

        let request = if Self::is_gpt5_model(&self.model) {
            // GPT-5 configuration with maximum reasoning and verbosity
            ResponsesRequest {
                model: self.model.clone(),
                input,
                temperature: None, // GPT-5 doesn't support temperature (like O3)
                max_output_tokens: max_tokens,
                reasoning: Some(ReasoningConfig {
                    effort: "high".to_string(), // Maximum reasoning for GPT-5
                }),
                text: Some(TextConfig {
                    verbosity: "high".to_string(), // High verbosity for detailed responses
                }),
            }
        } else {
            // O3 configuration
            ResponsesRequest {
                model: self.model.clone(),
                input,
                temperature: None, // O3 models don't support temperature
                max_output_tokens: max_tokens,
                reasoning: Some(ReasoningConfig {
                    effort: "high".to_string(), // Maximum reasoning for O3
                }),
                text: None, // O3 doesn't support verbosity parameter
            }
        };

        info!("OpenAI Responses API request - Model: {}, Input length: {}, Temperature: {:?}, Max output tokens: {:?}, Reasoning effort: {:?}", 
            request.model, request.input.len(), request.temperature, request.max_output_tokens,
            request.reasoning.as_ref().map(|r| &r.effort));

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send request to OpenAI Responses API")?;

        let status = response.status();
        info!("OpenAI response status: {}", status);

        if status.is_success() {
            let response_text = response.text().await?;
            debug!("OpenAI Responses API raw response: {}", response_text);
            let parsed: ResponsesResponse = serde_json::from_str(&response_text)
                .context("Failed to parse OpenAI Responses API response")?;

            // Extract the message content from the output
            let mut content = String::new();
            debug!("Parsing {} outputs", parsed.output.len());
            for output in &parsed.output {
                debug!("Output type: {}", output.output_type);

                // Note: O3 models have a "reasoning" output type but it doesn't contain
                // the actual reasoning steps - just an empty summary array.
                // The reasoning happens internally but isn't exposed via the API.

                if output.output_type == "message" {
                    if let Some(contents) = &output.content {
                        debug!("Found {} content items", contents.len());
                        for c in contents {
                            debug!("Content type: {}", c.content_type);
                            if c.content_type == "output_text" {
                                if let Some(text) = &c.text {
                                    content = text.clone();
                                    info!("Extracted text from o3 response: {} chars", text.len());
                                    break;
                                }
                            }
                        }
                    }
                }
            }

            let usage = parsed.usage.map(|u| TokenUsage {
                prompt_tokens: u.input_tokens,
                completion_tokens: u.output_tokens,
                total_tokens: u.total_tokens,
            });

            Ok(LLMResponse {
                content,
                model: parsed.model,
                usage,
                finish_reason: Some("stop".to_string()),
            })
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!(
                "OpenAI Responses API error - Status: {}, Response: {}",
                status, error_text
            );

            if let Ok(error) = serde_json::from_str::<OpenAIError>(&error_text) {
                anyhow::bail!(
                    "OpenAI Responses API error ({}): {} - {}",
                    status,
                    error.error.error_type,
                    error.error.message
                );
            } else {
                anyhow::bail!("OpenAI Responses API error ({}): {}", status, error_text);
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
        let mut last_error = None;

        for attempt in 0..self.max_retries {
            if attempt > 0 {
                let delay = Duration::from_millis(1000 * (attempt as u64 + 1));
                debug!("Retry attempt {} after {:?}", attempt + 1, delay);
                tokio::time::sleep(delay).await;
            }

            let result = if Self::is_o3_model(&self.model) || Self::is_gpt5_model(&self.model) {
                info!("Using Responses API for model: {}", self.model);
                self.make_responses_request(messages.clone(), temperature, max_tokens)
                    .await
            } else {
                info!("Using chat completions API for model: {}", self.model);
                self.make_chat_request(messages.clone(), temperature, max_tokens)
                    .await
            };

            match result {
                Ok(response) => return Ok(response),
                Err(e) => {
                    warn!("OpenAI request failed (attempt {}): {}", attempt + 1, e);
                    last_error = Some(e);

                    // Don't retry on certain errors
                    if let Some(err_str) = last_error.as_ref().map(|e| e.to_string()) {
                        if err_str.contains("invalid_api_key")
                            || err_str.contains("insufficient_quota")
                            || err_str.contains("model_not_found")
                        {
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
