use anyhow::{Context, Result};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tracing::info;

use crate::llm::{
    client::{ChatMessage, LLMClient},
    config::LLMConfig,
    model_aliases::ModelResolver,
    openai::OpenAIClient,
    openrouter::OpenRouterClient,
    token_config::TokenConfig,
    Role,
};

/// Hybrid biased reasoning tool where Claude provides the reasoning
/// and an external LLM checks for bias
#[derive(Debug, Serialize, Deserialize)]
pub struct HybridBiasedReasoningRequest {
    /// The reasoning step provided by Claude
    pub reasoning_step: String,

    /// Context or original query this reasoning addresses
    pub context: Option<String>,

    /// Step number in the reasoning chain
    pub step_number: Option<u32>,

    /// Previous reasoning steps for context
    pub previous_steps: Option<Vec<String>>,

    /// Session ID for tracking
    pub session_id: Option<String>,

    /// Model to use for bias checking (defaults to configured bias checker)
    pub bias_check_model: Option<String>,

    /// Temperature for bias checking (lower = more consistent)
    pub temperature: Option<f32>,

    /// Types of bias to check for
    pub bias_types: Option<Vec<String>>,

    /// File paths to include in context for bias checking
    pub file_paths: Option<Vec<String>>,

    /// Whether to include file contents in the analysis
    pub include_file_contents: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HybridBiasedReasoningResponse {
    /// Overall bias assessment
    pub bias_detected: bool,

    /// Confidence in the bias assessment (0.0-1.0)
    pub confidence: f32,

    /// Specific biases found
    pub biases_found: Vec<BiasDetail>,

    /// Suggestions for improvement
    pub suggestions: Vec<String>,

    /// Overall bias score (0.0 = no bias, 1.0 = severe bias)
    pub bias_score: f32,

    /// Whether Claude should revise this step
    pub revision_recommended: bool,

    /// Alternative phrasing if bias detected
    pub alternative_phrasing: Option<String>,

    /// Model used for bias checking
    pub model_used: String,

    /// Session ID if provided
    pub session_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BiasDetail {
    pub bias_type: String,
    pub description: String,
    pub severity: String,         // "low", "medium", "high"
    pub location: Option<String>, // Where in the text
}

#[derive(Debug, Default)]
struct SessionState {
    reasoning_history: Vec<String>,
    bias_checks: Vec<BiasDetail>,
    total_bias_score: f32,
    steps_checked: u32,
    file_contexts: HashMap<String, String>, // Store file contents for session
}

/// Tool for hybrid biased reasoning
pub struct HybridBiasedReasoningTool {
    sessions: Arc<Mutex<HashMap<String, SessionState>>>,
    model_resolver: ModelResolver,
}

impl HybridBiasedReasoningTool {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            model_resolver: ModelResolver::new(),
        }
    }

    /// Read files and return their contents
    fn read_files(&self, file_paths: &[String]) -> Result<HashMap<String, String>> {
        let mut file_contents = HashMap::new();

        for path in file_paths {
            let file_path = Path::new(path);
            if file_path.exists() && file_path.is_file() {
                match fs::read_to_string(file_path) {
                    Ok(content) => {
                        info!("Read file for bias context: {}", path);
                        file_contents.insert(path.clone(), content);
                    }
                    Err(e) => {
                        info!("Failed to read file {}: {}", path, e);
                        // Continue with other files even if one fails
                    }
                }
            } else {
                info!("File not found or not a file: {}", path);
            }
        }

        Ok(file_contents)
    }

