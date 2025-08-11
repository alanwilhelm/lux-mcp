use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
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
    /// File paths to include in context for the LLM
    #[serde(default)]
    pub file_paths: Option<Vec<String>>,
    /// Whether to include file contents (default: true)
    #[serde(default = "default_true")]
    pub include_file_contents: bool,
    /// Use mini model for cost savings (overrides model selection)
    #[serde(default)]
    pub use_mini: bool,
}

fn default_true() -> bool {
    true
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
    /// Read files and return their contents
    fn read_files(&self, file_paths: &[String]) -> Vec<(String, String)> {
        let mut file_contents = Vec::new();

        for path in file_paths {
            let file_path = Path::new(path);
            if file_path.exists() && file_path.is_file() {
                match fs::read_to_string(file_path) {
                    Ok(content) => {
                        info!("Read file for chat context: {}", path);
                        // Truncate very large files to avoid token limits
                        let truncated = if content.len() > 10000 {
                            format!("{}... [truncated]", &content[..10000])
                        } else {
                            content
                        };
                        file_contents.push((path.clone(), truncated));
                    }
                    Err(e) => {
                        warn!("Failed to read file {}: {}", path, e);
                    }
                }
            } else {
                info!("File not found or not a file: {}", path);
            }
        }

        file_contents
    }

    pub fn new(config: LLMConfig) -> Result<Self> {
        let model_resolver = ModelResolver::with_config(Some(config.clone()));

        // Initialize OpenAI client if API key is available
        let openai_client = if let Some(api_key) = &config.openai_api_key {
            let client = OpenAIClient::new(
                api_key.clone(),
                config.model_normal.clone(),
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
            "Chat request received - Message: {}, Model: {:?}, Temperature: {:?}, File paths: {:?}",
            request.message, request.model, request.temperature, request.file_paths
        );

        // Resolve model alias - use mini model if requested for cost savings
        let requested_model = if request.use_mini {
            self.config.model_mini.clone()
        } else {
            request
                .model
                .as_ref()
                .map(|m| self.model_resolver.resolve(m))
                .unwrap_or_else(|| self.config.model_normal.clone())
        };

        info!(
            "Resolved model: {} (requested: {:?}, use_mini: {}, default: {})",
            requested_model, request.model, request.use_mini, self.config.model_normal
        );

        // If requested model is blocked (e.g., gpt-4o family), switch to default immediately
        let requested_model = if self.model_resolver.is_blocked_model(&requested_model) {
            warn!(
                "Requested model '{}' is blocked by policy. Using default '{}' instead.",
                requested_model, self.config.model_normal
            );
            self.config.model_normal.clone()
        } else {
            requested_model
        };

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
        // Strict policy: Only GPT-5 and GPT-5-mini are allowed
        let mut fallbacks = Vec::new();
        if requested_model != "gpt-5" {
            fallbacks.push("gpt-5".to_string());
        }
        if requested_model != "gpt-5-mini" {
            fallbacks.push("gpt-5-mini".to_string());
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

        // STRICT MODEL POLICY ENFORCEMENT
        // ONLY gpt-5 and gpt-5-mini are supported
        let resolved_model = self.model_resolver.resolve(&model).to_lowercase();
        
        // Log the policy check for debugging
        info!("🔒 MODEL POLICY CHECK: Requested '{}' -> Resolved '{}'", model, resolved_model);
        
        // Check if this is a supported model (ONLY 3 ALLOWED)
        let is_supported = matches!(resolved_model.as_str(), 
            "gpt-5" | "gpt-5-mini" | "gpt-5-nano" | "gpt5-mini" | "gpt5-nano"
        );
        
        // If not supported, use fallback with warning
        let (actual_model, actual_request) = if !is_supported {
            warn!("⚠️ Model '{}' is not supported. Using gpt-5 instead.", model);
            
            // Add a note to the response about the model substitution
            let mut modified_request = request.clone();
            modified_request.message = format!(
                "[System Note: Model '{}' is not supported. Using gpt-5 instead.]\n\n{}",
                model, request.message
            );
            ("gpt-5".to_string(), modified_request)
        } else {
            (resolved_model.clone(), request.clone())
        };

        // Always use OpenAI since we only support gpt-5 and gpt-5-mini
        info!("Using OpenAI for model: {}", actual_model);
        
        let client: Arc<dyn LLMClient> = if let Some(api_key) = &self.config.openai_api_key {
            debug!("Creating OpenAI client for model: {}", actual_model);
            let new_client = OpenAIClient::new(
                api_key.clone(),
                actual_model.clone(),
                self.config.openai_base_url.clone(),
            )?;
            Arc::new(new_client) as Arc<dyn LLMClient>
        } else {
            error!("OpenAI API key not configured");
            anyhow::bail!("OpenAI API key not configured. Please set OPENAI_API_KEY");
        };

        // Build message with optional file contents
        let mut full_message = String::new();

        // Add file contents if provided
        if let Some(ref file_paths) = actual_request.file_paths {
            info!("File paths provided: {:?}", file_paths);
            if actual_request.include_file_contents && !file_paths.is_empty() {
                info!("Attempting to read {} files", file_paths.len());
                let file_contents = self.read_files(file_paths);
                info!("Successfully read {} files", file_contents.len());
                if !file_contents.is_empty() {
                    full_message.push_str("=== FILE CONTEXT ===\n");
                    for (path, content) in file_contents {
                        full_message.push_str(&format!("\n📄 File: {}\n", path));
                        full_message.push_str(&format!("```\n{}\n```\n", content));
                    }
                    full_message.push_str("\n=== END FILE CONTEXT ===\n\n");
                    info!("Added {} files to chat context", file_paths.len());
                }
            }
        }

        // Add the actual message
        full_message.push_str(&actual_request.message);

        // Create messages
        let messages = vec![ChatMessage {
            role: Role::User,
            content: full_message,
        }];

        // ALWAYS USE OPTIMAL INTELLIGENCE - MAXIMUM TOKENS FOR DEEPEST REASONING
        // No user override - always use the maximum for each model
        // Check mini FIRST to handle gpt-5-mini correctly
        let max_tokens = if model.contains("mini") || model.ends_with("-mini") {
            16000 // ALL mini models: Respect their 16K limit (including gpt-5-mini)
        } else if model == "gpt-5" || model.starts_with("gpt-5-") {
            128000 // GPT-5 full models: MAXIMUM INTELLIGENCE (128K completion tokens)
        } else if model.starts_with("o3") {
            100000 // O3: MAXIMUM REASONING DEPTH
        } else if model.starts_with("o4") {
            50000 // O4: MAXIMUM FAST REASONING
        } else {
            20000 // Standard models: MAXIMUM THINKING ROOM
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
