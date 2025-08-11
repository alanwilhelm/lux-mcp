use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::env;
use std::fmt;

#[derive(Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    // API Keys - never serialize these for security
    #[serde(skip_serializing)]
    pub openai_api_key: Option<String>,
    #[serde(skip_serializing)]
    pub openrouter_api_key: Option<String>,

    // Simplified model configuration
    pub model_reasoning: String, // Main reasoning model (e.g., gpt-5)
    pub model_normal: String,    // Main normal model (e.g., gpt-5)
    pub model_mini: String,      // Mini model (e.g., gpt-5-mini)

    // Named model definitions (for OpenRouter models)
    pub model_opus: Option<String>,   // Claude 3 Opus mapping
    pub model_sonnet: Option<String>, // Claude Sonnet mapping
    pub model_grok: Option<String>,   // Grok mapping

    // API endpoints (optional overrides)
    pub openai_base_url: Option<String>,
    pub openrouter_base_url: Option<String>,

    // Request settings
    pub request_timeout_secs: u64,
    pub max_retries: u32,
}

impl LLMConfig {
    pub fn from_env() -> Result<Self> {
        // Load .env file if it exists
        dotenv::dotenv().ok();

        Ok(Self {
            // API Keys (filter out empty strings)
            openai_api_key: env::var("OPENAI_API_KEY").ok().filter(|s| !s.is_empty()),
            openrouter_api_key: env::var("OPENROUTER_API_KEY")
                .ok()
                .filter(|s| !s.is_empty()),

            // Model configuration with backward compatibility
            model_reasoning: env::var("LUX_MODEL_REASONING")
                .or_else(|_| env::var("LUX_DEFAULT_REASONING_MODEL")) // Backward compat
                .unwrap_or_else(|_| "gpt-5".to_string()),
            model_normal: env::var("LUX_MODEL_NORMAL")
                .or_else(|_| env::var("LUX_DEFAULT_CHAT_MODEL")) // Backward compat
                .unwrap_or_else(|_| "gpt-5".to_string()),
            model_mini: env::var("LUX_MODEL_MINI")
                .or_else(|_| env::var("LUX_DEFAULT_BIAS_CHECKER_MODEL")) // Backward compat
                .unwrap_or_else(|_| "gpt-5-mini".to_string()),

            // Named model definitions with defaults
            model_opus: env::var("LUX_MODEL_OPUS")
                .ok()
                .or_else(|| Some("anthropic/claude-4.1-opus".to_string())),
            model_sonnet: env::var("LUX_MODEL_SONNET")
                .ok()
                .or_else(|| Some("anthropic/claude-4-sonnet".to_string())),
            model_grok: env::var("LUX_MODEL_GROK")
                .ok()
                .or_else(|| Some("x-ai/grok-beta".to_string())),

            // API endpoints
            openai_base_url: env::var("OPENAI_BASE_URL").ok(),
            openrouter_base_url: env::var("OPENROUTER_BASE_URL")
                .ok()
                .or_else(|| Some("https://openrouter.ai/api/v1".to_string())),

            // Request settings
            request_timeout_secs: env::var("LUX_REQUEST_TIMEOUT_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(30),
            max_retries: env::var("LUX_MAX_RETRIES")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(3),
        })
    }

    pub fn validate(&self) -> Result<()> {
        if self.openai_api_key.is_none() && self.openrouter_api_key.is_none() {
            anyhow::bail!(
                "No API keys configured. Please set OPENAI_API_KEY or OPENROUTER_API_KEY"
            );
        }
        Ok(())
    }
}

impl Default for LLMConfig {
    fn default() -> Self {
        Self {
            openai_api_key: None,
            openrouter_api_key: None,
            model_reasoning: "gpt-5".to_string(),
            model_normal: "gpt-5".to_string(),
            model_mini: "gpt-5-mini".to_string(),
            model_opus: Some("anthropic/claude-4.1-opus".to_string()),
            model_sonnet: Some("anthropic/claude-4-sonnet".to_string()),
            model_grok: Some("x-ai/grok-beta".to_string()),
            openai_base_url: None,
            openrouter_base_url: Some("https://openrouter.ai/api/v1".to_string()),
            request_timeout_secs: 30,
            max_retries: 3,
        }
    }
}

// Custom Debug implementation that redacts API keys for security
impl fmt::Debug for LLMConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LLMConfig")
            .field("openai_api_key", &self.openai_api_key.as_ref().map(|_| "[REDACTED]"))
            .field("openrouter_api_key", &self.openrouter_api_key.as_ref().map(|_| "[REDACTED]"))
            .field("model_reasoning", &self.model_reasoning)
            .field("model_normal", &self.model_normal)
            .field("model_mini", &self.model_mini)
            .field("model_opus", &self.model_opus)
            .field("model_sonnet", &self.model_sonnet)
            .field("model_grok", &self.model_grok)
            .field("openai_base_url", &self.openai_base_url)
            .field("openrouter_base_url", &self.openrouter_base_url)
            .field("request_timeout_secs", &self.request_timeout_secs)
            .field("max_retries", &self.max_retries)
            .finish()
    }
}