    pub async fn check_reasoning_bias(
        &self,
        request: HybridBiasedReasoningRequest,
    ) -> Result<HybridBiasedReasoningResponse> {
        let session_id = request
            .session_id
            .clone()
            .unwrap_or_else(|| "default".to_string());

        // Read files if provided
        let mut file_contents = HashMap::new();
        if let Some(ref file_paths) = request.file_paths {
            if request.include_file_contents.unwrap_or(true) {
                file_contents = self.read_files(file_paths)?;
            }
        }

        // Update session history and store file contexts
        {
            let mut sessions = self.sessions.lock();
            let session = sessions
                .entry(session_id.clone())
                .or_insert_with(SessionState::default);
            session
                .reasoning_history
                .push(request.reasoning_step.clone());
            session.steps_checked += 1;

            // Store file contents in session for reference
            for (path, content) in &file_contents {
                session.file_contexts.insert(path.clone(), content.clone());
            }
        }

        // Get LLM configuration
        let config = LLMConfig::from_env()?;
        let model_resolver = ModelResolver::with_config(Some(config.clone()));
        let model_name = request
            .bias_check_model
            .as_deref()
            .unwrap_or(&config.model_mini);
        let resolved_model = model_resolver.resolve(model_name);

        // Create LLM client for bias checking
        let client: Box<dyn LLMClient> = if model_resolver.is_openrouter_model(&resolved_model) {
            Box::new(OpenRouterClient::new(
                config
                    .openrouter_api_key
                    .context("OpenRouter API key not configured")?,
                resolved_model.clone(),
                config.openrouter_base_url,
            )?)
        } else {
            Box::new(OpenAIClient::new(
                config
                    .openai_api_key
                    .context("OpenAI API key not configured")?,
                resolved_model.clone(),
                config.openai_base_url,
            )?)
        };

        // Build bias checking prompt with file contents
        let bias_check_prompt = self.build_bias_check_prompt(&request, &file_contents)?;

        // Get bias analysis from external LLM
        let messages = vec![
            ChatMessage {
                role: Role::System,
                content: BIAS_CHECKER_SYSTEM_PROMPT.to_string(),
            },
            ChatMessage {
                role: Role::User,
                content: bias_check_prompt,
            },
        ];

        let temperature = request.temperature.unwrap_or(0.3); // Low temperature for consistency
        let max_tokens = Some(TokenConfig::get_optimal_tokens(&resolved_model));

        let response = client
            .complete(messages, Some(temperature), max_tokens)
            .await
            .context("Failed to get bias analysis from LLM")?;

        // Parse the bias analysis
        let analysis = self.parse_bias_analysis(&response.content, &resolved_model)?;

        // Update session with findings
        {
            let mut sessions = self.sessions.lock();
            if let Some(session) = sessions.get_mut(&session_id) {
                session.bias_checks.extend(analysis.biases_found.clone());
                session.total_bias_score += analysis.bias_score;
            }
        }

        info!(
            "Hybrid bias check - Step: {:?}, Bias detected: {}, Score: {:.2}, Model: {}",
            request.step_number, analysis.bias_detected, analysis.bias_score, resolved_model
        );

        Ok(HybridBiasedReasoningResponse {
            bias_detected: analysis.bias_detected,
            confidence: analysis.confidence,
            biases_found: analysis.biases_found,
            suggestions: analysis.suggestions,
            bias_score: analysis.bias_score,
            revision_recommended: analysis.revision_recommended,
            alternative_phrasing: analysis.alternative_phrasing,
            model_used: resolved_model,
            session_id: if session_id == "default" {
                None
            } else {
                Some(session_id)
            },
        })
    }

