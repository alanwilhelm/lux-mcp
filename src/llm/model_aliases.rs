use std::collections::HashMap;

pub struct ModelResolver {
    aliases: HashMap<String, String>,
}

impl ModelResolver {
    pub fn new() -> Self {
        let mut aliases = HashMap::new();
        
        // Basic GPT-4 variants
        aliases.insert("gpt4".to_string(), "gpt-4".to_string());
        aliases.insert("gpt-4".to_string(), "gpt-4".to_string());
        aliases.insert("4".to_string(), "gpt-4".to_string());
        
        // GPT-4.1 variants
        aliases.insert("gpt4.1".to_string(), "gpt-4-turbo-preview".to_string());
        aliases.insert("gpt-4.1".to_string(), "gpt-4-turbo-preview".to_string());
        aliases.insert("4.1".to_string(), "gpt-4-turbo-preview".to_string());
        aliases.insert("gpt41".to_string(), "gpt-4-turbo-preview".to_string());
        aliases.insert("gpt-4-turbo".to_string(), "gpt-4-turbo-preview".to_string());
        
        // O3 variants - map to full model names
        aliases.insert("o3".to_string(), "o3".to_string());
        aliases.insert("o3-pro".to_string(), "o3-pro-2025-06-10".to_string());
        aliases.insert("o3pro".to_string(), "o3-pro-2025-06-10".to_string());
        
        // O4 variants - direct model names
        aliases.insert("o4-mini".to_string(), "o4-mini".to_string());
        aliases.insert("o4mini".to_string(), "o4-mini".to_string());
        
        // GPT-4o-mini variants
        aliases.insert("mini".to_string(), "gpt-4o-mini".to_string());
        aliases.insert("4omini".to_string(), "gpt-4o-mini".to_string());
        aliases.insert("gpt4o-mini".to_string(), "gpt-4o-mini".to_string());
        aliases.insert("gpt-4o-mini".to_string(), "gpt-4o-mini".to_string());
        
        // Claude variants (via OpenRouter)
        aliases.insert("claude".to_string(), "anthropic/claude-4-sonnet".to_string());
        aliases.insert("claude3".to_string(), "anthropic/claude-3-opus".to_string());
        aliases.insert("claude-3".to_string(), "anthropic/claude-3-opus".to_string());
        aliases.insert("opus".to_string(), "anthropic/claude-4-opus".to_string());
        aliases.insert("claude-opus".to_string(), "anthropic/claude-3-opus".to_string());
        aliases.insert("sonnet".to_string(), "anthropic/claude-3-sonnet".to_string());
        aliases.insert("claude-sonnet".to_string(), "anthropic/claude-3-sonnet".to_string());
        aliases.insert("haiku".to_string(), "anthropic/claude-3-haiku".to_string());
        aliases.insert("claude-haiku".to_string(), "anthropic/claude-3-haiku".to_string());
        aliases.insert("claude-instant".to_string(), "anthropic/claude-instant-1.2".to_string());
        aliases.insert("instant".to_string(), "anthropic/claude-instant-1.2".to_string());
        
        // Claude 3.5 variants
        aliases.insert("claude-3.5".to_string(), "anthropic/claude-3.5-sonnet".to_string());
        aliases.insert("claude3.5".to_string(), "anthropic/claude-3.5-sonnet".to_string());
        aliases.insert("sonnet-3.5".to_string(), "anthropic/claude-3.5-sonnet".to_string());
        aliases.insert("claude-3.5-sonnet".to_string(), "anthropic/claude-3.5-sonnet".to_string());
        
        // Claude 3.5 with date suffixes (map to base model)
        aliases.insert("claude-3-5-sonnet-20241022".to_string(), "anthropic/claude-3.5-sonnet".to_string());
        aliases.insert("claude-3.5-sonnet-20241022".to_string(), "anthropic/claude-3.5-sonnet".to_string());
        aliases.insert("claude-3-5-sonnet-latest".to_string(), "anthropic/claude-3.5-sonnet".to_string());
        aliases.insert("claude-3.5-sonnet-latest".to_string(), "anthropic/claude-3.5-sonnet".to_string());
        
        // Claude 4 variants (future models)
        aliases.insert("opus-4".to_string(), "anthropic/claude-4-opus".to_string());
        aliases.insert("opus4".to_string(), "anthropic/claude-4-opus".to_string());
        aliases.insert("4-opus".to_string(), "anthropic/claude-4-opus".to_string());
        aliases.insert("claude-4-opus".to_string(), "anthropic/claude-4-opus".to_string());
        aliases.insert("sonnet-4".to_string(), "anthropic/claude-4-sonnet".to_string());
        aliases.insert("sonnet4".to_string(), "anthropic/claude-4-sonnet".to_string());
        aliases.insert("4-sonnet".to_string(), "anthropic/claude-4-sonnet".to_string());
        aliases.insert("claude-4-sonnet".to_string(), "anthropic/claude-4-sonnet".to_string());
        
        // Google models - Many aliases, two models
        // All Gemini variants → gemini-2.5-pro
        aliases.insert("gemini".to_string(), "google/gemini-2.5-pro".to_string());
        aliases.insert("gemini-pro".to_string(), "google/gemini-2.5-pro".to_string());
        aliases.insert("geminipro".to_string(), "google/gemini-2.5-pro".to_string());
        aliases.insert("gemini-2.5".to_string(), "google/gemini-2.5-pro".to_string());
        aliases.insert("gemini2.5".to_string(), "google/gemini-2.5-pro".to_string());
        aliases.insert("gemini-1.5".to_string(), "google/gemini-2.5-pro".to_string());
        aliases.insert("gemini1.5".to_string(), "google/gemini-2.5-pro".to_string());
        aliases.insert("gemini2".to_string(), "google/gemini-2.5-pro".to_string());
        
        // All Flash variants → gemini-2.5-flash
        aliases.insert("flash".to_string(), "google/gemini-2.5-flash".to_string());
        aliases.insert("gemini-flash".to_string(), "google/gemini-2.5-flash".to_string());
        aliases.insert("geminiflash".to_string(), "google/gemini-2.5-flash".to_string());
        aliases.insert("gemini-2.5-flash".to_string(), "google/gemini-2.5-flash".to_string());
        aliases.insert("gemini2.5flash".to_string(), "google/gemini-2.5-flash".to_string());
        aliases.insert("flash-2.5".to_string(), "google/gemini-2.5-flash".to_string());
        aliases.insert("flash2.5".to_string(), "google/gemini-2.5-flash".to_string());
        aliases.insert("gflash".to_string(), "google/gemini-2.5-flash".to_string());
        
        // Common OpenRouter models
        aliases.insert("llama3".to_string(), "meta-llama/llama-3-70b-instruct".to_string());
        aliases.insert("llama-3".to_string(), "meta-llama/llama-3-70b-instruct".to_string());
        aliases.insert("mixtral".to_string(), "mistralai/mixtral-8x7b-instruct".to_string());
        aliases.insert("mistral".to_string(), "mistralai/mistral-7b-instruct".to_string());
        aliases.insert("deepseek".to_string(), "deepseek/deepseek-coder".to_string());
        
        Self { aliases }
    }
    
