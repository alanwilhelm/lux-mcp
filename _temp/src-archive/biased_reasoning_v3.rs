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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum StepType {
    Query,          // Initial question
    Reasoning,      // Primary model reasoning
    BiasAnalysis,   // Bias check result (VISIBLE)
    Correction,     // Corrected reasoning (VISIBLE)
    Guidance,       // User input/guidance
    Synthesis,      // Final compilation
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BiasedReasoningRequest {
    pub step_type: StepType,
    pub content: String,
    pub step_number: u32,
    
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub continue_to_next: bool,  // Auto-continue to next logical step
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BiasedReasoningResponse {
    pub step_type: StepType,
    pub step_number: u32,
    pub content: String,
    
    // Context-specific fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bias_analysis: Option<BiasCheckResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correction_details: Option<CorrectionDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_metadata: Option<ReasoningMetadata>,
    
    pub model_used: String,
    pub next_action: NextAction,
    pub session_status: SessionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiasCheckResult {
    pub has_bias: bool,
    pub bias_types: Vec<BiasType>,
    pub severity: Option<BiasSeverity>,
    pub explanation: String,
    pub suggestions: Vec<String>,
    pub confidence: f32,  // 0.0 to 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrectionDetails {
    pub original_text: String,
    pub corrected_text: String,
    pub changes_made: Vec<String>,
    pub improvement_score: f32,  // 0.0 to 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningMetadata {
    pub thinking_time_ms: u64,
    pub tokens_generated: Option<u32>,
    pub confidence_level: f32,
    pub reasoning_depth: String,  // "shallow", "moderate", "deep"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NextAction {
    BiasCheck,           // Next step should be bias analysis
    CorrectionNeeded,    // Bias found, correction recommended
    ContinueReasoning,   // Continue with next reasoning step
    AwaitingGuidance,    // Waiting for user input
    ReadyForSynthesis,   // Ready to compile final answer
    Complete,            // Process complete
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStatus {
    pub total_steps: u32,
    pub reasoning_steps: u32,
    pub bias_checks: u32,
    pub corrections_made: u32,
    pub overall_quality: f32,  // 0.0 to 1.0
    pub is_complete: bool,
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
    SelectiveEvidence,
    FalseEquivalence,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BiasSeverity {
    Low,
    Medium,
    High,
    Critical,
}

// Session state for maintaining context
struct BiasedReasoningState {
    original_query: String,
    steps: Vec<ProcessedStep>,
    conversation_history: Vec<ChatMessage>,
    bias_count: HashMap<BiasType, u32>,
    total_corrections: u32,
    current_reasoning_chain: Vec<String>,  // Current chain of reasoning
}

#[derive(Debug, Clone)]
struct ProcessedStep {
    step_number: u32,
    step_type: StepType,
    content: String,
    bias_analysis: Option<BiasCheckResult>,
    correction: Option<CorrectionDetails>,
    metadata: Option<ReasoningMetadata>,
}

pub struct BiasedReasoningTool {
    session_manager: Arc<SessionManager>,
    openai_client: Option<Arc<dyn LLMClient>>,
    openrouter_clients: Vec<(String, Arc<dyn LLMClient>)>,
    model_resolver: ModelResolver,
    config: LLMConfig,
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
        
        // ALWAYS use configured defaults
        let primary_model = self.model_resolver.resolve(&self.config.default_reasoning_model);
        let verifier_model = self.model_resolver.resolve(&self.config.default_bias_checker_model);
        
        info!("Processing step {} (type: {:?}) with models: primary='{}', verifier='{}'", 
              request.step_number, request.step_type, primary_model, verifier_model);
        
        // Get or create state
        let mut state_guard = self.state_store.lock();
        let state = state_guard.entry(session_id.clone()).or_insert_with(|| {
            BiasedReasoningState {
                original_query: String::new(),
                steps: Vec::new(),
                conversation_history: vec![
                    ChatMessage {
                        role: Role::System,
                        content: "You are a reasoning assistant. Think through problems step-by-step, showing your thinking clearly.".to_string(),
                    }
                ],
                bias_count: HashMap::new(),
                total_corrections: 0,
                current_reasoning_chain: Vec::new(),
            }
        });
        
        // Process based on step type
        let response = match request.step_type {
            StepType::Query => self.handle_query(state, request, &session_id, &monitor).await?,
            StepType::Reasoning => self.handle_reasoning(state, request, &primary_model, &monitor).await?,
            StepType::BiasAnalysis => self.handle_bias_analysis(state, request, &verifier_model).await?,
            StepType::Correction => self.handle_correction(state, request, &verifier_model).await?,
            StepType::Guidance => self.handle_guidance(state, request).await?,
            StepType::Synthesis => self.handle_synthesis(state, request, &primary_model).await?,
        };
        
        // Store the processed step
        state.steps.push(ProcessedStep {
            step_number: request.step_number,
            step_type: request.step_type.clone(),
            content: response.content.clone(),
            bias_analysis: response.bias_analysis.clone(),
            correction: response.correction_details.clone(),
            metadata: response.reasoning_metadata.clone(),
        });
        
        // Clean up if complete
        if response.session_status.is_complete {
            state_guard.remove(&session_id);
        }
        
        Ok(response)
    }
    
    async fn handle_query(
        &self,
        state: &mut BiasedReasoningState,
        request: BiasedReasoningRequest,
        session_id: &str,
        monitor: &Arc<Mutex<crate::monitoring::MetacognitiveMonitor>>,
    ) -> Result<BiasedReasoningResponse> {
        // Initialize new session
        state.original_query = request.content.clone();
        state.steps.clear();
        state.bias_count.clear();
        state.total_corrections = 0;
        state.current_reasoning_chain.clear();
        
        // Reset monitor
        {
            let mut monitor_guard = monitor.lock();
            monitor_guard.reset_session();
        }
        
        // Add query to conversation
        state.conversation_history.push(ChatMessage {
            role: Role::User,
            content: format!("Query: {}\n\nPlease begin your step-by-step reasoning.", request.content),
        });
        
        Ok(BiasedReasoningResponse {
            step_type: StepType::Query,
            step_number: request.step_number,
            content: format!("Query received: {}", request.content),
            bias_analysis: None,
            correction_details: None,
            reasoning_metadata: None,
            model_used: "none".to_string(),
            next_action: NextAction::ContinueReasoning,
            session_status: self.calculate_session_status(state),
        })
    }
    
    async fn handle_reasoning(
        &self,
        state: &mut BiasedReasoningState,
        request: BiasedReasoningRequest,
        model: &str,
        monitor: &Arc<Mutex<crate::monitoring::MetacognitiveMonitor>>,
    ) -> Result<BiasedReasoningResponse> {
        let client = self.get_client_for_model(model)?;
        
        // Build context from previous steps
        let context = self.build_reasoning_context(state, &request);
        
        // Add context to conversation if this is a guided step
        if !request.content.is_empty() && request.content != "continue" {
            state.conversation_history.push(ChatMessage {
                role: Role::User,
                content: format!("Guidance: {}", request.content),
            });
        }
        
        // Generate reasoning
        info!("🧠 Generating reasoning step with {}", model);
        let start = Instant::now();
        
        let messages = if state.conversation_history.len() > 1 {
            state.conversation_history.clone()
        } else {
            vec![
                ChatMessage {
                    role: Role::System,
                    content: "You are a reasoning assistant. Think through problems step-by-step.".to_string(),
                },
                ChatMessage {
                    role: Role::User,
                    content: context,
                },
            ]
        };
        
        let temperature = request.temperature.unwrap_or(0.7);
        let response = client
            .complete(messages, Some(temperature), Some(10000))
            .await?;
        
        let duration = start.elapsed();
        info!("✅ {} completed reasoning in {:?}", model, duration);
        
        // Add to conversation history
        state.conversation_history.push(ChatMessage {
            role: Role::Assistant,
            content: response.content.clone(),
        });
        
        // Add to reasoning chain
        state.current_reasoning_chain.push(response.content.clone());
        
        // Update monitor
        {
            let mut monitor_guard = monitor.lock();
            monitor_guard.process_thought(&response.content, None);
        }
        
        let metadata = ReasoningMetadata {
            thinking_time_ms: duration.as_millis() as u64,
            tokens_generated: response.usage.as_ref().map(|u| u.completion_tokens),
            confidence_level: 0.8,  // Could be calculated based on content
            reasoning_depth: self.assess_reasoning_depth(&response.content),
        };
        
        Ok(BiasedReasoningResponse {
            step_type: StepType::Reasoning,
            step_number: request.step_number,
            content: response.content,
            bias_analysis: None,
            correction_details: None,
            reasoning_metadata: Some(metadata),
            model_used: model.to_string(),
            next_action: NextAction::BiasCheck,
            session_status: self.calculate_session_status(state),
        })
    }
    
    async fn handle_bias_analysis(
        &self,
        state: &mut BiasedReasoningState,
        request: BiasedReasoningRequest,
        model: &str,
    ) -> Result<BiasedReasoningResponse> {
        // Get the last reasoning step to analyze
        let last_reasoning = state.current_reasoning_chain.last()
            .ok_or_else(|| anyhow::anyhow!("No reasoning step to analyze"))?;
        
        let client = self.get_client_for_model(model)?;
        
        info!("🔍 Analyzing reasoning for bias with {}", model);
        let start = Instant::now();
        
        let bias_check = self.check_for_bias(
            last_reasoning,
            &state.original_query,
            state.current_reasoning_chain.len() as u32,
            &client,
            model,
        ).await?;
        
        let duration = start.elapsed();
        info!("✅ Bias analysis completed in {:?}", duration);
        
        // Update bias counts
        for bias_type in &bias_check.bias_types {
            *state.bias_count.entry(bias_type.clone()).or_insert(0) += 1;
        }
        
        let next_action = if bias_check.has_bias && 
            matches!(bias_check.severity, Some(BiasSeverity::High) | Some(BiasSeverity::Critical)) {
            NextAction::CorrectionNeeded
        } else {
            NextAction::AwaitingGuidance
        };
        
        Ok(BiasedReasoningResponse {
            step_type: StepType::BiasAnalysis,
            step_number: request.step_number,
            content: self.format_bias_analysis(&bias_check),
            bias_analysis: Some(bias_check),
            correction_details: None,
            reasoning_metadata: Some(ReasoningMetadata {
                thinking_time_ms: duration.as_millis() as u64,
                tokens_generated: None,
                confidence_level: 0.9,
                reasoning_depth: "analytical".to_string(),
            }),
            model_used: model.to_string(),
            next_action,
            session_status: self.calculate_session_status(state),
        })
    }
    
    async fn handle_correction(
        &self,
        state: &mut BiasedReasoningState,
        request: BiasedReasoningRequest,
        model: &str,
    ) -> Result<BiasedReasoningResponse> {
        // Get the last bias analysis
        let last_bias = state.steps.iter().rev()
            .find(|s| matches!(s.step_type, StepType::BiasAnalysis))
            .and_then(|s| s.bias_analysis.as_ref())
            .ok_or_else(|| anyhow::anyhow!("No bias analysis found for correction"))?;
        
        // Get the reasoning that needs correction
        let reasoning_to_correct = state.current_reasoning_chain.last()
            .ok_or_else(|| anyhow::anyhow!("No reasoning to correct"))?;
        
        let client = self.get_client_for_model(model)?;
        
        info!("✏️ Generating correction with {}", model);
        let corrected = self.generate_correction(
            reasoning_to_correct,
            last_bias,
            &client,
        ).await?;
        
        // Replace the last reasoning in the chain
        if let Some(last) = state.current_reasoning_chain.last_mut() {
            *last = corrected.corrected_text.clone();
        }
        
        state.total_corrections += 1;
        
        Ok(BiasedReasoningResponse {
            step_type: StepType::Correction,
            step_number: request.step_number,
            content: format!("Corrected reasoning:\n\n{}", corrected.corrected_text),
            bias_analysis: None,
            correction_details: Some(corrected),
            reasoning_metadata: None,
            model_used: model.to_string(),
            next_action: NextAction::AwaitingGuidance,
            session_status: self.calculate_session_status(state),
        })
    }
    
    async fn handle_guidance(
        &self,
        state: &mut BiasedReasoningState,
        request: BiasedReasoningRequest,
    ) -> Result<BiasedReasoningResponse> {
        // Process user guidance
        info!("📝 Processing user guidance");
        
        // Determine next action based on guidance
        let next_action = if request.content.to_lowercase().contains("synthesize") ||
            request.content.to_lowercase().contains("conclude") ||
            request.content.to_lowercase().contains("final") {
            NextAction::ReadyForSynthesis
        } else {
            NextAction::ContinueReasoning
        };
        
        Ok(BiasedReasoningResponse {
            step_type: StepType::Guidance,
            step_number: request.step_number,
            content: format!("Guidance received: {}", request.content),
            bias_analysis: None,
            correction_details: None,
            reasoning_metadata: None,
            model_used: "none".to_string(),
            next_action,
            session_status: self.calculate_session_status(state),
        })
    }
    
    async fn handle_synthesis(
        &self,
        state: &mut BiasedReasoningState,
        request: BiasedReasoningRequest,
        model: &str,
    ) -> Result<BiasedReasoningResponse> {
        let client = self.get_client_for_model(model)?;
        
        info!("🎯 Generating final synthesis with {}", model);
        
        let synthesis_prompt = self.build_synthesis_prompt(state);
        let messages = vec![
            ChatMessage {
                role: Role::System,
                content: "You are synthesizing a complete answer based on the reasoning process.".to_string(),
            },
            ChatMessage {
                role: Role::User,
                content: synthesis_prompt,
            },
        ];
        
        let response = client
            .complete(messages, Some(0.7), Some(10000))
            .await?;
        
        let mut status = self.calculate_session_status(state);
        status.is_complete = true;
        
        Ok(BiasedReasoningResponse {
            step_type: StepType::Synthesis,
            step_number: request.step_number,
            content: response.content,
            bias_analysis: None,
            correction_details: None,
            reasoning_metadata: None,
            model_used: model.to_string(),
            next_action: NextAction::Complete,
            session_status: status,
        })
    }
    
    fn build_reasoning_context(&self, state: &BiasedReasoningState, request: &BiasedReasoningRequest) -> String {
        let mut context = format!("Original Query: {}\n\n", state.original_query);
        
        if !state.current_reasoning_chain.is_empty() {
            context.push_str("Previous reasoning:\n");
            for (i, reasoning) in state.current_reasoning_chain.iter().enumerate() {
                context.push_str(&format!("Step {}: {}\n\n", i + 1, reasoning));
            }
        }
        
        if !request.content.is_empty() && request.content != "continue" {
            context.push_str(&format!("User guidance: {}\n\n", request.content));
        }
        
        context.push_str("Continue your reasoning for the next step:");
        context
    }
    
    async fn check_for_bias(
        &self,
        thought: &str,
        original_query: &str,
        step_number: u32,
        client: &Arc<dyn LLMClient>,
        model_name: &str,
    ) -> Result<BiasCheckResult> {
        let check_prompt = format!(
            "Analyze the following reasoning step for biases and errors:\n\n\
            Original Query: {}\n\
            Step {}: {}\n\n\
            Check for:\n\
            - Confirmation bias: Only considering evidence that supports initial assumptions\n\
            - Anchoring bias: Over-relying on first piece of information\n\
            - Availability bias: Overweighting easily recalled information\n\
            - Logical fallacies and reasoning errors\n\
            - Selective evidence: Cherry-picking data\n\
            - Over-generalization: Drawing broad conclusions from limited data\n\
            - Emotional reasoning: Letting feelings override logic\n\n\
            Provide analysis in this format:\n\
            HAS_BIAS: [Yes/No]\n\
            BIAS_TYPES: [List of detected biases]\n\
            SEVERITY: [Low/Medium/High/Critical]\n\
            CONFIDENCE: [0.0-1.0]\n\
            EXPLANATION: [Detailed explanation]\n\
            SUGGESTIONS: [List of improvements]",
            original_query, step_number, thought
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
        
        let temperature = if model_name.starts_with("o4") {
            None
        } else {
            Some(0.3)
        };
        
        let response = client
            .complete(messages, temperature, Some(10000))
            .await?;
        
        self.parse_bias_response(&response.content)
    }
    
    fn parse_bias_response(&self, response: &str) -> Result<BiasCheckResult> {
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
            if types_str.contains("general") || types_str.contains("over") {
                bias_types.push(BiasType::OverGeneralization);
            }
            if types_str.contains("selective") {
                bias_types.push(BiasType::SelectiveEvidence);
            }
        }
        
        let severity = response.lines()
            .find(|l| l.starts_with("SEVERITY:"))
            .and_then(|l| {
                let sev = l.replace("SEVERITY:", "").trim().to_lowercase();
                match sev.as_str() {
                    "low" => Some(BiasSeverity::Low),
                    "medium" => Some(BiasSeverity::Medium),
                    "high" => Some(BiasSeverity::High),
                    "critical" => Some(BiasSeverity::Critical),
                    _ => None,
                }
            });
        
        let confidence = response.lines()
            .find(|l| l.starts_with("CONFIDENCE:"))
            .and_then(|l| l.replace("CONFIDENCE:", "").trim().parse::<f32>().ok())
            .unwrap_or(0.8);
        
        let explanation = response.lines()
            .find(|l| l.starts_with("EXPLANATION:"))
            .map(|l| l.replace("EXPLANATION:", "").trim().to_string())
            .unwrap_or_else(|| "No explanation provided".to_string());
        
        let mut suggestions = Vec::new();
        let mut in_suggestions = false;
        for line in response.lines() {
            if line.starts_with("SUGGESTIONS:") {
                in_suggestions = true;
                continue;
            }
            if in_suggestions && (line.trim().starts_with('-') || line.trim().starts_with('•')) {
                suggestions.push(line.trim_start_matches(&['-', '•', ' '][..]).trim().to_string());
            }
        }
        
        Ok(BiasCheckResult {
            has_bias,
            bias_types,
            severity,
            explanation,
            suggestions,
            confidence,
        })
    }
    
    async fn generate_correction(
        &self,
        original: &str,
        bias_check: &BiasCheckResult,
        client: &Arc<dyn LLMClient>,
    ) -> Result<CorrectionDetails> {
        let prompt = format!(
            "Rewrite the following reasoning to address identified biases:\n\n\
            Original: {}\n\n\
            Biases: {:?}\n\
            Explanation: {}\n\
            Suggestions: {}\n\n\
            Provide a corrected version that:\n\
            1. Maintains core insights\n\
            2. Addresses the identified biases\n\
            3. Incorporates the suggestions\n\
            4. Remains clear and logical",
            original,
            bias_check.bias_types,
            bias_check.explanation,
            bias_check.suggestions.join("; ")
        );
        
        let messages = vec![
            ChatMessage {
                role: Role::System,
                content: "You are helping improve reasoning by addressing biases.".to_string(),
            },
            ChatMessage {
                role: Role::User,
                content: prompt,
            },
        ];
        
        let model_name = client.get_model_name();
        let temperature = if model_name.starts_with("o4") {
            None
        } else {
            Some(0.5)
        };
        
        let response = client
            .complete(messages, temperature, Some(10000))
            .await?;
        
        let changes_made = bias_check.bias_types.iter()
            .map(|bt| format!("Addressed {:?}", bt))
            .collect();
        
        Ok(CorrectionDetails {
            original_text: original.to_string(),
            corrected_text: response.content,
            changes_made,
            improvement_score: 0.8, // Could calculate based on bias severity
        })
    }
    
    fn format_bias_analysis(&self, bias_check: &BiasCheckResult) -> String {
        let mut output = String::new();
        
        if bias_check.has_bias {
            output.push_str(&format!("⚠️ Biases Detected (Confidence: {:.1})\n\n", bias_check.confidence));
            output.push_str(&format!("Types: {}\n", 
                bias_check.bias_types.iter()
                    .map(|bt| format!("{:?}", bt))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
            
            if let Some(ref sev) = bias_check.severity {
                output.push_str(&format!("Severity: {:?}\n", sev));
            }
            
            output.push_str(&format!("\nExplanation:\n{}\n", bias_check.explanation));
            
            if !bias_check.suggestions.is_empty() {
                output.push_str("\nSuggestions for improvement:\n");
                for suggestion in &bias_check.suggestions {
                    output.push_str(&format!("• {}\n", suggestion));
                }
            }
        } else {
            output.push_str("✅ No significant biases detected\n\n");
            output.push_str(&format!("Confidence: {:.1}\n", bias_check.confidence));
            output.push_str(&format!("Analysis: {}", bias_check.explanation));
        }
        
        output
    }
    
    fn build_synthesis_prompt(&self, state: &BiasedReasoningState) -> String {
        let mut prompt = format!("Original Query: {}\n\n", state.original_query);
        
        prompt.push_str("Reasoning Process:\n");
        for (i, reasoning) in state.current_reasoning_chain.iter().enumerate() {
            prompt.push_str(&format!("Step {}: {}\n\n", i + 1, reasoning));
        }
        
        prompt.push_str("\nBias Summary:\n");
        if state.bias_count.is_empty() {
            prompt.push_str("No significant biases detected throughout the reasoning process.\n");
        } else {
            for (bias_type, count) in &state.bias_count {
                prompt.push_str(&format!("- {:?}: {} occurrences\n", bias_type, count));
            }
        }
        
        prompt.push_str(&format!("\nTotal corrections made: {}\n", state.total_corrections));
        
        prompt.push_str("\nBased on this reasoning process, provide a comprehensive final answer to the original query.");
        
        prompt
    }
    
    fn assess_reasoning_depth(&self, content: &str) -> String {
        let word_count = content.split_whitespace().count();
        let has_examples = content.contains("example") || content.contains("instance");
        let has_analysis = content.contains("because") || content.contains("therefore") || content.contains("thus");
        
        if word_count > 200 && has_examples && has_analysis {
            "deep".to_string()
        } else if word_count > 100 && has_analysis {
            "moderate".to_string()
        } else {
            "shallow".to_string()
        }
    }
    
    fn calculate_session_status(&self, state: &BiasedReasoningState) -> SessionStatus {
        let reasoning_steps = state.steps.iter()
            .filter(|s| matches!(s.step_type, StepType::Reasoning))
            .count() as u32;
        
        let bias_checks = state.steps.iter()
            .filter(|s| matches!(s.step_type, StepType::BiasAnalysis))
            .count() as u32;
        
        let overall_quality = if reasoning_steps == 0 {
            1.0
        } else {
            let bias_penalty = state.bias_count.values().sum::<u32>() as f32 * 0.1;
            (1.0 - bias_penalty / reasoning_steps as f32).max(0.3)
        };
        
        SessionStatus {
            total_steps: state.steps.len() as u32,
            reasoning_steps,
            bias_checks,
            corrections_made: state.total_corrections,
            overall_quality,
            is_complete: false,
        }
    }
    
    fn get_client_for_model(&self, model: &str) -> Result<Arc<dyn LLMClient>> {
        let is_openrouter = model.contains('/') || 
            model.starts_with("claude") || 
            model.starts_with("gemini");
        
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