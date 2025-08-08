use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use tracing::{info, error};
use parking_lot::Mutex;
use std::collections::HashMap;

use crate::llm::{
    client::{ChatMessage, LLMClient},
    config::LLMConfig,
    model_aliases::ModelResolver,
    openai::OpenAIClient,
    openrouter::OpenRouterClient,
    Role,
};
use crate::session::SessionManager;

#[derive(Debug, Serialize, Deserialize)]
pub struct BiasedReasoningRequest {
    // For step 1: the query, for 2+: guidance or continuation
    pub step_content: String,
    pub step_number: u32,
    pub total_steps: u32,
    pub next_step_needed: bool,
    
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    #[serde(default)]
    pub bias_config: BiasCheckConfig,
}

fn default_temperature() -> f32 { 0.7 }

#[derive(Debug, Serialize, Deserialize)]
pub struct BiasCheckConfig {
    #[serde(default = "default_true")]
    pub check_confirmation_bias: bool,
    #[serde(default = "default_true")]
    pub check_anchoring_bias: bool,
    #[serde(default = "default_true")]
    pub check_availability_bias: bool,
    #[serde(default = "default_true")]
    pub check_reasoning_errors: bool,
    #[serde(default = "default_bias_threshold")]
    pub bias_threshold: f32,
}

fn default_true() -> bool { true }
fn default_bias_threshold() -> f32 { 0.7 }

