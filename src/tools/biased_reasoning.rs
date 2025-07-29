use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, info, error};
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

// Step-by-step request
#[derive(Debug, Serialize, Deserialize)]
pub struct BiasedReasoningRequest {
    pub query: String,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub new_session: Option<bool>,
    #[serde(default)]
    pub primary_model: Option<String>,
    #[serde(default)]
    pub verifier_model: Option<String>,
    #[serde(default = "default_max_steps")]
    pub max_analysis_rounds: u32,
}

fn default_max_steps() -> u32 { 3 }

// Step-by-step response
#[derive(Debug, Serialize, Deserialize)]
pub struct BiasedReasoningResponse {
    pub session_id: String,
    pub step_type: StepType,
    pub step_number: u32,
    pub content: String,
    pub model_used: String,
    pub next_action: NextAction,
    pub session_status: SessionStatus,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bias_analysis: Option<BiasCheckResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correction_details: Option<CorrectionDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_metadata: Option<ReasoningMetadata>,
}

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessLogEntry {
    pub action_type: ProcessActionType,
    pub step_number: u32,
    pub timestamp: String,
    pub model_used: String,
    pub content: String,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ProcessActionType {
    PrimaryReasoning,
    BiasChecking,
    CorrectionGeneration,
    QualityAssessment,
    FinalAnswerGeneration,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VerifiedReasoningStep {
    pub step_number: u32,
    pub primary_thought: String,
    pub bias_check: BiasCheckResult,
    pub corrected_thought: Option<String>,
    pub step_quality: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BiasCheckResult {
    pub has_bias: bool,
    pub bias_types: Vec<BiasType>,
    pub severity: Severity,
    pub explanation: String,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum BiasType {
    ConfirmationBias,
    AnchoringBias,
    AvailabilityBias,
    ReasoningError,
    OverGeneralization,
    FalseEquivalence,
    CircularReasoning,
    HastyConclusion,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    None,
    Low,
    Medium,
    High,
    Critical,
}

// Session state for step-by-step processing
#[derive(Clone)]
struct SessionState {
    query: String,
    step_count: u32,
    last_step_type: StepType,
    reasoning_steps: Vec<VerifiedReasoningStep>,
    detailed_process_log: Vec<ProcessLogEntry>,
    primary_conversation: Vec<ChatMessage>,
    bias_counts: HashMap<BiasType, u32>,
    final_answer: Option<String>,
}

pub struct BiasedReasoningTool {
    session_manager: Arc<SessionManager>,
    openai_client: Option<Arc<dyn LLMClient>>,
    openrouter_clients: Vec<(String, Arc<dyn LLMClient>)>,
    model_resolver: ModelResolver,
    config: LLMConfig,
    sessions: Arc<Mutex<HashMap<String, SessionState>>>,
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
            sessions: Arc::new(Mutex::new(HashMap::new())),
        })
    }
    
    // New step-by-step API with proper async handling
    pub async fn process_step(&self, request: BiasedReasoningRequest) -> Result<BiasedReasoningResponse> {
        use chrono::Utc;
        use sha2::{Sha256, Digest};
        
        // Generate or resolve session ID
        let session_id = if request.new_session.unwrap_or(false) {
            // Force new session
            format!("bias_{}", Utc::now().timestamp_millis())
        } else if let Some(provided_id) = request.session_id {
            // Use provided session ID
            provided_id
        } else {
            // Generate deterministic ID from query
            let mut hasher = Sha256::new();
            hasher.update(request.query.as_bytes());
            format!("bias_{}", hex::encode(&hasher.finalize()[..8]))
        };
        
        // ALWAYS use configured defaults
        let primary_model = self.model_resolver.resolve(&self.config.default_reasoning_model);
        let verifier_model = self.model_resolver.resolve(&self.config.default_bias_checker_model);
        
        // Initialize session if needed and get step info
        let (step_type, step_count, is_new_session) = {
            let mut sessions = self.sessions.lock();
            
            if !sessions.contains_key(&session_id) {
                sessions.insert(session_id.clone(), SessionState {
                    query: request.query.clone(),
                    step_count: 1,
                    last_step_type: StepType::Query,
                    reasoning_steps: Vec::new(),
                    detailed_process_log: Vec::new(),
                    primary_conversation: vec![
                        ChatMessage {
                            role: Role::System,
                            content: "You are a reasoning assistant. Think through problems step-by-step, showing your thinking clearly.".to_string(),
                        },
                        ChatMessage {
                            role: Role::User,
                            content: format!("Query: {}\n\nPlease reason through this step-by-step.", request.query),
                        },
                    ],
                    bias_counts: HashMap::new(),
                    final_answer: None,
                });
                (StepType::Query, 1, true)
            } else {
                let session = sessions.get_mut(&session_id).unwrap();
                session.step_count += 1;
                let step_count = session.step_count;
                
                // Determine next step type based on last step
                let step_type = match session.last_step_type {
                    StepType::Query => StepType::Reasoning,
                    StepType::Reasoning => StepType::BiasAnalysis,
                    StepType::BiasAnalysis => StepType::Reasoning,
                    _ => StepType::Reasoning,
                };
                
                // Update last step type
                session.last_step_type = step_type.clone();
                
                (step_type, step_count, false)
            }
        };
        
        // Process based on step type
        match step_type {
            StepType::Query => {
                Ok(BiasedReasoningResponse {
                    session_id: session_id.clone(),
                    step_type: StepType::Query,
                    step_number: step_count,
                    content: format!("Query received: {}\n\nStarting reasoning process...", request.query),
                    model_used: "none".to_string(),
                    next_action: NextAction::ContinueReasoning,
                    session_status: SessionStatus {
                        total_steps: 1,
                        reasoning_steps: 0,
                        bias_checks: 0,
                        corrections_made: 0,
                        overall_quality: 1.0,
                        is_complete: false,
                    },
                    bias_analysis: None,
                    correction_details: None,
                    reasoning_metadata: None,
                })
            },
            
            StepType::Reasoning => {
                self.handle_reasoning_step(session_id, step_count, primary_model).await
            },
            
            StepType::BiasAnalysis => {
                self.handle_bias_analysis_step(session_id, step_count, verifier_model, request.max_analysis_rounds).await
            },
            
            _ => {
                Err(anyhow::anyhow!("Step type not yet implemented: {:?}", step_type))
            }
        }
    }
    
    async fn handle_reasoning_step(
        &self,
        session_id: String,
        step_count: u32,
        primary_model: String,
    ) -> Result<BiasedReasoningResponse> {
        use chrono::Utc;
        
        // Get conversation from session
        let conversation = {
            let sessions = self.sessions.lock();
            sessions.get(&session_id)
                .map(|s| s.primary_conversation.clone())
                .unwrap_or_default()
        };
        
        let primary_client = self.get_client_for_model(&primary_model)?;
        
        info!("🧠 Generating reasoning step with {}", primary_model);
        let start = Instant::now();
        
        let primary_response = primary_client
            .complete(conversation, Some(0.7), Some(10000))
            .await
            .context("Failed to get reasoning step")?;
        
        let duration = start.elapsed();
        info!("✅ {} completed reasoning in {:?}", primary_model, duration);
        
        // Update session with new reasoning
        let session_status = {
            let mut sessions = self.sessions.lock();
            if let Some(session) = sessions.get_mut(&session_id) {
                // Add to conversation
                session.primary_conversation.push(ChatMessage {
                    role: Role::Assistant,
                    content: primary_response.content.clone(),
                });
                
                // Log the step
                session.detailed_process_log.push(ProcessLogEntry {
                    action_type: ProcessActionType::PrimaryReasoning,
                    step_number: step_count,
                    timestamp: Utc::now().to_rfc3339(),
                    model_used: primary_model.clone(),
                    content: format!("Generated reasoning step:\n{}", primary_response.content),
                    duration_ms: Some(duration.as_millis() as u64),
                });
                
                SessionStatus {
                    total_steps: step_count,
                    reasoning_steps: session.reasoning_steps.len() as u32 + 1,
                    bias_checks: session.reasoning_steps.iter().filter(|s| s.bias_check.has_bias).count() as u32,
                    corrections_made: session.reasoning_steps.iter().filter(|s| s.corrected_thought.is_some()).count() as u32,
                    overall_quality: 0.8,
                    is_complete: false,
                }
            } else {
                SessionStatus {
                    total_steps: step_count,
                    reasoning_steps: 1,
                    bias_checks: 0,
                    corrections_made: 0,
                    overall_quality: 0.8,
                    is_complete: false,
                }
            }
        };
        
        Ok(BiasedReasoningResponse {
            session_id: session_id.clone(),
            step_type: StepType::Reasoning,
            step_number: step_count,
            content: primary_response.content,
            model_used: primary_model,
            next_action: NextAction::BiasCheck,
            session_status,
            bias_analysis: None,
            correction_details: None,
            reasoning_metadata: Some(ReasoningMetadata {
                thinking_time_ms: duration.as_millis() as u64,
                tokens_generated: primary_response.usage.as_ref().map(|u| u.completion_tokens),
                confidence_level: 0.8,
                reasoning_depth: "moderate".to_string(),
            }),
        })
    }
    
    async fn handle_bias_analysis_step(
        &self,
        session_id: String,
        step_count: u32,
        verifier_model: String,
        max_rounds: u32,
    ) -> Result<BiasedReasoningResponse> {
        use chrono::Utc;
        
        // Get the last reasoning step and query
        let (last_thought, query) = {
            let sessions = self.sessions.lock();
            if let Some(session) = sessions.get(&session_id) {
                let last_thought = session.primary_conversation.last()
                    .filter(|m| m.role == Role::Assistant)
                    .map(|m| m.content.clone())
                    .unwrap_or_default();
                (last_thought, session.query.clone())
            } else {
                return Err(anyhow::anyhow!("Session not found"));
            }
        };
        
        let verifier_client = self.get_client_for_model(&verifier_model)?;
        
        info!("🔍 Checking for bias with {}", verifier_model);
        let start = Instant::now();
        
        let bias_check = self.check_step_for_bias(
            &last_thought,
            &query,
            step_count,
            &verifier_client,
            &BiasCheckConfig::default(),
            &verifier_model,
        ).await?;
        
        let duration = start.elapsed();
        info!("✅ Bias check completed in {:?}", duration);
        
        // Update session and determine next action
        let (next_action, session_status) = {
            let mut sessions = self.sessions.lock();
            if let Some(session) = sessions.get_mut(&session_id) {
                // Track bias types
                for bias_type in &bias_check.bias_types {
                    *session.bias_counts.entry(bias_type.clone()).or_insert(0) += 1;
                }
                
                // Store the verified step
                session.reasoning_steps.push(VerifiedReasoningStep {
                    step_number: step_count - 1,
                    primary_thought: last_thought.clone(),
                    bias_check: bias_check.clone(),
                    corrected_thought: None,
                    step_quality: match bias_check.severity {
                        Severity::None => 1.0,
                        Severity::Low => 0.8,
                        Severity::Medium => 0.6,
                        Severity::High => 0.4,
                        Severity::Critical => 0.2,
                    },
                });
                
                // Log the bias check
                session.detailed_process_log.push(ProcessLogEntry {
                    action_type: ProcessActionType::BiasChecking,
                    step_number: step_count,
                    timestamp: Utc::now().to_rfc3339(),
                    model_used: verifier_model.clone(),
                    content: format!(
                        "Bias check results:\n- Has bias: {}\n- Bias types: {:?}\n- Severity: {:?}",
                        bias_check.has_bias,
                        bias_check.bias_types,
                        bias_check.severity
                    ),
                    duration_ms: Some(duration.as_millis() as u64),
                });
                
                // Determine next action
                let next_action = if session.reasoning_steps.len() >= max_rounds as usize {
                    NextAction::ReadyForSynthesis
                } else {
                    NextAction::ContinueReasoning
                };
                
                // Add continuation prompt if needed
                if next_action == NextAction::ContinueReasoning {
                    session.primary_conversation.push(ChatMessage {
                        role: Role::User,
                        content: "Continue your reasoning to the next step.".to_string(),
                    });
                }
                
                let status = SessionStatus {
                    total_steps: step_count,
                    reasoning_steps: session.reasoning_steps.len() as u32,
                    bias_checks: session.reasoning_steps.iter().filter(|s| s.bias_check.has_bias).count() as u32,
                    corrections_made: session.reasoning_steps.iter().filter(|s| s.corrected_thought.is_some()).count() as u32,
                    overall_quality: 0.8,
                    is_complete: next_action == NextAction::Complete,
                };
                
                (next_action, status)
            } else {
                return Err(anyhow::anyhow!("Session not found"));
            }
        };
        
        Ok(BiasedReasoningResponse {
            session_id: session_id.clone(),
            step_type: StepType::BiasAnalysis,
            step_number: step_count,
            content: format!(
                "Bias Analysis:\n\n{}\n{}\n{}",
                if bias_check.has_bias { "⚠️ Biases detected!" } else { "✅ No significant biases detected." },
                if !bias_check.bias_types.is_empty() {
                    format!("\nBias types: {:?}", bias_check.bias_types)
                } else {
                    String::new()
                },
                if !bias_check.suggestions.is_empty() {
                    format!("\nSuggestions: {}", bias_check.suggestions.join(", "))
                } else {
                    String::new()
                }
            ),
            model_used: verifier_model,
            next_action,
            session_status,
            bias_analysis: Some(bias_check),
            correction_details: None,
            reasoning_metadata: None,
        })
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
            Check for the following:\n",
            original_query, step_number, thought
        );
        
        if config.check_confirmation_bias {
            check_prompt.push_str("- Confirmation bias: Is the reasoning cherry-picking evidence?\n");
        }
        if config.check_anchoring_bias {
            check_prompt.push_str("- Anchoring bias: Is it overly influenced by initial information?\n");
        }
        if config.check_availability_bias {
            check_prompt.push_str("- Availability bias: Is it overweighting easily recalled examples?\n");
        }
        if config.check_reasoning_errors {
            check_prompt.push_str("- Reasoning errors: Are there logical fallacies or poor inferences?\n");
        }
        
        check_prompt.push_str("\nProvide a structured analysis with:\n\
            1. Whether bias is present (yes/no)\n\
            2. Types of bias found\n\
            3. Severity (none/low/medium/high/critical)\n\
            4. Brief explanation\n\
            5. Suggestions for improvement");
        
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
        
        let max_tokens = 10000;
        
        let temperature = if verifier_model_name.starts_with("o4") {
            None
        } else {
            Some(0.3)
        };
        
        let response = verifier_client
            .complete(messages, temperature, Some(max_tokens))
            .await
            .map_err(|e| {
                error!("Verifier model '{}' failed during bias check: {}", verifier_model_name, e);
                anyhow::anyhow!("Failed to check for bias with model '{}': {}", verifier_model_name, e)
            })?;
        
        self.parse_bias_check_response(&response.content)
    }
    
    fn parse_bias_check_response(&self, content: &str) -> Result<BiasCheckResult> {
        let content_lower = content.to_lowercase();
        
        let has_bias = content_lower.contains("yes") || 
                      content_lower.contains("bias is present") ||
                      content_lower.contains("found bias");
        
        let mut bias_types = Vec::new();
        if content_lower.contains("confirmation bias") {
            bias_types.push(BiasType::ConfirmationBias);
        }
        if content_lower.contains("anchoring") {
            bias_types.push(BiasType::AnchoringBias);
        }
        if content_lower.contains("availability") {
            bias_types.push(BiasType::AvailabilityBias);
        }
        if content_lower.contains("reasoning error") || content_lower.contains("logical fallacy") {
            bias_types.push(BiasType::ReasoningError);
        }
        if content_lower.contains("overgeneralization") {
            bias_types.push(BiasType::OverGeneralization);
        }
        if content_lower.contains("circular") {
            bias_types.push(BiasType::CircularReasoning);
        }
        
        let severity = if content_lower.contains("critical") {
            Severity::Critical
        } else if content_lower.contains("high") {
            Severity::High
        } else if content_lower.contains("medium") {
            Severity::Medium
        } else if content_lower.contains("low") {
            Severity::Low
        } else {
            Severity::None
        };
        
        let suggestions = if has_bias {
            vec!["Consider alternative perspectives".to_string(),
                 "Verify assumptions with evidence".to_string()]
        } else {
            vec![]
        };
        
        Ok(BiasCheckResult {
            has_bias,
            bias_types,
            severity,
            explanation: content.lines().take(3).collect::<Vec<_>>().join(" "),
            suggestions,
        })
    }
    
    fn get_client_for_model(&self, model: &str) -> Result<Arc<dyn LLMClient>> {
        if self.model_resolver.is_openrouter_model(model) {
            if self.config.openrouter_api_key.is_none() {
                anyhow::bail!("OpenRouter API key not configured");
            }
            
            if let Some((_, client)) = self.openrouter_clients
                .iter()
                .find(|(m, _)| m == model) {
                Ok(client.clone())
            } else {
                let api_key = self.config.openrouter_api_key.as_ref()
                    .ok_or_else(|| anyhow::anyhow!("OpenRouter API key not available"))?;
                let new_client = OpenRouterClient::new(
                    api_key.clone(),
                    model.to_string(),
                    self.config.openrouter_base_url.clone(),
                )?;
                Ok(Arc::new(new_client) as Arc<dyn LLMClient>)
            }
        } else {
            if self.openai_client.is_some() {
                if let Some(api_key) = &self.config.openai_api_key {
                    let new_client = OpenAIClient::new(
                        api_key.clone(),
                        model.to_string(),
                        self.config.openai_base_url.clone(),
                    )?;
                    Ok(Arc::new(new_client) as Arc<dyn LLMClient>)
                } else {
                    anyhow::bail!("OpenAI API key not configured");
                }
            } else {
                anyhow::bail!("OpenAI API key not configured");
            }
        }
    }
}