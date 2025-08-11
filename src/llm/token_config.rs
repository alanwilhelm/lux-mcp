use tracing::info;

/// Centralized token configuration for all models
pub struct TokenConfig;

impl TokenConfig {
    /// Get the optimal token count for any model
    /// Mini models get limited tokens regardless of base model
    pub fn get_optimal_tokens(model: &str) -> u32 {
        let tokens = if model.contains("mini") || model.ends_with("-mini") {
            16000 // ALL mini models: Respect their 16K limit (including gpt-5-mini if it exists)
        } else if model == "gpt-5" || model.starts_with("gpt-5-") {
            128000 // GPT-5 full models: MAXIMUM INTELLIGENCE (128K completion tokens)
        } else if model.starts_with("o3") {
            100000 // O3: MAXIMUM REASONING DEPTH
        } else if model.starts_with("o4") {
            50000 // O4: MAXIMUM FAST REASONING
        } else if model.starts_with("gpt-4o") || model == "gpt-4o" {
            16384 // GPT-4o models: Respect their 16K token limit
        } else {
            16384 // Standard models: Safe default (was 20000, but many models don't support that)
        };

        info!(
            "🎯 Token allocation for model '{}': {} tokens",
            model, tokens
        );

        tokens
    }

    /// Get tokens for reasoning tasks
    pub fn get_reasoning_tokens(model: &str) -> u32 {
        let tokens = if model.contains("mini") || model.ends_with("-mini") {
            16000 // ALL mini models: Respect their 16K limit (including gpt-5-mini if it exists)
        } else if model == "gpt-5" || model.starts_with("gpt-5-") {
            128000 // GPT-5 full models: Maximum supported (128K completion tokens)
        } else if model.starts_with("o3") {
            100000 // O3: Maximum reasoning
        } else if model.starts_with("o4") {
            50000 // O4: Fast reasoning
        } else if model.starts_with("gpt-4o") || model == "gpt-4o" {
            16384 // GPT-4o models: Respect their 16K token limit
        } else {
            16384 // Standard models: Safe default
        };

        info!(
            "🧠 Reasoning token allocation for model '{}': {} tokens",
            model, tokens
        );

        tokens
    }

    /// Returns true if this model only supports default temperature (e.g., O4, GPT-5, GPT-5-mini)
    pub fn requires_default_temperature(model: &str) -> bool {
        // O4 models only support default temperature
        if model.starts_with("o4") {
            return true;
        }
        // GPT-5 models only support default temperature
        let m = model.to_lowercase();
        if m == "gpt-5" || m.starts_with("gpt-5-") || m.contains("gpt5") {
            return true;
        }
        false
    }
}
