use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info};

use crate::llm::{
    client::{ChatMessage, LLMClient},
    config::LLMConfig,
    model_aliases::ModelResolver,
    openai::OpenAIClient,
    openrouter::OpenRouterClient,
    Role,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatRequest {
    pub message: String,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatResponse {
    pub content: String,
    pub model: String,
    pub usage: Option<TokenUsage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

pub struct ChatTool {
    openai_client: Option<Arc<dyn LLMClient>>,
    openrouter_clients: Vec<(String, Arc<dyn LLMClient>)>,
    model_resolver: ModelResolver,
    config: LLMConfig,
}

impl ChatTool {
    pub fn new(config: LLMConfig) -> Result<Self> {
        let model_resolver = ModelResolver::new();
        
        // Initialize OpenAI client if API key is available
        let openai_client = if let Some(api_key) = &config.openai_api_key {
            let client = OpenAIClient::new(
                api_key.clone(),
                config.default_chat_model.clone(),
                config.openai_base_url.clone(),
            )?;
            Some(Arc::new(client) as Arc<dyn LLMClient>)
        } else {
            None
        };
        
        // Initialize OpenRouter clients for commonly used models
        let mut openrouter_clients = Vec::new();
        if let Some(api_key) = &config.openrouter_api_key {
            // Pre-create clients for common OpenRouter models
            let common_models = vec![
                "meta-llama/llama-3-70b-instruct",
                "mistralai/mixtral-8x7b-instruct",
                "anthropic/claude-3-opus",
                "anthropic/claude-3-sonnet",
                "google/gemini-2.5-pro",
                "google/gemini-2.5-flash",
                "google/gemini-2.0-flash-exp:free",
            ];
            
            for model in common_models {
                let client = OpenRouterClient::new(
                    api_key.clone(),
                    model.to_string(),
                    config.openrouter_base_url.clone(),
                )?;
                openrouter_clients.push((
                    model.to_string(),
                    Arc::new(client) as Arc<dyn LLMClient>
                ));
            }
        }
        
        Ok(Self {
            openai_client,
            openrouter_clients,
            model_resolver,
            config,
        })
    }
    
    pub async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        // Resolve model alias
        let model = request.model
            .as_ref()
            .map(|m| self.model_resolver.resolve(m))
            .unwrap_or_else(|| self.config.default_chat_model.clone());
        
        debug!("Resolved model '{}' for chat request", model);
        
        // Determine which client to use
        let client: Arc<dyn LLMClient> = if self.model_resolver.is_openrouter_model(&model) {
            // OpenRouter model
            if self.config.openrouter_api_key.is_none() {
                anyhow::bail!(
                    "OpenRouter API key not configured. Please set OPENROUTER_API_KEY"
                );
            }
            
            // Check if we have a pre-created client
            if let Some((_, client)) = self.openrouter_clients
                .iter()
                .find(|(m, _)| m == &model) {
                client.clone()
            } else {
                // Create a new client for this model
                let api_key = self.config.openrouter_api_key.as_ref().unwrap();
                let new_client = OpenRouterClient::new(
                    api_key.clone(),
                    model.clone(),
                    self.config.openrouter_base_url.clone(),
                )?;
                Arc::new(new_client) as Arc<dyn LLMClient>
            }
        } else {
            // OpenAI model
            if let Some(client) = &self.openai_client {
                // Create a new client with the specific model
                if let Some(api_key) = &self.config.openai_api_key {
                    let new_client = OpenAIClient::new(
                        api_key.clone(),
                        model.clone(),
                        self.config.openai_base_url.clone(),
                    )?;
                    Arc::new(new_client) as Arc<dyn LLMClient>
                } else {
                    anyhow::bail!("OpenAI API key not configured. Please set OPENAI_API_KEY");
                }
            } else {
                anyhow::bail!("OpenAI API key not configured. Please set OPENAI_API_KEY");
            }
        };
        
        // Create messages
        let messages = vec![ChatMessage {
            role: Role::User,
            content: request.message.clone(),
        }];
        
        // Make the request
        info!("Sending chat request to model '{}'", model);
        let response = client
            .complete(messages, request.temperature, request.max_tokens)
            .await
            .context("Failed to complete chat request")?;
        
        Ok(ChatResponse {
            content: response.content,
            model: response.model,
            usage: response.usage.map(|u| TokenUsage {
                prompt_tokens: u.prompt_tokens,
                completion_tokens: u.completion_tokens,
                total_tokens: u.total_tokens,
            }),
        })
    }
    
    pub fn suggest_model(&self, input: &str) -> Vec<String> {
        self.model_resolver.suggest_similar(input)
    }
}