    pub fn resolve(&self, input: &str) -> String {
        // Convert to lowercase and remove common separators for lookup
        let normalized = input.to_lowercase()
            .replace('-', "")
            .replace('_', "")
            .replace(' ', "");
        
        // First check exact match
        if let Some(resolved) = self.aliases.get(input) {
            return resolved.clone();
        }
        
        // Then check normalized version
        if let Some(resolved) = self.aliases.get(&normalized) {
            return resolved.clone();
        }
        
        // Try stripping date suffix (e.g., -20241022 or -latest)
        let without_date = self.strip_date_suffix(input);
        if without_date != input {
            if let Some(resolved) = self.aliases.get(&without_date) {
                return resolved.clone();
            }
            // Also try normalized version without date
            let normalized_without_date = without_date.to_lowercase()
                .replace('-', "")
                .replace('_', "")
                .replace(' ', "");
            if let Some(resolved) = self.aliases.get(&normalized_without_date) {
                return resolved.clone();
            }
        }
        
        // Check if it's an OpenRouter model (contains /)
        if input.contains('/') {
            return input.to_string();
        }
        
        // Default to the input as-is
        input.to_string()
    }
    
    fn strip_date_suffix(&self, model: &str) -> String {
        // Remove date suffixes like -20241022 or -latest
        if let Some(idx) = model.rfind('-') {
            let suffix = &model[idx + 1..];
            // Check if suffix is all digits (date) or "latest"
            if suffix.chars().all(|c| c.is_numeric()) || suffix == "latest" {
                return model[..idx].to_string();
            }
        }
        model.to_string()
    }
    
    pub fn is_openrouter_model(&self, model: &str) -> bool {
        let resolved = self.resolve(model);
        resolved.contains('/')
    }
    
    pub fn suggest_similar(&self, input: &str) -> Vec<String> {
        let normalized = input.to_lowercase();
        let mut suggestions = Vec::new();
        
        for (alias, _) in &self.aliases {
            if alias.contains(&normalized) || normalized.contains(alias) {
                suggestions.push(alias.clone());
            }
        }
        
        suggestions.sort();
        suggestions.dedup();
        suggestions.truncate(5); // Return top 5 suggestions
        suggestions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_model_aliases() {
        let resolver = ModelResolver::new();
        
        // Test various GPT-4 aliases
        assert_eq!(resolver.resolve("gpt4"), "gpt-4");
        assert_eq!(resolver.resolve("4"), "gpt-4");
        assert_eq!(resolver.resolve("GPT4"), "gpt-4");
        assert_eq!(resolver.resolve("gpt-4"), "gpt-4");
        
        // Test mini variants
        assert_eq!(resolver.resolve("mini"), "gpt-4o-mini");
        assert_eq!(resolver.resolve("4omini"), "gpt-4o-mini");
        assert_eq!(resolver.resolve("gpt4o-mini"), "gpt-4o-mini");
        
        // Test Claude
        assert_eq!(resolver.resolve("claude"), "anthropic/claude-4-sonnet");
        assert_eq!(resolver.resolve("opus"), "anthropic/claude-4-opus");
        assert_eq!(resolver.resolve("sonnet"), "anthropic/claude-3-sonnet");
        assert_eq!(resolver.resolve("claude-3.5"), "anthropic/claude-3.5-sonnet");
        
        // Test OpenRouter detection
        assert!(resolver.is_openrouter_model("llama3"));
        assert!(resolver.is_openrouter_model("meta-llama/llama-3-70b"));
        assert!(!resolver.is_openrouter_model("gpt4"));
    }
}