    fn build_bias_check_prompt(
        &self,
        request: &HybridBiasedReasoningRequest,
        file_contents: &HashMap<String, String>,
    ) -> Result<String> {
        let mut prompt = String::new();

        // Add file contents if provided
        if !file_contents.is_empty() {
            prompt.push_str("=== FILE CONTEXT ===\n");
            for (path, content) in file_contents {
                prompt.push_str(&format!("File: {}\n", path));
                // Limit file content to avoid token limits
                let truncated = if content.len() > 5000 {
                    format!("{}... [truncated]", &content[..5000])
                } else {
                    content.clone()
                };
                prompt.push_str(&format!("```\n{}\n```\n\n", truncated));
            }
            prompt.push_str("=== END FILE CONTEXT ===\n\n");
        }

        // Add context if provided
        if let Some(context) = &request.context {
            prompt.push_str(&format!("Original Context/Query: {}\n\n", context));
        }

        // Add previous steps if provided
        if let Some(previous_steps) = &request.previous_steps {
            if !previous_steps.is_empty() {
                prompt.push_str("Previous Reasoning Steps:\n");
                for (i, step) in previous_steps.iter().enumerate() {
                    prompt.push_str(&format!("Step {}: {}\n", i + 1, step));
                }
                prompt.push_str("\n");
            }
        }

        // Add current reasoning step
        if let Some(step_num) = request.step_number {
            prompt.push_str(&format!("Current Reasoning Step {}:\n", step_num));
        } else {
            prompt.push_str("Current Reasoning Step:\n");
        }
        prompt.push_str(&format!("{}\n\n", request.reasoning_step));

        // Add specific bias types to check if provided
        if let Some(bias_types) = &request.bias_types {
            prompt.push_str("Please check specifically for these types of bias:\n");
            for bias_type in bias_types {
                prompt.push_str(&format!("- {}\n", bias_type));
            }
            prompt.push_str("\n");
        }

        prompt.push_str("Analyze the above reasoning for any biases, logical fallacies, or problematic assumptions. Provide:\n");
        prompt.push_str("1. A list of specific biases found (if any)\n");
        prompt.push_str("2. Severity assessment for each bias\n");
        prompt.push_str("3. Concrete suggestions for improvement\n");
        prompt.push_str("4. An alternative phrasing if significant bias is detected\n");
        prompt.push_str("5. An overall bias score from 0.0 (no bias) to 1.0 (severe bias)\n");

        if !file_contents.is_empty() {
            prompt.push_str("\nIMPORTANT: Consider the file context provided above when analyzing for biases.\n");
        }

        Ok(prompt)
    }

    fn parse_bias_analysis(
        &self,
        content: &str,
        model: &str,
    ) -> Result<HybridBiasedReasoningResponse> {
        // Parse the LLM response to extract bias information
        // This is a simplified parser - in production, you might want structured output

        let mut biases_found = Vec::new();
        let mut suggestions = Vec::new();
        let mut bias_score: f32 = 0.0;
        let mut alternative_phrasing = None;

        // Look for bias indicators in the response
        let lower_content = content.to_lowercase();

        // Check for common bias types
        if lower_content.contains("confirmation bias") {
            biases_found.push(BiasDetail {
                bias_type: "Confirmation Bias".to_string(),
                description: "Favoring information that confirms existing beliefs".to_string(),
                severity: "medium".to_string(),
                location: None,
            });
            bias_score += 0.3;
        }

        if lower_content.contains("anchoring bias") {
            biases_found.push(BiasDetail {
                bias_type: "Anchoring Bias".to_string(),
                description: "Over-relying on first piece of information".to_string(),
                severity: "medium".to_string(),
                location: None,
            });
            bias_score += 0.3;
        }

        if lower_content.contains("availability bias")
            || lower_content.contains("availability heuristic")
        {
            biases_found.push(BiasDetail {
                bias_type: "Availability Bias".to_string(),
                description: "Overweighting easily recalled information".to_string(),
                severity: "low".to_string(),
                location: None,
            });
            bias_score += 0.2;
        }

        if lower_content.contains("hasty generalization") {
            biases_found.push(BiasDetail {
                bias_type: "Hasty Generalization".to_string(),
                description: "Drawing broad conclusions from limited evidence".to_string(),
                severity: "high".to_string(),
                location: None,
            });
            bias_score += 0.4;
        }

        if lower_content.contains("false dichotomy") || lower_content.contains("false dilemma") {
            biases_found.push(BiasDetail {
                bias_type: "False Dichotomy".to_string(),
                description: "Presenting only two options when more exist".to_string(),
                severity: "medium".to_string(),
                location: None,
            });
            bias_score += 0.3;
        }

        // Extract suggestions (look for numbered lists or bullet points)
        for line in content.lines() {
            if line.starts_with("- ")
                || line.starts_with("* ")
                || (line.len() > 2
                    && line.chars().nth(0).unwrap().is_numeric()
                    && line.chars().nth(1).unwrap() == '.')
            {
                let suggestion = line.trim_start_matches(|c: char| !c.is_alphabetic());
                if suggestion.len() > 10 && !suggestion.to_lowercase().contains("bias") {
                    suggestions.push(suggestion.to_string());
                }
            }
        }

        // Look for alternative phrasing
        if lower_content.contains("instead") || lower_content.contains("alternative") {
            // Try to extract alternative phrasing
            if let Some(idx) = lower_content.find("instead") {
                let after_instead = &content[idx + 7..];
                if let Some(end_idx) = after_instead.find('.') {
                    alternative_phrasing = Some(after_instead[..end_idx].trim().to_string());
                }
            }
        }

        // Calculate confidence based on model and analysis depth
        let confidence = if model.contains("o4") || model.contains("o3") {
            0.85
        } else if model.contains("gpt-4") {
            0.80
        } else {
            0.70
        };

        // Clamp bias score
        bias_score = bias_score.min(1.0);

        Ok(HybridBiasedReasoningResponse {
            bias_detected: !biases_found.is_empty(),
            confidence,
            biases_found,
            suggestions,
            bias_score,
            revision_recommended: bias_score > 0.5,
            alternative_phrasing,
            model_used: model.to_string(),
            session_id: None, // Will be set by caller
        })
    }

