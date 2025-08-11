use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::Path;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupConfigRequest {
    /// User's OpenAI API key (optional - will prompt if not provided)
    #[serde(default)]
    pub openai_api_key: Option<String>,

    /// User's OpenRouter API key (optional - will prompt if not provided)
    #[serde(default)]
    pub openrouter_api_key: Option<String>,

    /// Whether to use advanced models (gpt-5, o3-pro) or standard (gpt-4o)
    #[serde(default)]
    pub use_advanced_models: bool,

    /// Custom model preferences
    #[serde(default)]
    pub custom_models: Option<CustomModels>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomModels {
    pub reasoning_model: Option<String>,
    pub normal_model: Option<String>,
    pub mini_model: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetupConfigResponse {
    pub status: String,
    pub instructions_for_host_llm: Vec<String>,
    pub env_template: String,
    pub env_file_path: String,
    pub current_config: ConfigStatus,
    pub next_steps: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigStatus {
    pub env_file_exists: bool,
    pub openai_configured: bool,
    pub openrouter_configured: bool,
    pub models_configured: bool,
    pub current_models: ModelConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelConfig {
    pub reasoning: String,
    pub normal: String,
    pub mini: String,
    pub opus: String,
    pub sonnet: String,
    pub grok: String,
}

pub struct SetupConfigTool;

impl SetupConfigTool {
    pub fn new() -> Self {
        Self
    }

    pub async fn setup_config(&self, request: SetupConfigRequest) -> Result<SetupConfigResponse> {
        info!("Setup config request received");

        // Check current configuration status
        let env_path = Path::new(".env");
        let env_file_exists = env_path.exists();

        // Check current environment variables
        let openai_configured =
            env::var("OPENAI_API_KEY").is_ok() || request.openai_api_key.is_some();
        let openrouter_configured =
            env::var("OPENROUTER_API_KEY").is_ok() || request.openrouter_api_key.is_some();

        // Get current or default model configuration
        let current_models = ModelConfig {
            reasoning: env::var("LUX_MODEL_REASONING").unwrap_or_else(|_| "gpt-5".to_string()),
            normal: env::var("LUX_MODEL_NORMAL").unwrap_or_else(|_| "gpt-5".to_string()),
            mini: env::var("LUX_MODEL_MINI").unwrap_or_else(|_| "gpt-5-mini".to_string()),
            opus: env::var("LUX_MODEL_OPUS")
                .unwrap_or_else(|_| "anthropic/claude-4.1-opus".to_string()),
            sonnet: env::var("LUX_MODEL_SONNET")
                .unwrap_or_else(|_| "anthropic/claude-4-sonnet".to_string()),
            grok: env::var("LUX_MODEL_GROK").unwrap_or_else(|_| "x-ai/grok-beta".to_string()),
        };

        let models_configured = env::var("LUX_MODEL_REASONING").is_ok()
            || env::var("LUX_MODEL_NORMAL").is_ok()
            || env::var("LUX_MODEL_MINI").is_ok();

        // Determine recommended models based on preferences
        let (reasoning_model, normal_model, mini_model) = if request.use_advanced_models {
            ("gpt-5", "gpt-5", "gpt-5-mini")
        } else {
            ("gpt-4o", "gpt-4o", "gpt-4o-mini")
        };

        // Override with custom models if provided
        let reasoning_model = request
            .custom_models
            .as_ref()
            .and_then(|m| m.reasoning_model.as_deref())
            .unwrap_or(reasoning_model);
        let normal_model = request
            .custom_models
            .as_ref()
            .and_then(|m| m.normal_model.as_deref())
            .unwrap_or(normal_model);
        let mini_model = request
            .custom_models
            .as_ref()
            .and_then(|m| m.mini_model.as_deref())
            .unwrap_or(mini_model);

        // Generate .env template
        let env_template = self.generate_env_template(
            request.openai_api_key.as_deref(),
            request.openrouter_api_key.as_deref(),
            reasoning_model,
            normal_model,
            mini_model,
        );

        // Generate instructions for the host LLM
        let instructions_for_host_llm = vec![
            "📋 **INSTRUCTIONS FOR CONFIGURING LUX MCP**".to_string(),
            "".to_string(),
            "You need to create or update the .env file for Lux MCP configuration.".to_string(),
            "".to_string(),
            "**STEP 1: Check if .env exists**".to_string(),
            format!(
                "- Current status: .env file {} exist",
                if env_file_exists { "DOES" } else { "does NOT" }
            ),
            if env_file_exists {
                "- Action: UPDATE the existing .env file with the new configuration below"
                    .to_string()
            } else {
                "- Action: CREATE a new .env file with the configuration below".to_string()
            },
            "".to_string(),
            "**STEP 2: Use the Write or Edit tool**".to_string(),
            if env_file_exists {
                "- Use the Edit tool to update the .env file".to_string()
            } else {
                "- Use the Write tool to create the .env file".to_string()
            },
            "- File path: ./.env".to_string(),
            "".to_string(),
            "**STEP 3: Configuration Template**".to_string(),
            "Copy the EXACT content below into the .env file:".to_string(),
            "".to_string(),
            "```".to_string(),
            env_template.clone(),
            "```".to_string(),
            "".to_string(),
            "**STEP 4: Verify Configuration**".to_string(),
            "After creating/updating the file, verify it was saved correctly.".to_string(),
            "".to_string(),
            "**IMPORTANT NOTES:**".to_string(),
            "- Replace YOUR_OPENAI_API_KEY_HERE with the actual OpenAI API key".to_string(),
            "- Replace YOUR_OPENROUTER_API_KEY_HERE with the actual OpenRouter API key (optional)"
                .to_string(),
            "- At least ONE API key must be provided for Lux to function".to_string(),
            "- The model configuration uses the latest and most capable models".to_string(),
            "- You can customize the models by editing the LUX_MODEL_* variables".to_string(),
        ];

        // Generate next steps
        let mut next_steps = vec![];

        if !openai_configured && request.openai_api_key.is_none() {
            next_steps.push("⚠️ Add your OpenAI API key to the OPENAI_API_KEY field".to_string());
        }

        if !openrouter_configured && request.openrouter_api_key.is_none() {
            next_steps.push("💡 (Optional) Add your OpenRouter API key for access to Claude, Gemini, and other models".to_string());
        }

        if !env_file_exists {
            next_steps.push(
                "📝 Create the .env file using the Write tool with the template provided"
                    .to_string(),
            );
        } else {
            next_steps.push(
                "✏️ Update the .env file using the Edit tool with the template provided"
                    .to_string(),
            );
        }

        next_steps.push("🔄 Restart the Lux MCP server after configuration".to_string());
        next_steps.push("✅ Test the configuration with a simple confer command".to_string());

        Ok(SetupConfigResponse {
            status: if env_file_exists {
                "update_required"
            } else {
                "creation_required"
            }
            .to_string(),
            instructions_for_host_llm,
            env_template,
            env_file_path: "./.env".to_string(),
            current_config: ConfigStatus {
                env_file_exists,
                openai_configured,
                openrouter_configured,
                models_configured,
                current_models,
            },
            next_steps,
        })
    }

    fn generate_env_template(
        &self,
        openai_key: Option<&str>,
        openrouter_key: Option<&str>,
        reasoning_model: &str,
        normal_model: &str,
        mini_model: &str,
    ) -> String {
        format!(
            r#"# Lux MCP Configuration File
# Generated by setup_config tool
# Documentation: https://github.com/lux-mcp/docs

# ============================================
# API KEYS (At least one required)
# ============================================

# OpenAI API Key (for GPT-4, GPT-5, O3, O4 models)
# Get your key at: https://platform.openai.com/api-keys
OPENAI_API_KEY="{}"

# OpenRouter API Key (for Claude, Gemini, Llama, and other models)
# Get your key at: https://openrouter.ai/keys
OPENROUTER_API_KEY="{}"

# ============================================
# MODEL CONFIGURATION
# ============================================

# Main reasoning model - Used for complex reasoning tasks
# Recommended: gpt-5 (most capable), gpt-4o (standard), o3-pro (deep reasoning)
LUX_MODEL_REASONING="{}"

# Normal model - Used for standard conversational tasks
# Recommended: gpt-5 (most capable), gpt-4o (standard)
LUX_MODEL_NORMAL="{}"

# Mini model - Used for simple/fast tasks and cost optimization
# Recommended: gpt-5-mini (advanced), gpt-4o-mini (standard)
LUX_MODEL_MINI="{}"

# ============================================
# NAMED MODEL ALIASES (Optional)
# ============================================

# These allow you to use simple names like "opus", "sonnet", "grok" in tools
# Customize these to your preferred models

# Claude Opus mapping (use with model: "opus")
LUX_MODEL_OPUS="anthropic/claude-4.1-opus"

# Claude Sonnet mapping (use with model: "sonnet")
LUX_MODEL_SONNET="anthropic/claude-4-sonnet"

# Grok mapping (use with model: "grok")
LUX_MODEL_GROK="x-ai/grok-beta"

# ============================================
# OPTIONAL SETTINGS
# ============================================

# Logging level (debug, info, warn, error)
RUST_LOG="info"

# Request timeout in seconds (default: 30)
# LUX_REQUEST_TIMEOUT_SECS="60"

# Maximum retry attempts for failed requests (default: 3)
# LUX_MAX_RETRIES="3"
"#,
            openai_key.unwrap_or("YOUR_OPENAI_API_KEY_HERE"),
            openrouter_key.unwrap_or("YOUR_OPENROUTER_API_KEY_HERE"),
            reasoning_model,
            normal_model,
            mini_model,
        )
    }
}
