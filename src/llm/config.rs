use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    // API Keys
    pub openai_api_key: Option<String>,
    pub openrouter_api_key: Option<String>,
    
    // Default models for different roles
    pub default_chat_model: String,
    pub default_reasoning_model: String,
    pub default_bias_checker_model: String,
    
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
            openai_api_key: env::var("OPENAI_API_KEY")
                .ok()
                .filter(|s| !s.is_empty()),
            openrouter_api_key: env::var("OPENROUTER_API_KEY")
                .ok()
                .filter(|s| !s.is_empty()),
            
            // Default models
            default_chat_model: env::var("LUX_DEFAULT_CHAT_MODEL")
                .unwrap_or_else(|_| "gpt-4-turbo-preview".to_string()),
            default_reasoning_model: env::var("LUX_DEFAULT_REASONING_MODEL")
                .unwrap_or_else(|_| "o3-pro".to_string()),
            default_bias_checker_model: env::var("LUX_DEFAULT_BIAS_CHECKER_MODEL")
                .unwrap_or_else(|_| "o4-mini".to_string()),
            
            // API endpoints
            openai_base_url: env::var("OPENAI_BASE_URL").ok(),
            openrouter_base_url: env::var("OPENROUTER_BASE_URL").ok()
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
            default_chat_model: "gpt4.1".to_string(),
            default_reasoning_model: "o3-pro".to_string(), 
            default_bias_checker_model: "o4-mini".to_string(),
            openai_base_url: None,
            openrouter_base_url: Some("https://openrouter.ai/api/v1".to_string()),
            request_timeout_secs: 30,
            max_retries: 3,
        }
    }
}