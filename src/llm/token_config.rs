use tracing::info;

/// Centralized token configuration for all models
pub struct TokenConfig;

impl TokenConfig {
    /// Get the optimal token count for any model
    /// GPT-5 family models ALL get 128K tokens!
    pub fn get_optimal_tokens(model: &str) -> u32 {
        let tokens = match model {
            // GPT-5 FAMILY - ALL GET 128K TOKENS!
            "gpt-5" => 128000,      // Maximum intelligence
            "gpt-5-mini" | "gpt5-mini" => 128000,  // Also 128K!
            "gpt-5-nano" | "gpt5-nano" => 128000,  // Also 128K per official docs!
            // Legacy models (will be removed)
            _ if model.starts_with("o3") => 100000,
            _ if model.starts_with("o4") => 50000,
            _ => 16384, // Safe default
        };

        info!(
            "🎯 Token allocation for model '{}': {} tokens",
            model, tokens
        );

        tokens
    }

    /// Get tokens for reasoning tasks
    pub fn get_reasoning_tokens(model: &str) -> u32 {
        let tokens = match model {
            // GPT-5 FAMILY - ALL GET 128K FOR MAXIMUM REASONING!
            "gpt-5" => 128000,      // Maximum reasoning depth
            "gpt-5-mini" | "gpt5-mini" => 128000,  // Full reasoning power
            "gpt-5-nano" | "gpt5-nano" => 128000,  // Full reasoning power
            // Legacy models (will be removed)
            _ if model.starts_with("o3") => 100000,
            _ if model.starts_with("o4") => 50000,
            _ => 16384, // Safe default
        };

        info!(
            "🧠 Reasoning token allocation for model '{}': {} tokens",
            model, tokens
        );

        tokens
    }

    /// Returns true if this model only supports default temperature (e.g., O4, GPT-5)
    pub fn requires_default_temperature(model: &str) -> bool {
        // O4 models only support default temperature
        if model.starts_with("o4") {
            return true;
        }
        // ALL GPT-5 FAMILY models use Responses API (no temperature support)
        matches!(model, 
            "gpt-5" | "gpt-5-mini" | "gpt-5-nano" | 
            "gpt5-mini" | "gpt5-nano"
        ) || model.starts_with("gpt-5-")
    }
}