    pub fn get_session_summary(&self, session_id: Option<String>) -> Result<String> {
        let session_id = session_id.unwrap_or_else(|| "default".to_string());
        let sessions = self.sessions.lock();

        if let Some(session) = sessions.get(&session_id) {
            let avg_bias = if session.steps_checked > 0 {
                session.total_bias_score / session.steps_checked as f32
            } else {
                0.0
            };

            let file_count = session.file_contexts.len();
            let file_list = if file_count > 0 {
                format!(
                    "\nFiles in context: {}",
                    session
                        .file_contexts
                        .keys()
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            } else {
                String::new()
            };

            let summary = format!(
                "Session '{}': {} steps checked, {} biases found, average bias score: {:.2}, {} files loaded{}",
                session_id,
                session.steps_checked,
                session.bias_checks.len(),
                avg_bias,
                file_count,
                file_list
            );
            Ok(summary)
        } else {
            Ok(format!("No session found with ID: {}", session_id))
        }
    }

    /// Get previously loaded file contents from session
    pub fn get_session_files(&self, session_id: Option<String>) -> HashMap<String, String> {
        let session_id = session_id.unwrap_or_else(|| "default".to_string());
        let sessions = self.sessions.lock();

        sessions
            .get(&session_id)
            .map(|session| session.file_contexts.clone())
            .unwrap_or_default()
    }

    pub fn clear_session(&self, session_id: Option<String>) -> Result<()> {
        let session_id = session_id.unwrap_or_else(|| "default".to_string());
        let mut sessions = self.sessions.lock();
        sessions.remove(&session_id);
        info!("Cleared hybrid bias checking session: {}", session_id);
        Ok(())
    }
}

impl Default for HybridBiasedReasoningTool {
    fn default() -> Self {
        Self::new()
    }
}

const BIAS_CHECKER_SYSTEM_PROMPT: &str = r#"You are an expert bias detector and critical thinking analyst. Your role is to:

1. Identify cognitive biases, logical fallacies, and problematic assumptions in reasoning
2. Assess the severity of each bias found
3. Provide specific, actionable suggestions for improvement
4. Maintain objectivity and focus on the reasoning structure, not the content

Common biases to check for:
- Confirmation bias: Favoring information that confirms existing beliefs
- Anchoring bias: Over-relying on the first piece of information
- Availability heuristic: Overweighting easily recalled information
- Selection bias: Cherry-picking data that supports a conclusion
- Hasty generalization: Drawing broad conclusions from limited samples
- False dichotomy: Presenting limited options when more exist
- Ad hominem: Attacking the person rather than the argument
- Straw man: Misrepresenting an argument to make it easier to attack
- Slippery slope: Assuming one event will lead to extreme consequences
- Appeal to authority: Using authority as evidence without justification
- Bandwagon fallacy: Assuming something is true because many believe it
- Circular reasoning: Using the conclusion as a premise

Be thorough but fair. Not all reasoning contains bias, and sometimes apparent biases are justified by context."#;
