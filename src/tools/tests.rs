#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::config::LLMConfig;
    
    #[test]
    fn test_config_backward_compatibility() {
        // Test new variables
        std::env::set_var("LUX_MODEL_REASONING", "test-reasoning");
        std::env::set_var("LUX_MODEL_NORMAL", "test-normal");
        std::env::set_var("LUX_MODEL_MINI", "test-mini");
        
        let config = LLMConfig::from_env().unwrap();
        assert_eq!(config.model_reasoning, "test-reasoning");
        assert_eq!(config.model_normal, "test-normal");
        assert_eq!(config.model_mini, "test-mini");
        
        // Clear new variables
        std::env::remove_var("LUX_MODEL_REASONING");
        std::env::remove_var("LUX_MODEL_NORMAL");
        std::env::remove_var("LUX_MODEL_MINI");
        
        // Test old variables (backward compatibility)
        std::env::set_var("LUX_DEFAULT_REASONING_MODEL", "old-reasoning");
        std::env::set_var("LUX_DEFAULT_CHAT_MODEL", "old-chat");
        std::env::set_var("LUX_DEFAULT_BIAS_CHECKER_MODEL", "old-bias");
        
        let config = LLMConfig::from_env().unwrap();
        assert_eq!(config.model_reasoning, "old-reasoning");
        assert_eq!(config.model_normal, "old-chat");
        assert_eq!(config.model_mini, "old-bias");
        
        // Clean up
        std::env::remove_var("LUX_DEFAULT_REASONING_MODEL");
        std::env::remove_var("LUX_DEFAULT_CHAT_MODEL");
        std::env::remove_var("LUX_DEFAULT_BIAS_CHECKER_MODEL");
    }
    
    #[test]
    fn test_config_defaults_when_no_env() {
        // Clear all relevant variables
        std::env::remove_var("LUX_MODEL_REASONING");
        std::env::remove_var("LUX_MODEL_NORMAL");
        std::env::remove_var("LUX_MODEL_MINI");
        std::env::remove_var("LUX_DEFAULT_REASONING_MODEL");
        std::env::remove_var("LUX_DEFAULT_CHAT_MODEL");
        std::env::remove_var("LUX_DEFAULT_BIAS_CHECKER_MODEL");
        
        let config = LLMConfig::from_env().unwrap();
        assert_eq!(config.model_reasoning, "gpt-5");
        assert_eq!(config.model_normal, "gpt-5");
        assert_eq!(config.model_mini, "gpt-5-mini");
    }
    
    #[test]
    fn test_config_new_vars_override_old() {
        // Set both old and new variables
        std::env::set_var("LUX_MODEL_REASONING", "new-reasoning");
        std::env::set_var("LUX_DEFAULT_REASONING_MODEL", "old-reasoning");
        
        let config = LLMConfig::from_env().unwrap();
        // New variables should take precedence
        assert_eq!(config.model_reasoning, "new-reasoning");
        
        // Clean up
        std::env::remove_var("LUX_MODEL_REASONING");
        std::env::remove_var("LUX_DEFAULT_REASONING_MODEL");
    }
}