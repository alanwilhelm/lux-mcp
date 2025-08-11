use anyhow::{Context, Result};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use crate::llm::{
    client::{ChatMessage, LLMClient},
    config::LLMConfig,
    model_aliases::ModelResolver,
    openai::OpenAIClient,
    openrouter::OpenRouterClient,
    token_config::TokenConfig,
    Role,
};

/// Sequential thinking with external LLM integration
/// Combines Anthropic's simple structure with LLM-powered thought generation

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalThoughtData {
    pub thought: String,
    pub thought_number: u32,
    pub total_thoughts: u32,
    pub next_thought_needed: bool,

    #[serde(default)]
    pub is_revision: bool,
    #[serde(default)]
    pub revises_thought: Option<u32>,
    #[serde(default)]
    pub branch_from_thought: Option<u32>,
    #[serde(default)]
    pub branch_id: Option<String>,
    #[serde(default)]
    pub needs_more_thoughts: bool,

    // LLM-specific fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_used: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SequentialThinkingExternalRequest {
    pub thought: String, // For step 1: the problem/query, for 2+: guidance or continuation
    pub thought_number: u32,
    pub total_thoughts: u32,
    pub next_thought_needed: bool,

    #[serde(default)]
    pub is_revision: bool,
    #[serde(default)]
    pub revises_thought: Option<u32>,
    #[serde(default)]
    pub branch_from_thought: Option<u32>,
    #[serde(default)]
    pub branch_id: Option<String>,
    #[serde(default)]
    pub needs_more_thoughts: bool,

    // Session and model configuration
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default = "default_temperature")]
    pub temperature: f32,

    // Control whether to use LLM or just record
    #[serde(default = "default_true")]
    pub use_llm: bool,
}