impl Default for BiasCheckConfig {
    fn default() -> Self {
        Self {
            check_confirmation_bias: true,
            check_anchoring_bias: true,
            check_availability_bias: true,
            check_reasoning_errors: true,
            bias_threshold: 0.7,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BiasedReasoningResponse {
    pub current_step: u32,
    pub step_reasoning: String,
    pub bias_analysis: BiasCheckResult,
    pub corrected_reasoning: Option<String>,
    pub status: String,  // "reasoning", "completed"
    pub next_action: Option<String>,
    pub model_used: String,
    pub verifier_used: String,
    pub overall_progress: f32,  // 0.0 to 1.0
    pub reasoning_quality: f32, // 0.0 to 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiasCheckResult {
    pub has_bias: bool,
    pub bias_types: Vec<BiasType>,
    pub severity: Option<BiasSeverity>,
    pub explanation: String,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BiasType {
    ConfirmationBias,
    AnchoringBias,
    AvailabilityBias,
    ReasoningError,
    LogicalFallacy,
    EmotionalReasoning,
    OverGeneralization,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BiasSeverity {
    Low,
    Medium,
    High,
    Critical,
}

// Session state for maintaining context between steps
struct BiasedReasoningState {
    original_query: String,
    reasoning_history: Vec<ReasoningStep>,
    conversation_history: Vec<ChatMessage>,
    bias_count: HashMap<BiasType, u32>,
    total_corrections: u32,
}

#[derive(Debug, Clone)]
struct ReasoningStep {
    step_number: u32,
    original_thought: String,
    bias_check: BiasCheckResult,
    corrected_thought: Option<String>,
    quality_score: f32,
}

pub struct BiasedReasoningTool {
    session_manager: Arc<SessionManager>,
    openai_client: Option<Arc<dyn LLMClient>>,
    openrouter_clients: Vec<(String, Arc<dyn LLMClient>)>,
    model_resolver: ModelResolver,
    config: LLMConfig,
    // State management for step-by-step processing
    state_store: Arc<Mutex<HashMap<String, BiasedReasoningState>>>,
}

impl BiasedReasoningTool {
    pub fn new(config: LLMConfig, session_manager: Arc<SessionManager>) -> Result<Self> {
        let model_resolver = ModelResolver::new();
        
        let openai_client = if let Some(api_key) = &config.openai_api_key {
            let client = OpenAIClient::new(
                api_key.clone(),
                config.default_reasoning_model.clone(),
                config.openai_base_url.clone(),
            )?;
            Some(Arc::new(client) as Arc<dyn LLMClient>)
        } else {
            None
        };
        
        let mut openrouter_clients = Vec::new();
        if let Some(api_key) = &config.openrouter_api_key {
            let common_models = vec![
                "anthropic/claude-3-opus",
                "google/gemini-2.5-pro",
            ];
            
            for model in common_models {
                let client = OpenRouterClient::new(
                    api_key.clone(),
                    model.to_string(),
                    config.openrouter_base_url.clone(),
                )?;
                openrouter_clients.push((
                    model.to_string(),
                    Arc::new(client) as Arc<dyn LLMClient>
                ));
            }
        }
        
        Ok(Self {
            session_manager,
            openai_client,
            openrouter_clients,
            model_resolver,
            config,
            state_store: Arc::new(Mutex::new(HashMap::new())),
        })
    }
    
    pub async fn process_step(&self, request: BiasedReasoningRequest) -> Result<BiasedReasoningResponse> {
        // Get or create session
        let session_id = self.session_manager.get_or_create_session(request.session_id);
        let monitor = self.session_manager.get_monitor(&session_id)?;
        
        // Validate step number
        if request.step_number < 1 {
            anyhow::bail!("step_number must be at least 1");
        }
        
        if request.total_steps < 1 {
            anyhow::bail!("total_steps must be at least 1");
        }
        
        // ALWAYS use configured defaults
        let primary_model = self.model_resolver.resolve(&self.config.default_reasoning_model);
        let verifier_model = self.model_resolver.resolve(&self.config.default_bias_checker_model);
        
        info!("Processing step {} of {} with primary: '{}', verifier: '{}'", 
              request.step_number, request.total_steps, primary_model, verifier_model);
        
        // Get or create state
        let mut state_guard = self.state_store.lock();
        let state = state_guard.entry(session_id.clone()).or_insert_with(|| {
            BiasedReasoningState {
                original_query: String::new(),
                reasoning_history: Vec::new(),
                conversation_history: vec![
                    ChatMessage {
                        role: Role::System,
                        content: "You are a reasoning assistant. Think through problems step-by-step, showing your thinking clearly.".to_string(),
                    }
                ],
                bias_count: HashMap::new(),
                total_corrections: 0,
            }
        });
        
        // For first step, store the original query
        if request.step_number == 1 {
            state.original_query = request.step_content.clone();
            // Reset state for new reasoning session
            state.reasoning_history.clear();
            state.bias_count.clear();
            state.total_corrections = 0;
            state.conversation_history = vec![
                ChatMessage {
                    role: Role::System,
                    content: "You are a reasoning assistant. Think through problems step-by-step, showing your thinking clearly.".to_string(),
                },
            ];
            
            // Reset monitor
            let mut monitor_guard = monitor.lock();
            monitor_guard.reset_session();
        }
        
        // Build context from previous steps
        let context = self.build_context(state);
        
        // Generate reasoning for this step
        let primary_client = self.get_client_for_model(&primary_model)?;
        let verifier_client = self.get_client_for_model(&verifier_model)?;
        
        let user_prompt = self.build_user_prompt(&request, &context, state);
        
        // Add user prompt to conversation
        state.conversation_history.push(ChatMessage {
            role: Role::User,
            content: user_prompt.clone(),
        });
        
        // Generate primary reasoning
        info!("🧠 Step {}: Generating reasoning with {}", request.step_number, primary_model);
        let primary_start = Instant::now();
        
        let primary_response = primary_client
            .complete(
                state.conversation_history.clone(),
                Some(request.temperature),
                Some(10000),
            )
            .await?;
        
        let primary_duration = primary_start.elapsed();
        info!("✅ {} completed step {} in {:?}", primary_model, request.step_number, primary_duration);
        
        let step_reasoning = primary_response.content.clone();
        
        // Add assistant response to conversation
        state.conversation_history.push(ChatMessage {
            role: Role::Assistant,
            content: step_reasoning.clone(),
        });
        
        // Check for bias
        info!("🔍 Step {}: Checking for bias with {}", request.step_number, verifier_model);
        let bias_check = self.check_step_for_bias(
            &step_reasoning,
            &state.original_query,
            request.step_number,
            &verifier_client,
            &request.bias_config,
            &verifier_model,
        ).await?;
        
        // Generate correction if needed
        let corrected_reasoning = if bias_check.has_bias && bias_check.severity.is_some() {
            match &bias_check.severity {
                Some(BiasSeverity::High) | Some(BiasSeverity::Critical) => {
                    info!("⚠️ Step {}: High/Critical bias detected - generating correction", request.step_number);
                    let corrected = self.generate_corrected_thought(
                        &step_reasoning,
                        &bias_check,
                        &verifier_client,
                    ).await?;
                    state.total_corrections += 1;
                    Some(corrected)
                },
                _ => None,
            }
        } else {
            None
        };
        
        // Update bias counts
        for bias_type in &bias_check.bias_types {
            *state.bias_count.entry(bias_type.clone()).or_insert(0) += 1;
        }
        
        // Calculate quality score
        let quality_score = self.calculate_step_quality(&bias_check);
        
        // Store this step in history
        state.reasoning_history.push(ReasoningStep {
            step_number: request.step_number,
            original_thought: step_reasoning.clone(),
            bias_check: bias_check.clone(),
            corrected_thought: corrected_reasoning.clone(),
            quality_score,
        });
        
        // Update monitor
        {
            let mut monitor_guard = monitor.lock();
            let thought_content = corrected_reasoning.as_ref().unwrap_or(&step_reasoning);
            monitor_guard.process_thought(thought_content, None);
        }
        
        // Determine status and next action
        let status = if request.next_step_needed {
            "reasoning".to_string()
        } else {
            "completed".to_string()
        };
        
        let next_action = if request.next_step_needed {
            Some(format!(
                "Continue with step {}. Focus on: {}",
                request.step_number + 1,
                self.suggest_next_focus(state, &bias_check)
            ))
        } else {
            None
        };
        
        let overall_progress = request.step_number as f32 / request.total_steps as f32;
        
        // Calculate average reasoning quality
        let reasoning_quality = if state.reasoning_history.is_empty() {
            quality_score
        } else {
            state.reasoning_history.iter()
                .map(|s| s.quality_score)
                .sum::<f32>() / state.reasoning_history.len() as f32
        };
        
        // Clean up state if completed
        if !request.next_step_needed {
            state_guard.remove(&session_id);
        }
        
        Ok(BiasedReasoningResponse {
            current_step: request.step_number,
            step_reasoning,
            bias_analysis: bias_check,
            corrected_reasoning,
            status,
            next_action,
            model_used: primary_model,
            verifier_used: verifier_model,
            overall_progress,
            reasoning_quality,
        })
    }
    
    fn build_context(&self, state: &BiasedReasoningState) -> String {
        let mut context = String::new();
        
        context.push_str(&format!("Original Query: {}\n\n", state.original_query));
        
        if !state.reasoning_history.is_empty() {
            context.push_str("Previous reasoning steps:\n");
            for step in &state.reasoning_history {
                context.push_str(&format!(
                    "Step {}: [Quality: {:.2}]\n{}\n",
                    step.step_number,
                    step.quality_score,
                    step.corrected_thought.as_ref().unwrap_or(&step.original_thought)
                ));
                
                if step.bias_check.has_bias {
                    context.push_str(&format!(
                        "   Biases detected: {:?}\n",
                        step.bias_check.bias_types
                    ));
                }
                context.push('\n');
            }
        }
        
        context
    }
    
    fn build_user_prompt(
        &self,
        request: &BiasedReasoningRequest,
        context: &str,
        state: &BiasedReasoningState,
    ) -> String {
        if request.step_number == 1 {
            format!(
                "{}Please provide step {} of your reasoning process.\n\n\
                Focus on establishing the foundation of your analysis.",
                context,
                request.step_number
            )
        } else {
            let step_type = if request.step_number <= 2 {
                "exploration and understanding"
            } else if request.step_number > request.total_steps - 2 {
                "synthesis and conclusion"
            } else {
                "deeper analysis"
            };
            
            format!(
                "{}Current step: {} of {}\n\
                User guidance: {}\n\n\
                Continue your reasoning with {} for this step.\n\
                Build on previous insights while adding new perspective.",
                context,
                request.step_number,
                request.total_steps,
                request.step_content,
                step_type
            )
        }
    }
    
    fn suggest_next_focus(&self, state: &BiasedReasoningState, bias_check: &BiasCheckResult) -> String {
        if bias_check.has_bias {
            "Address the identified biases and strengthen your reasoning".to_string()
        } else if state.reasoning_history.len() < 3 {
            "Explore different angles and gather more evidence".to_string()
        } else {
            "Begin synthesizing insights toward a conclusion".to_string()
        }
    }
    
    async fn check_step_for_bias(
        &self,
        thought: &str,
        original_query: &str,
        step_number: u32,
        verifier_client: &Arc<dyn LLMClient>,
        config: &BiasCheckConfig,
        verifier_model_name: &str,
    ) -> Result<BiasCheckResult> {
        let mut check_prompt = format!(
            "Analyze the following reasoning step for biases and errors:\n\n\
            Original Query: {}\n\
            Step {}: {}\n\n\
            Check for:\n",
            original_query, step_number, thought
        );
        
        if config.check_confirmation_bias {
            check_prompt.push_str("- Confirmation bias: Only considering evidence that supports initial assumptions\n");
        }
        if config.check_anchoring_bias {
            check_prompt.push_str("- Anchoring bias: Over-relying on first piece of information\n");
        }
        if config.check_availability_bias {
            check_prompt.push_str("- Availability bias: Overweighting easily recalled information\n");
        }
        if config.check_reasoning_errors {
            check_prompt.push_str("- Logical fallacies and reasoning errors\n");
        }
        
        check_prompt.push_str(
            "\nProvide analysis in this format:\n\
            HAS_BIAS: [Yes/No]\n\
            BIAS_TYPES: [List of detected biases]\n\
            SEVERITY: [Low/Medium/High/Critical]\n\
            EXPLANATION: [Brief explanation]\n\
            SUGGESTIONS: [List of improvements]"
        );
        
        let messages = vec![
            ChatMessage {
                role: Role::System,
                content: "You are a critical thinking expert who identifies biases and reasoning errors.".to_string(),
            },
            ChatMessage {
                role: Role::User,
                content: check_prompt,
            },
        ];
        
        let temperature = if verifier_model_name.starts_with("o4") {
            None
        } else {
            Some(0.3)
        };
        
        let response = verifier_client
            .complete(messages, temperature, Some(10000))
            .await
            .map_err(|e| {
                error!("Verifier model '{}' failed during bias check: {}", verifier_model_name, e);
                anyhow::anyhow!("Failed to check for bias: {}", e)
            })?;
        
        self.parse_bias_check_response(&response.content)
    }
    
    fn parse_bias_check_response(&self, response: &str) -> Result<BiasCheckResult> {
        let has_bias = response.contains("HAS_BIAS: Yes");
        
        let mut bias_types = Vec::new();
        if let Some(types_line) = response.lines().find(|l| l.starts_with("BIAS_TYPES:")) {
            let types_str = types_line.replace("BIAS_TYPES:", "").trim().to_lowercase();
            if types_str.contains("confirmation") {
                bias_types.push(BiasType::ConfirmationBias);
            }
            if types_str.contains("anchoring") {
                bias_types.push(BiasType::AnchoringBias);
            }
            if types_str.contains("availability") {
                bias_types.push(BiasType::AvailabilityBias);
            }
            if types_str.contains("reasoning") || types_str.contains("logical") {
                bias_types.push(BiasType::ReasoningError);
            }
            if types_str.contains("fallacy") {
                bias_types.push(BiasType::LogicalFallacy);
            }
            if types_str.contains("emotional") {
                bias_types.push(BiasType::EmotionalReasoning);
            }
            if types_str.contains("general") {
                bias_types.push(BiasType::OverGeneralization);
            }
        }
        
        let severity = if let Some(sev_line) = response.lines().find(|l| l.starts_with("SEVERITY:")) {
            let sev_str = sev_line.replace("SEVERITY:", "").trim().to_lowercase();
            match sev_str.as_str() {
                "low" => Some(BiasSeverity::Low),
                "medium" => Some(BiasSeverity::Medium),
                "high" => Some(BiasSeverity::High),
                "critical" => Some(BiasSeverity::Critical),
                _ => None,
            }
        } else {
            None
        };
        
        let explanation = response.lines()
            .find(|l| l.starts_with("EXPLANATION:"))
            .map(|l| l.replace("EXPLANATION:", "").trim().to_string())
            .unwrap_or_else(|| "No explanation provided".to_string());
        
        let mut suggestions = Vec::new();
        if let Some(sugg_line) = response.lines().find(|l| l.starts_with("SUGGESTIONS:")) {
            let sugg_text = response.lines()
                .skip_while(|l| !l.starts_with("SUGGESTIONS:"))
                .skip(1)
                .take_while(|l| !l.is_empty() && !l.starts_with("HAS_BIAS:"))
                .collect::<Vec<_>>()
                .join("\n");
            
            suggestions = sugg_text
                .lines()
                .filter(|l| l.trim().starts_with('-') || l.trim().starts_with('•'))
                .map(|l| l.trim_start_matches(&['-', '•', ' '][..]).trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
        
        Ok(BiasCheckResult {
            has_bias,
            bias_types,
            severity,
            explanation,
            suggestions,
        })
    }
    
    async fn generate_corrected_thought(
        &self,
        original_thought: &str,
        bias_check: &BiasCheckResult,
        verifier_client: &Arc<dyn LLMClient>,
    ) -> Result<String> {
        let correction_prompt = format!(
            "Rewrite the following reasoning step to address the identified biases:\n\n\
            Original thought: {}\n\n\
            Biases identified: {:?}\n\
            Explanation: {}\n\
            Suggestions: {}\n\n\
            Provide a corrected version that maintains the core insights while addressing these issues.",
            original_thought,
            bias_check.bias_types,
            bias_check.explanation,
            bias_check.suggestions.join("; ")
        );
        
        let messages = vec![
            ChatMessage {
                role: Role::System,
                content: "You are a reasoning assistant helping to improve thinking by addressing biases.".to_string(),
            },
            ChatMessage {
                role: Role::User,
                content: correction_prompt,
            },
        ];
        
        let verifier_model = verifier_client.get_model_name();
        let temperature = if verifier_model.starts_with("o4") {
            None
        } else {
            Some(0.5)
        };
        
        let response = verifier_client
            .complete(messages, temperature, Some(10000))
            .await
            .context("Failed to generate corrected thought")?;
        
        Ok(response.content)
    }
    
    fn calculate_step_quality(&self, bias_check: &BiasCheckResult) -> f32 {
        let mut quality = 1.0;
        
        if bias_check.has_bias {
            quality -= 0.1 * bias_check.bias_types.len() as f32;
            
            if let Some(ref severity) = bias_check.severity {
                match severity {
                    BiasSeverity::Low => quality -= 0.1,
                    BiasSeverity::Medium => quality -= 0.2,
                    BiasSeverity::High => quality -= 0.3,
                    BiasSeverity::Critical => quality -= 0.5,
                }
            }
        }
        
        quality.max(0.0)
    }
    
    fn get_client_for_model(&self, model: &str) -> Result<Arc<dyn LLMClient>> {
        // Determine if this is an OpenRouter model
        let is_openrouter = model.contains('/') || 
            model.starts_with("claude") || 
            model.starts_with("gemini") ||
            model.starts_with("meta") ||
            model.starts_with("mistral");
        
        if is_openrouter {
            if let Some((_, client)) = self.openrouter_clients.iter().find(|(m, _)| m == model) {
                return Ok(client.clone());
            }
            
            if let Some(api_key) = &self.config.openrouter_api_key {
                let client = OpenRouterClient::new(
                    api_key.clone(),
                    model.to_string(),
                    self.config.openrouter_base_url.clone(),
                )?;
                return Ok(Arc::new(client) as Arc<dyn LLMClient>);
            }
            
            anyhow::bail!("OpenRouter API key not configured for model: {}", model);
        } else {
            if let Some(client) = &self.openai_client {
                return Ok(client.clone());
            }
            
            anyhow::bail!("OpenAI API key not configured for model: {}", model);
        }
    }
}