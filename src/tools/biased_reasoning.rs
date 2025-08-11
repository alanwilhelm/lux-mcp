use anyhow::Result;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, error, info, warn};

use crate::llm::{
    client::{ChatMessage, LLMClient},
    config::LLMConfig,
    model_aliases::ModelResolver,
    openai::OpenAIClient,
    openrouter::OpenRouterClient,
    Role,
};
use crate::session::SessionManager;
use lux_synthesis::{
    events::{ActionItem, InsightEntry, Priority},
    EvolvingSynthesis, SynthesisEngine, SynthesisSink, SynthesisState,
};

// Backward compatibility types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesisSnapshot {
    pub current_understanding: String,
    pub top_insights: Vec<String>,
    pub next_actions: Vec<String>,
    pub confidence_level: String,
    pub ready_for_decision: bool,
}

impl From<&SynthesisState> for SynthesisSnapshot {
    fn from(state: &SynthesisState) -> Self {
        SynthesisSnapshot {
            current_understanding: state.current_understanding.clone(),
            top_insights: state
                .key_insights
                .iter()
                .take(5)
                .map(|i| i.insight.clone())
                .collect(),
            next_actions: state
                .action_items
                .iter()
                .filter(|a| matches!(a.priority, Priority::High))
                .take(3)
                .map(|a| a.action.clone())
                .collect(),
            confidence_level: match (state.confidence_score * 100.0) as i32 {
                0..=30 => "low".to_string(),
                31..=70 => "medium".to_string(),
                _ => "high".to_string(),
            },
            ready_for_decision: state.confidence_score >= 0.7,
        }
    }
}

// Helper function to apply old patch format as events
fn apply_patch_as_events(
    synthesis: &EvolvingSynthesis,
    patch: crate::tools::biased_reasoning_synthesis::SynthesisPatch,
    step_count: u32,
) {
    use lux_synthesis::events::SynthesisEvent;

    // Apply understanding update
    if let Some(understanding) = patch.current_understanding {
        let _ = synthesis.apply(SynthesisEvent::Understanding {
            text: understanding,
            confidence: patch.confidence_score,
            clarity: patch.clarity_score,
        });
    }

    // Apply insights
    if let Some(insights) = patch.key_insights {
        for insight in insights {
            let _ = synthesis.apply(SynthesisEvent::Insight(InsightEntry {
                insight: insight.insight,
                confidence: insight.confidence,
                source_step: step_count,
                supported_by_evidence: insight.supported_by_evidence,
            }));
        }
    }

    // Apply action items
    if let Some(actions) = patch.action_items {
        for action in actions {
            let _ = synthesis.apply(SynthesisEvent::Action(ActionItem {
                action: action.action,
                priority: match action.priority {
                    crate::tools::biased_reasoning_synthesis::Priority::High => Priority::High,
                    crate::tools::biased_reasoning_synthesis::Priority::Medium => Priority::Medium,
                    crate::tools::biased_reasoning_synthesis::Priority::Low => Priority::Low,
                },
                rationale: action.rationale,
                dependencies: action.dependencies,
            }));
        }
    }

    // Mark step complete
    let _ = synthesis.apply(SynthesisEvent::StepComplete {
        step_number: step_count,
    });
}

// Helper functions for formatting synthesis
fn format_current_synthesis(state: &SynthesisState) -> String {
    let snapshot: SynthesisSnapshot = state.into();
    let mut output = String::new();

    // Header with version
    output.push_str("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    output.push_str(&format!(
        "📊 **EVOLVING SYNTHESIS** (Version {})\n",
        state.version
    ));
    output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n");

    // Understanding section
    output.push_str("🎯 **Current Understanding:**\n");
    output.push_str(&format!(
        "   {}\n\n",
        if snapshot.current_understanding.is_empty() {
            "🔄 Building understanding..."
        } else {
            &snapshot.current_understanding
        }
    ));

    // Insights section
    output.push_str("💡 **Key Insights Collected:**\n");
    if state.key_insights.is_empty() {
        output.push_str("   • No insights collected yet\n");
    } else {
        for (i, insight) in state.key_insights.iter().enumerate() {
            let confidence_emoji = match (insight.confidence * 100.0) as i32 {
                0..=30 => "🔴",
                31..=60 => "🟡",
                61..=85 => "🟢",
                _ => "⭐",
            };
            output.push_str(&format!(
                "   {}. {} {} (conf: {:.0}%, step {})\n",
                i + 1,
                confidence_emoji,
                insight.insight,
                insight.confidence * 100.0,
                insight.source_step
            ));
        }
    }
    output.push_str("\n");

    // Action items section
    output.push_str("📋 **Action Items:**\n");
    if state.action_items.is_empty() {
        output.push_str("   • No specific actions identified yet\n");
    } else {
        for (i, action) in state.action_items.iter().enumerate() {
            let priority_emoji = match action.priority {
                Priority::High => "🔴",
                Priority::Medium => "🟡",
                Priority::Low => "🟢",
            };
            output.push_str(&format!(
                "   {}. {} [{}] {}\n",
                i + 1,
                priority_emoji,
                format!("{:?}", action.priority),
                action.action
            ));
            if !action.rationale.is_empty() {
                output.push_str(&format!("      → Rationale: {}\n", action.rationale));
            }
        }
    }
    output.push_str("\n");

    // Status indicators
    output.push_str("📈 **Progress Indicators:**\n");
    output.push_str(&format!(
        "   • Confidence: {:.0}% {}\n",
        state.confidence_score * 100.0,
        match (state.confidence_score * 100.0) as i32 {
            0..=30 => "🔴 Low - still exploring",
            31..=60 => "🟡 Medium - building understanding",
            61..=85 => "🟢 Good - clear picture emerging",
            _ => "⭐ High - ready for decisions",
        }
    ));
    output.push_str(&format!(
        "   • Clarity: {:.0}% {}\n",
        state.clarity_score * 100.0,
        if state.clarity_score >= 0.7 {
            "✓"
        } else {
            "..."
        }
    ));
    output.push_str(&format!(
        "   • Ready for Decision: {}\n",
        if snapshot.ready_for_decision {
            "✅ Yes"
        } else {
            "🔄 Not yet"
        }
    ));

    output.push_str("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    output
}

