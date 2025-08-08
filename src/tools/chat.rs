use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use crate::llm::{
    client::{ChatMessage, LLMClient},
    config::LLMConfig,
    model_aliases::ModelResolver,
    openai::OpenAIClient,
    openrouter::OpenRouterClient,
    Role,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub message: String,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub temperature: Option<f32>,
    // REMOVED max_tokens - always use optimal intelligence
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub continuation_id: Option<String>,
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
                openrouter_clients
                    .push((model.to_string(), Arc::new(client) as Arc<dyn LLMClient>));
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
        info!(
            "Chat request received - Message: {}, Model: {:?}, Temperature: {:?}",
            request.message, request.model, request.temperature
        );

        // Resolve model alias
        let requested_model = request
            .model
            .as_ref()
            .map(|m| self.model_resolver.resolve(m))
            .unwrap_or_else(|| self.config.default_chat_model.clone());

        info!(
            "Resolved model: {} (requested: {:?}, default: {})",
            requested_model, request.model, self.config.default_chat_model
        );

        // Define fallback models in order of preference
        let fallback_models = self.get_fallback_models(&requested_model);

        // Try requested model and fallbacks
        let mut last_error = None;
        for (attempt, model) in std::iter::once(requested_model.clone())
            .chain(fallback_models.iter().cloned())
            .enumerate()
        {
            if attempt > 0 {
                info!(
                    "🔄 Attempting fallback model: {} (attempt {})",
                    model,
                    attempt + 1
                );
            }

            match self
                .try_chat_with_model(request.clone(), model.clone())
                .await
            {
                Ok(response) => {
                    if attempt > 0 {
                        info!("✅ Successfully used fallback model: {}", model);
                    }
                    return Ok(response);
                }
                Err(e) => {
                    let error_str = e.to_string();
                    if error_str.contains("does not exist")
                        || error_str.contains("do not have access")
                        || error_str.contains("model_not_found")
                    {
                        warn!("❌ Model '{}' not available: {}", model, error_str);
                        last_error = Some(e);
                        continue;
                    } else {
                        // Non-recoverable error, return immediately
                        return Err(e);
                    }
                }
            }
        }

        // All models failed
        Err(last_error.unwrap_or_else(|| {
            anyhow::anyhow!(
                "All models failed. Requested: {}, tried fallbacks: {:?}",
                requested_model,
                fallback_models
            )
        }))
    }

    fn get_fallback_models(&self, requested_model: &str) -> Vec<String> {
        let mut fallbacks = Vec::new();

        // If a Claude model was requested, try other Claude variants
        if requested_model.contains("claude") {
            fallbacks.push("anthropic/claude-3.5-sonnet".to_string());
            fallbacks.push("anthropic/claude-3-opus".to_string());
            fallbacks.push("anthropic/claude-3-sonnet".to_string());
        }

        // Add default model as fallback if not already requested
        if requested_model != self.config.default_chat_model {
            fallbacks.push(self.config.default_chat_model.clone());
        }

        // Add some reliable fallbacks
        if !requested_model.contains("gpt-4") {
            fallbacks.push("gpt-4o-mini".to_string());
        }
        if !requested_model.contains("gemini") {
            fallbacks.push("google/gemini-2.5-flash".to_string());
        }

        // Remove duplicates while preserving order
        let mut seen = std::collections::HashSet::new();
        fallbacks.retain(|model| seen.insert(model.clone()));

        fallbacks
    }

    async fn try_chat_with_model(
        &self,
        request: ChatRequest,
        model: String,
    ) -> Result<ChatResponse> {
        debug!("Trying chat with model: {}", model);

        // Check API keys
        debug!(
            "OpenAI key available: {}",
            self.config.openai_api_key.is_some()
        );
        debug!(
            "OpenRouter key available: {}",
            self.config.openrouter_api_key.is_some()
        );

        // Determine which client to use
        let client: Arc<dyn LLMClient> = if self.model_resolver.is_openrouter_model(&model) {
            // OpenRouter model
            info!("Using OpenRouter for model: {}", model);
            if self.config.openrouter_api_key.is_none() {
                error!("OpenRouter API key not configured");
                anyhow::bail!("OpenRouter API key not configured. Please set OPENROUTER_API_KEY");
            }

            // Check if we have a pre-created client
            if let Some((_, client)) = self.openrouter_clients.iter().find(|(m, _)| m == &model) {
                client.clone()
            } else {
                // Create a new client for this model
                let api_key = self
                    .config
                    .openrouter_api_key
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("OpenRouter API key not available"))?;
                let new_client = OpenRouterClient::new(
                    api_key.clone(),
                    model.clone(),
                    self.config.openrouter_base_url.clone(),
                )?;
                Arc::new(new_client) as Arc<dyn LLMClient>
            }
        } else {
            // OpenAI model
            info!("Using OpenAI for model: {}", model);
            if let Some(_client) = &self.openai_client {
                // Create a new client with the specific model
                if let Some(api_key) = &self.config.openai_api_key {
                    debug!("Creating OpenAI client for model: {}", model);
                    let new_client = OpenAIClient::new(
                        api_key.clone(),
                        model.clone(),
                        self.config.openai_base_url.clone(),
                    )?;
                    Arc::new(new_client) as Arc<dyn LLMClient>
                } else {
                    error!("OpenAI API key not configured");
                    anyhow::bail!("OpenAI API key not configured. Please set OPENAI_API_KEY");
                }
            } else {
                error!("OpenAI client not initialized - API key missing");
                anyhow::bail!("OpenAI API key not configured. Please set OPENAI_API_KEY");
            }
        };

        // Create messages
        let messages = vec![ChatMessage {
            role: Role::User,
            content: request.message.clone(),
        }];

        // ALWAYS USE OPTIMAL INTELLIGENCE - MAXIMUM TOKENS FOR DEEPEST REASONING
        // No user override - always use the maximum for each model
        let max_tokens = if model == "gpt-5" || model.starts_with("gpt-5-") {
            128000 // GPT-5: MAXIMUM INTELLIGENCE (128K completion tokens)
        } else if model.starts_with("o3") {
            100000 // O3: MAXIMUM REASONING DEPTH
        } else if model.starts_with("o4") {
            50000  // O4: MAXIMUM FAST REASONING
        } else {
            20000  // Standard models: MAXIMUM THINKING ROOM
        };

        info!(
            "🚀 Sending chat request to model '{}' with max_tokens: {}",
            model, max_tokens
        );
        if model.starts_with("o3") {
            info!(
                "⏳ Using {} for deep reasoning. This may take 30 seconds to 5 minutes...",
                model
            );
            info!("💭 The model is thinking deeply about your question...");
        }

        let start_time = std::time::Instant::now();
        let response = client
            .complete(messages, request.temperature, Some(max_tokens))
            .await
            .map_err(|e| {
                let elapsed = start_time.elapsed();
                error!("Chat request failed after {:?}: {}", elapsed, e);
                anyhow::anyhow!("Failed to complete chat request after {:?}: {}", elapsed, e)
            })?;

        let elapsed = start_time.elapsed();
        info!("✅ {} responded in {:?}", model, elapsed);

        // Format the response with rich styling
        let formatted_content = self.format_chat_response(
            &response.content,
            &response.model,
            &response.usage,
            elapsed,
            request.temperature,
            max_tokens,
        );

        Ok(ChatResponse {
            content: formatted_content,
            model: response.model,
            usage: response.usage.map(|u| TokenUsage {
                prompt_tokens: u.prompt_tokens,
                completion_tokens: u.completion_tokens,
                total_tokens: u.total_tokens,
            }),
        })
    }

    fn format_chat_response(
        &self,
        content: &str,
        model: &str,
        usage: &Option<crate::llm::client::TokenUsage>,
        elapsed: std::time::Duration,
        temperature: Option<f32>,
        max_tokens: u32,
    ) -> String {
        let mut output = String::new();

        // Header
        output.push_str("\n🔍 **LUX ANALYSIS COMPLETE** 🔍\n\n");

        // Response metadata
        output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
        output.push_str(&format!("🤖 **Model:** {} ", model));

        // Add model type indicator
        let model_type = if model.contains("o3") || model.contains("o4") {
            "⚡ (Deep Reasoning Engine)"
        } else if model.contains("claude") {
            "🎨 (Creative Engine)"
        } else if model.contains("gemini") {
            "🌟 (Multimodal Engine)"
        } else if model.contains("gpt") {
            "💡 (General Engine)"
        } else {
            "🔧 (Custom Model)"
        };
        output.push_str(&format!("{}\n", model_type));

        // Performance metrics
        output.push_str(&format!(
            "⏱️  **Response Time:** {:.2}s ",
            elapsed.as_secs_f64()
        ));
        let speed_indicator = match elapsed.as_secs() {
            0..=2 => "⚡ (Lightning Fast)",
            3..=10 => "🚀 (Fast)",
            11..=30 => "✈️ (Normal)",
            31..=60 => "🚂 (Deep Thinking)",
            _ => "🐌 (Extended Analysis)",
        };
        output.push_str(&format!("{}\n", speed_indicator));

        // Configuration
        output.push_str(&format!(
            "🎛️  **Temperature:** {} ",
            temperature.unwrap_or(1.0)
        ));
        let creativity = match temperature.unwrap_or(1.0) {
            t if t <= 0.3 => "❄️ (Focused)",
            t if t <= 0.7 => "🌡️ (Balanced)",
            t if t <= 1.0 => "🔥 (Creative)",
            _ => "🌋 (Experimental)",
        };
        output.push_str(&format!("{}\n", creativity));

        output.push_str(&format!("📏 **Max Tokens:** {}\n", max_tokens));

        // Token usage if available
        if let Some(usage) = usage {
            output.push_str("\n📊 **Token Usage:**\n");
            output.push_str(&format!("   • Prompt: {} tokens\n", usage.prompt_tokens));
            output.push_str(&format!(
                "   • Response: {} tokens\n",
                usage.completion_tokens
            ));
            output.push_str(&format!("   • Total: {} tokens", usage.total_tokens));

            // Cost indicator (rough estimate)
            let efficiency = match usage.total_tokens {
                0..=500 => " 💰 (Efficient)",
                501..=2000 => " 💵 (Standard)",
                2001..=5000 => " 💸 (Verbose)",
                _ => " 🏦 (Extensive)",
            };
            output.push_str(&format!("{}\n", efficiency));
        }

        output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n");

        // Add emphasis to the actual response
        if model.contains("o3") || model.contains("o4") {
            output.push_str("⚠️ **IMPORTANT: You MUST now SYNTHESIZE and ACT on the following analysis from the ");
            output.push_str(&format!("{} model:**\n\n", model));
            output.push_str("---\n\n");
            output.push_str(content);
            output.push_str("\n\n---\n\n");
            output.push_str("🎯 **REQUIRED ACTIONS:**\n");
            output.push_str("1. ✅ SUMMARIZE the key findings\n");
            output.push_str("2. ✅ IDENTIFY the most critical recommendations\n");
            output.push_str("3. ✅ CREATE an actionable plan based on this analysis\n");
            output.push_str("4. ✅ RESPOND with clear next steps for the user\n\n");
            output.push_str(
                "⚡ **DO NOT just acknowledge this output - YOU MUST PROCESS AND ACT ON IT!** ⚡",
            );
        } else {
            // For non-reasoning models, simpler formatting
            output.push_str(content);
        }

        output
    }

    pub fn suggest_model(&self, input: &str) -> Vec<String> {
        self.model_resolver.suggest_similar(input)
    }
}