fn default_temperature() -> f32 {
    0.7
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SequentialThinkingExternalResponse {
    pub thought_number: u32,
    pub total_thoughts: u32,
    pub next_thought_needed: bool,
    pub thought_content: String,
    pub branches: Vec<String>,
    pub thought_history_length: usize,
    pub status: String, // "thinking", "revision", "branch", "complete"

    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_used: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_hint: Option<String>,
}

#[derive(Debug, Default)]
struct ExternalSessionState {
    thought_history: Vec<ExternalThoughtData>,
    branches: HashMap<String, Vec<ExternalThoughtData>>,
    original_query: Option<String>,
}

/// Tool for managing sequential thinking with external LLM
pub struct SequentialThinkingExternalTool {
    sessions: Arc<Mutex<HashMap<String, ExternalSessionState>>>,
}

impl SequentialThinkingExternalTool {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn process_thought(
        &self,
        request: SequentialThinkingExternalRequest,
    ) -> Result<SequentialThinkingExternalResponse> {
        let session_id = request
            .session_id
            .clone()
            .unwrap_or_else(|| "default".to_string());

        // Get or create session and extract needed data
        let (history_clone, original_query_clone) = {
            let mut sessions = self.sessions.lock();
            let session = sessions
                .entry(session_id.clone())
                .or_insert_with(ExternalSessionState::default);

            // Store original query if this is the first thought
            if request.thought_number == 1 {
                session.original_query = Some(request.thought.clone());
            }

            // Clone needed data before async call
            (
                session.thought_history.clone(),
                session.original_query.clone(),
            )
        }; // Lock is dropped here

        // Generate thought content using LLM if requested
        let (thought_content, model_used, confidence) = if request.use_llm {
            Self::generate_thought_with_llm(
                &request,
                &history_clone,
                original_query_clone.as_deref(),
            )
            .await?
        } else {
            // If not using LLM, just use the provided thought
            (request.thought.clone(), None, None)
        };

        // Re-acquire lock to update state
        let mut sessions = self.sessions.lock();
        let session = sessions
            .entry(session_id.clone())
            .or_insert_with(ExternalSessionState::default);

        // Create thought data
        let mut thought_data = ExternalThoughtData {
            thought: thought_content.clone(),
            thought_number: request.thought_number,
            total_thoughts: request.total_thoughts,
            next_thought_needed: request.next_thought_needed,
            is_revision: request.is_revision,
            revises_thought: request.revises_thought,
            branch_from_thought: request.branch_from_thought,
            branch_id: request.branch_id.clone(),
            needs_more_thoughts: request.needs_more_thoughts,
            model_used: model_used.clone(),
            confidence,
        };

        // Auto-adjust total_thoughts if needed
        if thought_data.thought_number > thought_data.total_thoughts {
            thought_data.total_thoughts = thought_data.thought_number;
        }

        // Add to history
        session.thought_history.push(thought_data.clone());

        // Track branches
        if let (Some(_), Some(branch_id)) =
            (thought_data.branch_from_thought, &thought_data.branch_id)
        {
            session
                .branches
                .entry(branch_id.clone())
                .or_insert_with(Vec::new)
                .push(thought_data.clone());
        }

        // Determine status
        let status = if !thought_data.next_thought_needed {
            "complete"
        } else if thought_data.is_revision {
            "revision"
        } else if thought_data.branch_from_thought.is_some() {
            "branch"
        } else {
            "thinking"
        };

        // Generate reasoning hint for next step
        let reasoning_hint = if thought_data.next_thought_needed {
            Some(Self::generate_reasoning_hint(
                &thought_data,
                &session.thought_history,
            ))
        } else {
            None
        };

        let branches: Vec<String> = session.branches.keys().cloned().collect();

        // Log if enabled
        if std::env::var("DISABLE_THOUGHT_LOGGING")
            .unwrap_or_default()
            .to_lowercase()
            != "true"
        {
            Self::log_external_thought(&thought_data);
        }

        info!(
            "Sequential external thought {}/{} - Status: {}, Model: {:?}, Confidence: {:?}",
            thought_data.thought_number,
            thought_data.total_thoughts,
            status,
            model_used,
            confidence
        );

        Ok(SequentialThinkingExternalResponse {
            thought_number: thought_data.thought_number,
            total_thoughts: thought_data.total_thoughts,
            next_thought_needed: thought_data.next_thought_needed,
            thought_content,
            branches,
            thought_history_length: thought_data.thought_number as usize,
            status: status.to_string(),
            session_id: if session_id == "default" {
                None
            } else {
                Some(session_id)
            },
            model_used,
            confidence,
            reasoning_hint,
        })
    }

    async fn generate_thought_with_llm(
        request: &SequentialThinkingExternalRequest,
        history: &[ExternalThoughtData],
        original_query: Option<&str>,
    ) -> Result<(String, Option<String>, Option<f32>)> {
        // Get LLM configuration
        let config = match LLMConfig::from_env() {
            Ok(c) => {
                info!("Loaded LLM config successfully");
                c
            }
            Err(e) => {
                warn!("Failed to load LLM config: {}. Using defaults.", e);
                LLMConfig::default()
            }
        };

        // Determine which model to use with fallback chain
        let model_name = request
            .model
            .as_deref()
            .or(Some(&config.model_normal))
            .filter(|s| !s.is_empty())
            .or(Some(&config.model_reasoning))
            .filter(|s| !s.is_empty())
            .unwrap_or("gpt-4o"); // Ultimate fallback

        info!("Using model '{}' for sequential thinking", model_name);

        let model_resolver = ModelResolver::with_config(Some(config.clone()));
        let resolved_model = model_resolver.resolve(model_name);

        // Create LLM client with better error handling
        info!("Creating LLM client for resolved model: {}", resolved_model);
        let client: Box<dyn LLMClient> = if model_resolver.is_openrouter_model(&resolved_model) {
            info!("Model is OpenRouter model, checking API key...");
            let api_key = config.openrouter_api_key.ok_or_else(|| {
                anyhow::anyhow!(
                    "OpenRouter API key not configured for model: {}",
                    resolved_model
                )
            })?;
            info!("Creating OpenRouter client...");
            Box::new(
                OpenRouterClient::new(api_key, resolved_model.clone(), config.openrouter_base_url)
                    .map_err(|e| anyhow::anyhow!("Failed to create OpenRouter client: {}", e))?,
            )
        } else {
            info!("Model is OpenAI model, checking API key...");
            let api_key = config.openai_api_key.ok_or_else(|| {
                anyhow::anyhow!(
                    "OpenAI API key not configured for model: {}",
                    resolved_model
                )
            })?;
            info!("API key found, creating OpenAI client...");
            Box::new(
                OpenAIClient::new(api_key, resolved_model.clone(), config.openai_base_url)
                    .map_err(|e| anyhow::anyhow!("Failed to create OpenAI client: {}", e))?,
            )
        };
        info!("Successfully created LLM client");

        // Build prompt based on thought number and context
        let prompt = if request.thought_number == 1 {
            // First thought - analyze the problem
            format!(
                "You are helping with step-by-step sequential thinking. This is step 1 of {}.\n\n\
                Problem/Query: {}\n\n\
                Please provide your initial analysis and understanding of this problem. \
                Focus on identifying key aspects, challenges, and potential approaches.",
                request.total_thoughts, request.thought
            )
        } else {
            // Subsequent thoughts - build on history
            let mut context = String::new();

            if let Some(query) = original_query {
                context.push_str(&format!("Original problem: {}\n\n", query));
            }

            context.push_str("Previous thoughts:\n");
            for thought in history.iter().take(5) {
                // Show last 5 thoughts for context
                context.push_str(&format!(
                    "Thought {}: {}\n",
                    thought.thought_number, thought.thought
                ));
            }

            let instruction = if request.is_revision {
                format!(
                    "This is a REVISION of thought {}. Please reconsider and provide an improved analysis:\n\
                    Guidance: {}",
                    request.revises_thought.unwrap_or(0),
                    request.thought
                )
            } else if request.branch_from_thought.is_some() {
                format!(
                    "This is a BRANCH from thought {} (branch: {}). Explore an alternative approach:\n\
                    Guidance: {}",
                    request.branch_from_thought.unwrap_or(0),
                    request.branch_id.as_ref().unwrap_or(&"unknown".to_string()),
                    request.thought
                )
            } else {
                format!(
                    "This is step {} of {}. Continue the analysis:\n\
                    Guidance: {}",
                    request.thought_number, request.total_thoughts, request.thought
                )
            };

            format!("{}\n{}", context, instruction)
        };

        // Build messages
        let messages = vec![
            ChatMessage {
                role: Role::System,
                content:
                    "You are a precise analytical assistant helping with sequential reasoning. \
                         Provide clear, focused thoughts that build on previous analysis. \
                         Be concise but thorough."
                        .to_string(),
            },
            ChatMessage {
                role: Role::User,
                content: prompt,
            },
        ];

        // Get token limit for this model
        let max_tokens = Some(TokenConfig::get_optimal_tokens(&resolved_model));

        // Call LLM with detailed error reporting
        let response = match client
            .complete(messages, Some(request.temperature), max_tokens)
            .await
        {
            Ok(resp) => resp,
            Err(e) => {
                error!("LLM call failed for model '{}': {}", resolved_model, e);
                return Err(anyhow::anyhow!(
                    "Failed to generate thought with LLM '{}': {}",
                    resolved_model,
                    e
                ));
            }
        };

        // Estimate confidence based on response characteristics
        let confidence = Self::estimate_confidence(&response.content);

        Ok((response.content, Some(resolved_model), Some(confidence)))
    }

    fn estimate_confidence(content: &str) -> f32 {
        // Simple heuristic for confidence
        let mut confidence: f32 = 0.7;

        // Increase confidence for certain patterns
        if content.contains("clearly")
            || content.contains("definitely")
            || content.contains("certain")
        {
            confidence += 0.1;
        }
        if content.contains("however") || content.contains("but") || content.contains("although") {
            confidence -= 0.1;
        }
        if content.contains("might") || content.contains("possibly") || content.contains("perhaps")
        {
            confidence -= 0.15;
        }

        // Clamp between 0.2 and 0.95
        confidence.max(0.2).min(0.95)
    }

    fn generate_reasoning_hint(
        current: &ExternalThoughtData,
        history: &[ExternalThoughtData],
    ) -> String {
        if current.is_revision {
            "Consider what was wrong with the previous approach and how to improve it".to_string()
        } else if current.branch_from_thought.is_some() {
            "Explore a different angle or methodology from the branching point".to_string()
        } else if current.thought_number < current.total_thoughts / 2 {
            "Continue exploring and analyzing the problem space".to_string()
        } else if current.thought_number < current.total_thoughts {
            "Start synthesizing insights and moving toward a conclusion".to_string()
        } else {
            "Finalize your analysis and present conclusions".to_string()
        }
    }

    fn log_external_thought(thought_data: &ExternalThoughtData) {
        let prefix = if thought_data.is_revision {
            format!(
                "🔄 Revision (revising thought {})",
                thought_data.revises_thought.unwrap_or(0)
            )
        } else if let Some(branch_from) = thought_data.branch_from_thought {
            format!(
                "🌿 Branch (from thought {}, ID: {})",
                branch_from,
                thought_data
                    .branch_id
                    .as_ref()
                    .unwrap_or(&"unknown".to_string())
            )
        } else {
            "🤖 AI Thought".to_string()
        };

        let model_info = thought_data
            .model_used
            .as_ref()
            .map(|m| format!(" [{}]", m))
            .unwrap_or_default();

        let confidence_info = thought_data
            .confidence
            .map(|c| format!(" (confidence: {:.2})", c))
            .unwrap_or_default();

        debug!(
            "\n┌─────────────────────────────────────────┐\n\
             │ {} {}/{}{}{}                            │\n\
             ├─────────────────────────────────────────┤\n\
             │ {}                                       │\n\
             └─────────────────────────────────────────┘",
            prefix,
            thought_data.thought_number,
            thought_data.total_thoughts,
            model_info,
            confidence_info,
            thought_data.thought
        );
    }

    // Helper function to clear a session
    pub fn clear_session(&self, session_id: Option<String>) -> Result<()> {
        let session_id = session_id.unwrap_or_else(|| "default".to_string());
        let mut sessions = self.sessions.lock();
        sessions.remove(&session_id);
        info!("Cleared external session: {}", session_id);
        Ok(())
    }
}

impl Default for SequentialThinkingExternalTool {
    fn default() -> Self {
        Self::new()
    }
}