// Helper function to format final synthesis
fn format_final_synthesis(
    state: &SynthesisState,
    total_steps: u32,
    models_used: Vec<String>,
) -> String {
    let mut output = String::new();

    output.push_str("✅ **FINAL SYNTHESIS** ✅\n\n");

    // Executive Summary
    output.push_str("## Executive Summary\n");
    output.push_str(&format!("{}\n\n", state.current_understanding));

    // Key Insights
    output.push_str("## Key Insights\n");
    if state.key_insights.is_empty() {
        output.push_str("No specific insights collected.\n\n");
    } else {
        for (i, insight) in state.key_insights.iter().enumerate() {
            output.push_str(&format!(
                "{}. {} (Confidence: {:.0}%)\n",
                i + 1,
                insight.insight,
                insight.confidence * 100.0
            ));
        }
        output.push_str("\n");
    }

    // Action Plan
    output.push_str("## Action Plan\n\n");

    // High priority actions
    let high_priority_actions: Vec<_> = state
        .action_items
        .iter()
        .filter(|a| matches!(a.priority, Priority::High))
        .collect();

    if !high_priority_actions.is_empty() {
        output.push_str("### Immediate Actions (High Priority)\n");
        for action in high_priority_actions {
            output.push_str(&format!("- {}\n", action.action));
            if !action.rationale.is_empty() {
                output.push_str(&format!("  → {}\n", action.rationale));
            }
        }
        output.push_str("\n");
    }

    // Medium priority actions
    let medium_priority_actions: Vec<_> = state
        .action_items
        .iter()
        .filter(|a| matches!(a.priority, Priority::Medium))
        .collect();

    if !medium_priority_actions.is_empty() {
        output.push_str("### Medium-term Actions\n");
        for action in medium_priority_actions {
            output.push_str(&format!("- {}\n", action.action));
        }
        output.push_str("\n");
    }

    // Low priority actions
    let low_priority_actions: Vec<_> = state
        .action_items
        .iter()
        .filter(|a| matches!(a.priority, Priority::Low))
        .collect();

    if !low_priority_actions.is_empty() {
        output.push_str("### Additional Considerations\n");
        for action in low_priority_actions {
            output.push_str(&format!("- {}\n", action.action));
        }
        output.push_str("\n");
    }

    // Context & Methodology
    output.push_str("## Context & Methodology\n");
    output.push_str(&format!("- **Total Analysis Steps:** {}\n", total_steps));
    output.push_str(&format!("- **Models Used:** {}\n", models_used.join(", ")));
    output.push_str(&format!(
        "- **Final Confidence:** {:.0}%\n",
        state.confidence_score * 100.0
    ));
    output.push_str(&format!(
        "- **Clarity Score:** {:.0}%\n",
        state.clarity_score * 100.0
    ));
    output.push_str(&format!("- **Synthesis Version:** {}\n", state.version));

    output
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum StepType {
    Query,        // Initial question
    Reasoning,    // Primary model reasoning
    BiasAnalysis, // Bias check result (VISIBLE)
    Correction,   // Corrected reasoning (VISIBLE)
    Guidance,     // User input/guidance
    Synthesis,    // Final compilation
}

impl std::fmt::Display for StepType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StepType::Query => write!(f, "❓ Query"),
            StepType::Reasoning => write!(f, "💭 Reasoning"),
            StepType::BiasAnalysis => write!(f, "🔍 Bias Analysis"),
            StepType::Correction => write!(f, "✏️ Correction"),
            StepType::Guidance => write!(f, "🎯 Guidance"),
            StepType::Synthesis => write!(f, "🎨 Synthesis"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum NextAction {
    BiasCheck,         // Next step should be bias analysis
    CorrectionNeeded,  // Bias found, correction recommended
    ContinueReasoning, // Continue with next reasoning step
    AwaitingGuidance,  // Waiting for user input
    ReadyForSynthesis, // Ready to compile final answer
    Complete,          // Process complete
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStatus {
    pub total_steps: u32,
    pub reasoning_steps: u32,
    pub bias_checks: u32,
    pub corrections_made: u32,
    pub overall_quality: f32, // 0.0 to 1.0
    pub is_complete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrectionDetails {
    pub original_text: String,
    pub corrected_text: String,
    pub changes_made: Vec<String>,
    pub improvement_score: f32, // 0.0 to 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningMetadata {
    pub thinking_time_ms: u64,
    pub tokens_generated: Option<u32>,
    pub confidence_level: f32,
    pub reasoning_depth: String, // "shallow", "moderate", "deep"
}

// Step-by-step request
#[derive(Debug, Clone, Serialize, Deserialize)]
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

    /// Optional file paths to include in reasoning context
    #[serde(default)]
    pub file_paths: Option<Vec<String>>,

    /// Whether to include file contents (default: true)
    #[serde(default = "default_true")]
    pub include_file_contents: bool,
}

fn default_max_steps() -> u32 {
    3
}

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub synthesis_snapshot: Option<SynthesisSnapshot>,
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

fn default_true() -> bool {
    true
}
fn default_bias_threshold() -> f32 {
    0.7
}

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
    pub confidence: f32, // 0.0 to 1.0 - confidence in the bias detection
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
    synthesis: Arc<Mutex<EvolvingSynthesis>>,
    synthesis_sink: Option<Arc<dyn SynthesisSink>>,
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
        let model_resolver = ModelResolver::with_config(Some(config.clone()));

        let openai_client = if let Some(api_key) = &config.openai_api_key {
            let client = OpenAIClient::new(
                api_key.clone(),
                config.model_reasoning.clone(),
                config.openai_base_url.clone(),
            )?;
            Some(Arc::new(client) as Arc<dyn LLMClient>)
        } else {
            None
        };

        let mut openrouter_clients = Vec::new();
        if let Some(api_key) = &config.openrouter_api_key {
            let common_models = vec!["anthropic/claude-3-opus", "google/gemini-2.5-pro"];

            for model in common_models {
                let client = OpenRouterClient::new(
                    api_key.clone(),
                    model.to_string(),
                    config.openrouter_base_url.clone(),
                )?;
                openrouter_clients
                    .push((model.to_string(), Arc::new(client) as Arc<dyn LLMClient>));
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
    pub async fn process_step(
        &self,
        request: BiasedReasoningRequest,
    ) -> Result<BiasedReasoningResponse> {
        use chrono::Utc;
        use sha2::{Digest, Sha256};

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

        // STRICT POLICY: Only GPT-5 family allowed
        let mut primary_model = self.model_resolver.resolve("gpt-5");
        let mut verifier_model = self.model_resolver.resolve("gpt-5-mini");
        if !self.model_resolver.is_allowed_model(&primary_model) {
            primary_model = "gpt-5".to_string();
        }
        if !self.model_resolver.is_allowed_model(&verifier_model) {
            verifier_model = "gpt-5-mini".to_string();
        }

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
                    synthesis: Arc::new(Mutex::new(EvolvingSynthesis::new_in_memory(
                        "biased_reasoning",
                        &session_id,
                    ))),
                    synthesis_sink: None, // Will be set from handler if DB is available
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
                    StepType::BiasAnalysis => {
                        // Check if ready for synthesis
                        if session.reasoning_steps.len() >= request.max_analysis_rounds as usize {
                            StepType::Synthesis
                        } else {
                            StepType::Reasoning
                        }
                    }
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
                // Get initial synthesis snapshot
                let synthesis_snapshot = {
                    let sessions = self.sessions.lock();
                    if let Some(session) = sessions.get(&session_id) {
                        let synthesis = session.synthesis.lock();
                        Some((&synthesis.snapshot()).into())
                    } else {
                        None
                    }
                };

                let formatted_content = {
                    let sessions = self.sessions.lock();
                    if let Some(session) = sessions.get(&session_id) {
                        let synthesis = session.synthesis.lock();
                        let state = synthesis.snapshot();
                        format!(
                            "📝 **Query Received - Step {}**
📊 **Models:** {} (reasoning) | {} (bias checking)
🎯 **Confidence:** {:.0}% | **Clarity:** {:.0}%
💭 **Understanding:** {}
💡 **Insights:** {} collected | **Actions:** {} identified
⚡ **Status:** {} | **Progress:** Initializing

🔧 **Configuration:**
- Primary Model: **{}**
- Bias Checker: **{}**
- Max Analysis Rounds: {}
- Session ID: {}

📌 **Query:** 
{}

{}

🚀 Starting step-by-step reasoning with real-time bias checking...",
                            step_count,
                            primary_model,
                            verifier_model,
                            state.confidence_score * 100.0,
                            state.clarity_score * 100.0,
                            if state.current_understanding.is_empty() {
                                "Initializing analysis..."
                            } else {
                                &state.current_understanding
                            },
                            state.key_insights.len(),
                            state.action_items.len(),
                            "🟦 Starting",
                            primary_model,
                            verifier_model,
                            request.max_analysis_rounds,
                            session_id,
                            request.query,
                            format_current_synthesis(&state)
                        )
                    } else {
                        format!("📝 **Query Received**\n\n🔧 **Configuration:**\n- Primary Model: **{}**\n- Bias Checker: **{}**\n- Max Rounds: {}\n\n📌 **Query:** {}\n\n🚀 Starting reasoning...", 
                            primary_model,
                            verifier_model,
                            request.max_analysis_rounds,
                            request.query)
                    }
                };

                Ok(BiasedReasoningResponse {
                    session_id: session_id.clone(),
                    step_type: StepType::Query,
                    step_number: step_count,
                    content: formatted_content,
                    model_used: format!("{} / {}", primary_model, verifier_model),
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
                    synthesis_snapshot,
                })
            }

            StepType::Reasoning => {
                self.handle_reasoning_step(session_id, step_count, primary_model)
                    .await
            }

            StepType::BiasAnalysis => {
                self.handle_bias_analysis_step(
                    session_id,
                    step_count,
                    verifier_model,
                    request.max_analysis_rounds,
                )
                .await
            }

            StepType::Synthesis => {
                self.handle_synthesis_step(session_id, step_count, primary_model)
                    .await
            }

            _ => Err(anyhow::anyhow!(
                "Step type not yet implemented: {:?}",
                step_type
            )),
        }
    }

    async fn handle_reasoning_step(
        &self,
        session_id: String,
        step_count: u32,
        primary_model: String,
    ) -> Result<BiasedReasoningResponse> {
        use chrono::Utc;

        // Get conversation and query from session
        let (conversation, query, synthesis_arc) = {
            let sessions = self.sessions.lock();
            if let Some(session) = sessions.get(&session_id) {
                (
                    session.primary_conversation.clone(),
                    session.query.clone(),
                    session.synthesis.clone(),
                )
            } else {
                return Err(anyhow::anyhow!("Session not found"));
            }
        };

        let primary_client = self.get_client_for_model(&primary_model)?;

        info!(
            "⚡ Generating cognitive frame with synthesis engine: {}",
            primary_model
        );
        let start = Instant::now();

        // Use synthesis-aware reasoning
        // We need to avoid holding the lock across await points
        // So we'll use standard prompting and then update synthesis
        let reasoning_content = {
            // Get current synthesis state
            let current_synthesis = {
                let synthesis = synthesis_arc.lock();
                synthesis.snapshot()
            };

            // Create prompt
            let prompt = crate::tools::biased_reasoning_prompts::reasoning_prompt_with_synthesis(
                &query,
                &current_synthesis,
                step_count,
            );

            let messages = vec![
                ChatMessage {
                    role: Role::System,
                    content: "You are a reasoning assistant that provides structured analysis. Always use the update_synthesis function to record your findings.".to_string(),
                },
                ChatMessage {
                    role: Role::User,
                    content: prompt,
                },
            ];

            // Call LLM
            let response = primary_client
                .complete(
                    messages,
                    if crate::llm::token_config::TokenConfig::requires_default_temperature(
                        &primary_model,
                    ) {
                        None
                    } else {
                        Some(0.7)
                    },
                    Some(crate::llm::token_config::TokenConfig::get_reasoning_tokens(
                        &primary_model,
                    )),
                )
                .await?;

            // Parse and apply synthesis update
            if let Ok(patch) =
                crate::tools::biased_reasoning_prompts::extract_synthesis_update(&response.content)
            {
                let synthesis = synthesis_arc.lock();
                // Convert patch to events and apply
                apply_patch_as_events(&synthesis, patch, step_count);
            }

            response.content
        };

        let duration = start.elapsed();
        info!(
            "✅ {} completed reasoning with synthesis in {:?}",
            primary_model, duration
        );

        // Update session with new reasoning
        let (session_status, synthesis_snapshot) = {
            let mut sessions = self.sessions.lock();
            if let Some(session) = sessions.get_mut(&session_id) {
                // Add to conversation
                session.primary_conversation.push(ChatMessage {
                    role: Role::Assistant,
                    content: reasoning_content.clone(),
                });

                // Log the step
                session.detailed_process_log.push(ProcessLogEntry {
                    action_type: ProcessActionType::PrimaryReasoning,
                    step_number: step_count,
                    timestamp: Utc::now().to_rfc3339(),
                    model_used: primary_model.clone(),
                    content: format!(
                        "Generated reasoning step with synthesis:\n{}",
                        reasoning_content
                    ),
                    duration_ms: Some(duration.as_millis() as u64),
                });

                // Get synthesis snapshot
                let synthesis = session.synthesis.lock();
                let state = synthesis.snapshot();
                let snapshot: SynthesisSnapshot = (&state).into();

                let status = SessionStatus {
                    total_steps: step_count,
                    reasoning_steps: session.reasoning_steps.len() as u32 + 1,
                    bias_checks: session
                        .reasoning_steps
                        .iter()
                        .filter(|s| s.bias_check.has_bias)
                        .count() as u32,
                    corrections_made: session
                        .reasoning_steps
                        .iter()
                        .filter(|s| s.corrected_thought.is_some())
                        .count() as u32,
                    overall_quality: state.confidence_score,
                    is_complete: false,
                };

                (status, Some(snapshot))
            } else {
                return Err(anyhow::anyhow!("Session not found"));
            }
        };

        let reasoning_depth = self.assess_reasoning_depth(&reasoning_content);

        // Include synthesis summary in content
        let formatted_content = {
            let synthesis = synthesis_arc.lock();
            let state = synthesis.snapshot();
            let snapshot: SynthesisSnapshot = (&state).into();

            format!(
                "⚡ **COGNITIVE FRAME {}** ⚡
═══════════════════════════════════════
🔮 **Reasoning Engine:** {} 
📊 **Confidence:** {:.0}% | **Clarity:** {:.0}%
🎯 **Current Analysis:** {}
💫 **Insights Collected:** {} | **Actions Identified:** {}
⚡ **Processing Status:** {} | **Frame {}/{}
═══════════════════════════════════════

{}

{}",
                step_count,
                primary_model,
                state.confidence_score * 100.0,
                state.clarity_score * 100.0,
                if state.current_understanding.is_empty() {
                    "Building understanding..."
                } else {
                    &state.current_understanding
                },
                state.key_insights.len(),
                state.action_items.len(),
                match (state.confidence_score * 100.0) as i32 {
                    0..=30 => "🔴 Low confidence - exploring",
                    31..=60 => "🟡 Medium confidence - analyzing",
                    61..=85 => "🟢 High confidence - converging",
                    _ => "✅ Very high confidence - ready",
                },
                step_count,
                step_count + 2,
                reasoning_content,
                format_current_synthesis(&state)
            )
        };

        Ok(BiasedReasoningResponse {
            session_id: session_id.clone(),
            step_type: StepType::Reasoning,
            step_number: step_count,
            content: formatted_content,
            model_used: primary_model,
            next_action: NextAction::BiasCheck,
            session_status,
            bias_analysis: None,
            correction_details: None,
            reasoning_metadata: Some(ReasoningMetadata {
                thinking_time_ms: duration.as_millis() as u64,
                tokens_generated: None, // synthesis response doesn't include usage
                confidence_level: synthesis_snapshot
                    .as_ref()
                    .map(|s: &SynthesisSnapshot| match s.confidence_level.as_str() {
                        "high" => 0.9,
                        "medium" => 0.6,
                        _ => 0.3,
                    })
                    .unwrap_or(0.5),
                reasoning_depth,
            }),
            synthesis_snapshot,
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

        // Get the last reasoning step, query, and synthesis
        let (last_thought, query, synthesis_arc) = {
            let sessions = self.sessions.lock();
            if let Some(session) = sessions.get(&session_id) {
                let last_thought = session
                    .primary_conversation
                    .last()
                    .filter(|m| m.role == Role::Assistant)
                    .map(|m| {
                        // Extract just the reasoning part (before synthesis snapshot)
                        if let Some(idx) = m.content.find("📊 **SYNTHESIS SNAPSHOT**") {
                            m.content[..idx].trim().to_string()
                        } else {
                            m.content.clone()
                        }
                    })
                    .unwrap_or_default();
                (
                    last_thought,
                    session.query.clone(),
                    session.synthesis.clone(),
                )
            } else {
                return Err(anyhow::anyhow!("Session not found"));
            }
        };

        let verifier_client = self.get_client_for_model(&verifier_model)?;

        info!(
            "🔍 Checking for bias with synthesis using {}",
            verifier_model
        );
        let start = Instant::now();

        // Use synthesis-aware bias checking
        // Avoid holding lock across await
        let (_bias_analysis_content, bias_check) = {
            // Get current synthesis state
            let current_synthesis = {
                let synthesis = synthesis_arc.lock();
                synthesis.snapshot()
            };

            // Create prompt
            let prompt = crate::tools::biased_reasoning_prompts::bias_check_prompt_with_synthesis(
                &last_thought,
                &current_synthesis,
            );

            let messages = vec![
                ChatMessage {
                    role: Role::System,
                    content: "You are a bias detection assistant. Analyze reasoning for biases and update the synthesis with any corrections needed.".to_string(),
                },
                ChatMessage {
                    role: Role::User,
                    content: prompt,
                },
            ];

            // Call LLM
            let response = verifier_client
                .complete(
                    messages,
                    if crate::llm::token_config::TokenConfig::requires_default_temperature(
                        &verifier_model,
                    ) {
                        None
                    } else {
                        Some(0.3)
                    },
                    Some(crate::llm::token_config::TokenConfig::get_optimal_tokens(
                        &verifier_model,
                    )),
                )
                .await?;

            // Parse bias check
            let bias_check = self.parse_bias_check_response(&response.content)?;

            // Parse and apply synthesis update
            if let Ok(patch) =
                crate::tools::biased_reasoning_prompts::extract_synthesis_update(&response.content)
            {
                let synthesis = synthesis_arc.lock();
                apply_patch_as_events(&synthesis, patch, step_count);
            }

            (response.content, bias_check)
        };

        let duration = start.elapsed();
        info!("✅ Bias check with synthesis completed in {:?}", duration);

        // Update session and determine next action
        let (next_action, session_status, synthesis_snapshot) = {
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
                        bias_check.has_bias, bias_check.bias_types, bias_check.severity
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

                // Get synthesis snapshot
                let synthesis = session.synthesis.lock();
                let snapshot: SynthesisSnapshot = (&synthesis.snapshot()).into();

                let status = SessionStatus {
                    total_steps: step_count,
                    reasoning_steps: session.reasoning_steps.len() as u32,
                    bias_checks: session
                        .reasoning_steps
                        .iter()
                        .filter(|s| s.bias_check.has_bias)
                        .count() as u32,
                    corrections_made: session
                        .reasoning_steps
                        .iter()
                        .filter(|s| s.corrected_thought.is_some())
                        .count() as u32,
                    overall_quality: synthesis.snapshot().confidence_score,
                    is_complete: next_action == NextAction::Complete
                        || next_action == NextAction::ReadyForSynthesis,
                };

                (next_action, status, Some(snapshot))
            } else {
                return Err(anyhow::anyhow!("Session not found"));
            }
        };

        // Include synthesis summary in content
        let formatted_content = {
            let synthesis = synthesis_arc.lock();
            let state = synthesis.snapshot();

            format!(
                "🔍 **BIAS DETECTION FRAME {}** 🔍
═══════════════════════════════════════
🔮 **Validation Engine:** {} 
📊 **Confidence:** {:.0}% | **Clarity:** {:.0}%
🎯 **Current Analysis:** {}
💫 **Insights Collected:** {} | **Actions Identified:** {}
⚠️  **Anomalies Detected:** {} | **Severity:** {}
⚡ **Processing Status:** {} | **Frame {}/{}
═══════════════════════════════════════

Pattern Analysis:

{}
{}
{}

{}",
                step_count,
                verifier_model,
                state.confidence_score * 100.0,
                state.clarity_score * 100.0,
                if state.current_understanding.is_empty() {
                    "Building understanding..."
                } else {
                    &state.current_understanding
                },
                state.key_insights.len(),
                state.action_items.len(),
                bias_check.bias_types.len(),
                if bias_check.has_bias {
                    match bias_check.severity {
                        Severity::Critical => "🔴 Critical",
                        Severity::High => "🟠 High",
                        Severity::Medium => "🟡 Medium",
                        Severity::Low => "🟢 Low",
                        Severity::None => "✅ None",
                    }
                } else {
                    "✅ None"
                },
                match (state.confidence_score * 100.0) as i32 {
                    0..=30 => "🔴 Low confidence - exploring",
                    31..=60 => "🟡 Medium confidence - analyzing",
                    61..=85 => "🟢 High confidence - converging",
                    _ => "✅ Very high confidence - ready",
                },
                step_count,
                step_count + 1,
                if bias_check.has_bias {
                    "⚠️ Biases detected!"
                } else {
                    "✅ No significant biases detected."
                },
                if !bias_check.bias_types.is_empty() {
                    format!("\nBias types: {:?}", bias_check.bias_types)
                } else {
                    String::new()
                },
                if !bias_check.suggestions.is_empty() {
                    format!("\nSuggestions: {}", bias_check.suggestions.join(", "))
                } else {
                    String::new()
                },
                format_current_synthesis(&state)
            )
        };

        Ok(BiasedReasoningResponse {
            session_id: session_id.clone(),
            step_type: StepType::BiasAnalysis,
            step_number: step_count,
            content: formatted_content,
            model_used: verifier_model,
            next_action,
            session_status,
            bias_analysis: Some(bias_check),
            correction_details: None,
            reasoning_metadata: None,
            synthesis_snapshot,
        })
    }

    async fn handle_synthesis_step(
        &self,
        session_id: String,
        step_count: u32,
        primary_model: String,
    ) -> Result<BiasedReasoningResponse> {
        use chrono::Utc;

        // Get query, synthesis, and models used from session
        let (query, synthesis_arc, models_used) = {
            let sessions = self.sessions.lock();
            if let Some(session) = sessions.get(&session_id) {
                let mut models = vec![primary_model.clone()];

                // Extract unique models from process log
                for entry in &session.detailed_process_log {
                    if !models.contains(&entry.model_used) {
                        models.push(entry.model_used.clone());
                    }
                }

                (session.query.clone(), session.synthesis.clone(), models)
            } else {
                return Err(anyhow::anyhow!("Session not found"));
            }
        };

        let primary_client = self.get_client_for_model(&primary_model)?;

        info!("🎯 Generating final synthesis with {}", primary_model);
        let start = Instant::now();

        // Generate final synthesis
        let summary_content = {
            // Get current synthesis state
            let current_synthesis = {
                let synthesis = synthesis_arc.lock();
                synthesis.snapshot()
            };

            // Create prompt
            let prompt = crate::tools::biased_reasoning_prompts::final_synthesis_prompt(
                &query,
                &current_synthesis,
                step_count,
            );

            let messages = vec![
                ChatMessage {
                    role: Role::System,
                    content: "Generate a complete, actionable synthesis of the analysis."
                        .to_string(),
                },
                ChatMessage {
                    role: Role::User,
                    content: prompt,
                },
            ];

            // Call LLM
            let response = primary_client
                .complete(
                    messages,
                    if crate::llm::token_config::TokenConfig::requires_default_temperature(
                        &primary_model,
                    ) {
                        None
                    } else {
                        Some(0.7)
                    },
                    Some(crate::llm::token_config::TokenConfig::get_reasoning_tokens(
                        &primary_model,
                    )),
                )
                .await?;

            // Parse and apply synthesis update
            if let Ok(patch) =
                crate::tools::biased_reasoning_prompts::extract_synthesis_update(&response.content)
            {
                let synthesis = synthesis_arc.lock();
                apply_patch_as_events(&synthesis, patch, step_count);
            }

            response.content
        };

        let duration = start.elapsed();
        info!("✅ Final synthesis completed in {:?}", duration);

        // Update session and get final status
        let (session_status, synthesis_snapshot) = {
            let mut sessions = self.sessions.lock();
            if let Some(session) = sessions.get_mut(&session_id) {
                // Log the synthesis
                session.detailed_process_log.push(ProcessLogEntry {
                    action_type: ProcessActionType::FinalAnswerGeneration,
                    step_number: step_count,
                    timestamp: Utc::now().to_rfc3339(),
                    model_used: primary_model.clone(),
                    content: "Generated final synthesis".to_string(),
                    duration_ms: Some(duration.as_millis() as u64),
                });

                // Store final answer
                session.final_answer = Some(summary_content.clone());

                // Get final synthesis
                let synthesis = session.synthesis.lock();
                let final_synthesis =
                    format_final_synthesis(&synthesis.snapshot(), step_count, models_used.clone());
                let snapshot = synthesis.snapshot();

                let status = SessionStatus {
                    total_steps: step_count,
                    reasoning_steps: session.reasoning_steps.len() as u32,
                    bias_checks: session
                        .reasoning_steps
                        .iter()
                        .filter(|s| s.bias_check.has_bias)
                        .count() as u32,
                    corrections_made: session
                        .reasoning_steps
                        .iter()
                        .filter(|s| s.corrected_thought.is_some())
                        .count() as u32,
                    overall_quality: synthesis.snapshot().confidence_score,
                    is_complete: true,
                };

                // Update the final answer with the formatted synthesis
                session.final_answer = Some(final_synthesis.clone());

                (status, Some((&snapshot).into()))
            } else {
                return Err(anyhow::anyhow!("Session not found"));
            }
        };

        // Get the formatted final synthesis
        let final_content = {
            let synthesis = synthesis_arc.lock();
            let state = synthesis.snapshot();

            format!(
                "🎯 **FINAL SYNTHESIS - FRAME {}** 🎯
═══════════════════════════════════════
🔮 **Primary Engine:** {} 
📊 **Confidence Level:** {:.0}% | **Clarity Score:** {:.0}%
💫 **Cognitive Analysis:** {}
⚡ **Insights Extracted:** {} | **Actions Generated:** {}
📈 **Processing Quality:** {} | **Synthesis Readiness:** {}
🔧 **Engines Utilized:** {}
✅ **Status:** REASONING CHAIN COMPLETE
═══════════════════════════════════════

{}",
                step_count,
                primary_model,
                state.confidence_score * 100.0,
                state.clarity_score * 100.0,
                if state.current_understanding.is_empty() {
                    "Complete understanding achieved"
                } else {
                    &state.current_understanding
                },
                state.key_insights.len(),
                state.action_items.len(),
                match (session_status.overall_quality * 100.0) as i32 {
                    0..=30 => "🔴 Poor",
                    31..=60 => "🟡 Fair",
                    61..=85 => "🟢 Good",
                    _ => "⭐ Excellent",
                },
                if state.confidence_score >= 0.8 {
                    "✅ Ready for decision"
                } else {
                    "⚠️ Consider additional analysis"
                },
                models_used.join(", "),
                format_final_synthesis(&state, step_count, models_used.clone())
            )
        };

        Ok(BiasedReasoningResponse {
            session_id: session_id.clone(),
            step_type: StepType::Synthesis,
            step_number: step_count,
            content: final_content,
            model_used: primary_model,
            next_action: NextAction::Complete,
            session_status,
            bias_analysis: None,
            correction_details: None,
            reasoning_metadata: Some(ReasoningMetadata {
                thinking_time_ms: duration.as_millis() as u64,
                tokens_generated: None,
                confidence_level: synthesis_snapshot
                    .as_ref()
                    .map(|s: &SynthesisSnapshot| match s.confidence_level.as_str() {
                        "high" => 0.9,
                        "medium" => 0.6,
                        _ => 0.3,
                    })
                    .unwrap_or(0.8),
                reasoning_depth: "deep".to_string(),
            }),
            synthesis_snapshot,
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
            check_prompt
                .push_str("- Confirmation bias: Is the reasoning cherry-picking evidence?\n");
        }
        if config.check_anchoring_bias {
            check_prompt
                .push_str("- Anchoring bias: Is it overly influenced by initial information?\n");
        }
        if config.check_availability_bias {
            check_prompt
                .push_str("- Availability bias: Is it overweighting easily recalled examples?\n");
        }
        if config.check_reasoning_errors {
            check_prompt
                .push_str("- Reasoning errors: Are there logical fallacies or poor inferences?\n");
        }

        check_prompt.push_str(
            "\nProvide a structured analysis with:\n\
            1. Whether bias is present (yes/no)\n\
            2. Types of bias found\n\
            3. Severity (none/low/medium/high/critical)\n\
            4. Confidence in this assessment (0.0-1.0)\n\
            5. Brief explanation\n\
            6. Suggestions for improvement",
        );

        let messages = vec![
            ChatMessage {
                role: Role::System,
                content:
                    "You are a critical thinking expert who identifies biases and reasoning errors."
                        .to_string(),
            },
            ChatMessage {
                role: Role::User,
                content: check_prompt,
            },
        ];

        // Use optimal tokens for verifier model - GPT-5 gets MAXIMUM
        let max_tokens =
            crate::llm::token_config::TokenConfig::get_optimal_tokens(&verifier_model_name);

        let temperature = if crate::llm::token_config::TokenConfig::requires_default_temperature(
            &verifier_model_name,
        ) {
            None
        } else {
            Some(0.3)
        };

        let response = verifier_client
            .complete(messages, temperature, Some(max_tokens))
            .await
            .map_err(|e| {
                error!(
                    "Verifier model '{}' failed during bias check: {}",
                    verifier_model_name, e
                );
                anyhow::anyhow!(
                    "Failed to check for bias with model '{}': {}",
                    verifier_model_name,
                    e
                )
            })?;

        self.parse_bias_check_response(&response.content)
    }

    fn parse_bias_check_response(&self, content: &str) -> Result<BiasCheckResult> {
        let content_lower = content.to_lowercase();

        let has_bias = content_lower.contains("yes")
            || content_lower.contains("bias is present")
            || content_lower.contains("found bias");

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
            vec![
                "Consider alternative perspectives".to_string(),
                "Verify assumptions with evidence".to_string(),
            ]
        } else {
            vec![]
        };

        // Extract confidence from response or calculate based on clarity
        let confidence = if content_lower.contains("confidence:") {
            content
                .lines()
                .find(|l| l.to_lowercase().contains("confidence:"))
                .and_then(|l| {
                    l.split(':')
                        .nth(1)
                        .and_then(|s| s.trim().parse::<f32>().ok())
                })
                .unwrap_or(0.8)
        } else {
            // Calculate confidence based on clarity of detection
            match severity {
                Severity::Critical => 0.95,
                Severity::High => 0.85,
                Severity::Medium => 0.75,
                Severity::Low => 0.65,
                Severity::None => {
                    if has_bias {
                        0.5
                    } else {
                        0.9
                    }
                }
            }
        };

        Ok(BiasCheckResult {
            has_bias,
            bias_types,
            severity,
            explanation: content.lines().take(3).collect::<Vec<_>>().join(" "),
            suggestions,
            confidence,
        })
    }

    fn assess_reasoning_depth(&self, content: &str) -> String {
        let word_count = content.split_whitespace().count();
        let has_examples = content.contains("example")
            || content.contains("instance")
            || content.contains("for example");
        let has_analysis = content.contains("because")
            || content.contains("therefore")
            || content.contains("thus")
            || content.contains("consequently");
        let has_structure =
            content.contains("first") || content.contains("second") || content.contains("finally");

        if word_count > 200 && has_examples && has_analysis && has_structure {
            "deep".to_string()
        } else if word_count > 100 && (has_analysis || has_examples) {
            "moderate".to_string()
        } else {
            "shallow".to_string()
        }
    }

    fn get_client_for_model(&self, model: &str) -> Result<Arc<dyn LLMClient>> {
        if self.model_resolver.is_openrouter_model(model) {
            if self.config.openrouter_api_key.is_none() {
                anyhow::bail!("OpenRouter API key not configured");
            }

            if let Some((_, client)) = self.openrouter_clients.iter().find(|(m, _)| m == model) {
                Ok(client.clone())
            } else {
                let api_key = self
                    .config
                    .openrouter_api_key
                    .as_ref()
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

    /// Set the synthesis sink for all sessions
    /// Read files and return their contents
    fn read_files(&self, file_paths: &[String]) -> Vec<(String, String)> {
        let mut file_contents = Vec::new();

        for path in file_paths {
            let file_path = Path::new(path);
            if file_path.exists() && file_path.is_file() {
                match fs::read_to_string(file_path) {
                    Ok(content) => {
                        info!("Read file for bias analysis: {}", path);
                        // Truncate very large files
                        let truncated = if content.len() > 15000 {
                            format!("{}... [truncated]", &content[..15000])
                        } else {
                            content
                        };
                        file_contents.push((path.clone(), truncated));
                    }
                    Err(e) => {
                        warn!("Failed to read file {}: {}", path, e);
                    }
                }
            } else {
                debug!("File not found or not a file: {}", path);
            }
        }

        file_contents
    }

    pub fn set_synthesis_sink(&self, sink: Arc<dyn SynthesisSink>) {
        let mut sessions = self.sessions.lock();
        for (_, session) in sessions.iter_mut() {
            session.synthesis_sink = Some(sink.clone());
        }
    }
}
