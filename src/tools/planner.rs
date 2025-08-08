use crate::llm::{
    client::{ChatMessage, LLMClient},
    config::LLMConfig,
    model_aliases::ModelResolver,
    openai::OpenAIClient,
    openrouter::OpenRouterClient,
    Role,
};
use crate::session::SessionManager;
use anyhow::{Context, Result};
use lux_synthesis::{
    events::{ActionItem, InsightEntry, Priority},
    EvolvingSynthesis, SynthesisEngine, SynthesisSink,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex as StdMutex};
use tracing::{error, info};

#[derive(Debug, Deserialize, Serialize)]
pub struct PlannerRequest {
    pub step: String,
    pub step_number: u32,
    pub total_steps: u32,
    pub next_step_required: bool,

    #[serde(default)]
    pub is_step_revision: bool,
    #[serde(default)]
    pub revises_step_number: Option<u32>,
    #[serde(default)]
    pub is_branch_point: bool,
    #[serde(default)]
    pub branch_from_step: Option<u32>,
    #[serde(default)]
    pub branch_id: Option<String>,
    #[serde(default)]
    pub more_steps_needed: bool,

    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
}

fn default_temperature() -> f32 {
    0.7
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlannerResponse {
    pub status: String,
    pub step_number: u32,
    pub total_steps: u32,
    pub next_step_required: bool,
    pub step_content: String,
    pub metadata: PlannerMetadata,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub continuation_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub planning_complete: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan_summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_steps: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_required: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_thinking: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub planner_required: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_used: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub synthesis_snapshot: Option<SynthesisSnapshot>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SynthesisSnapshot {
    pub current_plan: String,
    pub key_decisions: Vec<String>,
    pub next_actions: Vec<String>,
    pub confidence_level: String,
    pub ready_for_execution: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlannerMetadata {
    pub branches: Vec<String>,
    pub step_history_length: u32,
    pub is_step_revision: bool,
    pub revises_step_number: Option<u32>,
    pub is_branch_point: bool,
    pub branch_from_step: Option<u32>,
    pub branch_id: Option<String>,
    pub more_steps_needed: bool,
}

#[derive(Debug, Clone)]
pub struct StepData {
    pub step_number: u32,
    pub content: String,
    pub is_revision: bool,
    pub revises_step: Option<u32>,
    pub branch_id: Option<String>,
}

pub struct PlannerTool {
    session_manager: Arc<SessionManager>,
    openai_client: Option<Arc<dyn LLMClient>>,
    openrouter_clients: Vec<(String, Arc<dyn LLMClient>)>,
    model_resolver: ModelResolver,
    config: LLMConfig,
    step_history: Vec<StepData>,
    branches: HashMap<String, Vec<StepData>>,
    synthesis: Arc<StdMutex<EvolvingSynthesis>>,
    synthesis_sink: Option<Arc<dyn SynthesisSink>>,
}

impl PlannerTool {
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
                "anthropic/claude-3.5-sonnet-20241022",
                "google/gemini-2.0-flash-thinking-exp:free",
            ];

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

        let synthesis = Arc::new(StdMutex::new(EvolvingSynthesis::new_in_memory(
            "planner",
            "default_session",
        )));

        Ok(Self {
            session_manager,
            openai_client,
            openrouter_clients,
            model_resolver,
            config,
            step_history: Vec::new(),
            branches: HashMap::new(),
            synthesis,
            synthesis_sink: None,
        })
    }

    /// Set synthesis sink for database persistence
    pub fn set_synthesis_sink(&mut self, sink: Arc<dyn SynthesisSink>) {
        self.synthesis_sink = Some(sink);
    }

    pub async fn create_plan(&mut self, request: PlannerRequest) -> Result<PlannerResponse> {
        let session_id = self
            .session_manager
            .get_or_create_session(request.session_id.clone());
        let monitor = self.session_manager.get_monitor(&session_id)?;

        // Create synthesis for this session with appropriate sink
        if let Some(sink) = &self.synthesis_sink {
            // For now, we'll use in-memory synthesis even with a sink available
            // TODO: Create a constructor that accepts a custom sink
            self.synthesis = Arc::new(StdMutex::new(EvolvingSynthesis::new_in_memory(
                "planner",
                &session_id,
            )));
        } else {
            self.synthesis = Arc::new(StdMutex::new(EvolvingSynthesis::new_in_memory(
                "planner",
                &session_id,
            )));
        }

        // Validate step number
        if request.step_number < 1 {
            anyhow::bail!("step_number must be at least 1");
        }

        if request.total_steps < 1 {
            anyhow::bail!("total_steps must be at least 1");
        }

        // Get model for planning
        let model = request
            .model
            .as_ref()
            .map(|m| self.model_resolver.resolve(m))
            .unwrap_or_else(|| self.config.default_reasoning_model.clone());

        info!(
            "Planner request - Step {}/{}, Model: {}, Temperature: {}, Session: {:?}",
            request.step_number,
            request.total_steps,
            model,
            request.temperature,
            request.session_id
        );

        // Log processing status for long-running models
        if model.starts_with("o3") {
            info!(
                "⏳ Using {} - this may take 30 seconds to 5 minutes. Processing...",
                model
            );
        }

        // Generate planning content using LLM
        let generated_content = if request.step_number == 1 {
            // For first step, just use the provided description as the goal
            request.step.clone()
        } else {
            // Build context from previous steps
            let context = self.build_planning_context(&request);

            // Create prompt for LLM
            let system_prompt = self.build_system_prompt(&request);
            let user_prompt = self.build_user_prompt(&request, &context);

            // Call LLM to generate step content
            let client = self
                .get_client_for_model(&model)
                .context("Failed to get LLM client")?;

            let messages = vec![
                ChatMessage {
                    role: Role::System,
                    content: system_prompt,
                },
                ChatMessage {
                    role: Role::User,
                    content: user_prompt,
                },
            ];

            // Use more tokens for planning steps and add better error handling
            let max_tokens = if model.starts_with("o3") {
                32768
            } else {
                10000
            };

            info!(
                "🚀 Sending planning request to {} (max_tokens: {})",
                model, max_tokens
            );
            if model.starts_with("o3") {
                info!("⏳ Deep reasoning in progress. This is normal for o3 models...");
            }

            let start_time = std::time::Instant::now();
            let response = client
                .complete(messages, Some(request.temperature), Some(max_tokens))
                .await
                .map_err(|e| {
                    let elapsed = start_time.elapsed();
                    error!(
                        "LLM call failed after {:?} for model '{}': {}",
                        elapsed, model, e
                    );
                    anyhow::anyhow!(
                        "Failed to generate planning step with model '{}' after {:?}: {}",
                        model,
                        elapsed,
                        e
                    )
                })?;

            let elapsed = start_time.elapsed();
            info!("✅ {} responded in {:?}", model, elapsed);

            response.content
        };

        // Store step data with generated content
        let step_data = StepData {
            step_number: request.step_number,
            content: generated_content.clone(),
            is_revision: request.is_step_revision,
            revises_step: request.revises_step_number,
            branch_id: request.branch_id.clone(),
        };

        // Handle branching
        if request.is_branch_point && request.branch_id.is_some() {
            let branch_id = request.branch_id.as_ref().unwrap();
            self.branches
                .entry(branch_id.clone())
                .or_insert_with(Vec::new)
                .push(step_data.clone());
        }

        // Add to main history (revisions replace the original step)
        if request.is_step_revision && request.revises_step_number.is_some() {
            let revises_idx = request.revises_step_number.unwrap() as usize - 1;
            if revises_idx < self.step_history.len() {
                self.step_history[revises_idx] = step_data;
            }
        } else {
            self.step_history.push(step_data);
        }

        // Monitor for circular reasoning
        {
            let mut monitor_guard = monitor.lock();
            let signals =
                monitor_guard.analyze_thought(&generated_content, request.step_number as usize);

            if signals.circular_score > 0.7 {
                info!(
                    "Warning: Planning step {} may have circular reasoning",
                    request.step_number
                );
            }
        }

        // Update synthesis with this planning step
        {
            use lux_synthesis::events::SynthesisEvent;

            let synthesis = self.synthesis.lock().unwrap();

            // Update understanding
            let understanding = if request.step_number == 1 {
                format!("Starting to plan: {}", request.step)
            } else {
                format!(
                    "Step {}: {}",
                    request.step_number,
                    generated_content
                        .lines()
                        .next()
                        .unwrap_or(&generated_content)
                )
            };

            // Update confidence based on progress
            let progress = request.step_number as f32 / request.total_steps as f32;
            let confidence = 0.3 + (progress * 0.5);
            let clarity = 0.4 + (progress * 0.4);

            synthesis.apply(SynthesisEvent::Understanding {
                text: understanding,
                confidence: Some(confidence),
                clarity: Some(clarity),
            })?;

            // Extract insights from the planning step
            if generated_content.contains("key decision") || generated_content.contains("important")
            {
                synthesis.apply(SynthesisEvent::Insight(InsightEntry {
                    insight: format!(
                        "Planning insight from step {}: {}",
                        request.step_number,
                        generated_content
                            .lines()
                            .find(|l| l.contains("decision") || l.contains("important"))
                            .unwrap_or("Key planning decision made")
                    ),
                    confidence: 0.8,
                    source_step: request.step_number,
                    supported_by_evidence: true,
                }))?;
            }

            // Add action items if this is a concrete planning step
            if !request.next_step_required
                || generated_content.contains("implement")
                || generated_content.contains("action")
            {
                synthesis.apply(SynthesisEvent::Action(ActionItem {
                    action: format!("Execute planning step {}", request.step_number),
                    priority: if request.step_number <= 3 {
                        Priority::High
                    } else {
                        Priority::Medium
                    },
                    rationale: "Core planning step identified".to_string(),
                    dependencies: vec![],
                }))?;
            }

            // Mark step complete
            synthesis.apply(SynthesisEvent::StepComplete {
                step_number: request.step_number,
            })?;
        }

        // Build response with generated content
        let mut response = self.build_planning_response(&request, generated_content, &model);

        // Add synthesis snapshot to response
        {
            let synthesis = self.synthesis.lock().unwrap();
            let state = synthesis.snapshot();
            response.synthesis_snapshot = Some(SynthesisSnapshot {
                current_plan: state.current_understanding.clone(),
                key_decisions: state
                    .key_insights
                    .iter()
                    .map(|i| i.insight.clone())
                    .collect(),
                next_actions: state
                    .action_items
                    .iter()
                    .map(|a| a.action.clone())
                    .collect(),
                confidence_level: match (state.confidence_score * 100.0) as i32 {
                    0..=30 => "low".to_string(),
                    31..=60 => "medium".to_string(),
                    61..=85 => "high".to_string(),
                    _ => "very_high".to_string(),
                },
                ready_for_execution: state.confidence_score > 0.7,
            });
        }

        // Handle deep thinking pauses for complex plans
        if request.total_steps >= 5 && request.step_number <= 3 && request.next_step_required {
            response = self.add_deep_thinking_pause(response, &request);
        }

        // Handle plan completion
        if !request.next_step_required {
            response.planning_complete = Some(true);
            response.plan_summary = Some(self.generate_plan_summary(&request));
            response.next_steps = Some(
                "Planning complete. Present the complete plan to the user in a well-structured format with clear sections, \
                numbered steps, visual elements (ASCII charts/diagrams where helpful), sub-step breakdowns, and implementation guidance. \
                Use headings, bullet points, and visual organization to make the plan easy to follow. \
                If there are phases, dependencies, or parallel tracks, show these relationships visually. \
                IMPORTANT: Do NOT use emojis - use clear text formatting and ASCII characters only. \
                Do NOT mention time estimates or costs unless explicitly requested. \
                After presenting the plan, offer to either help implement specific parts or use the continuation_id to start related planning sessions.".to_string()
            );
        }

        Ok(response)
    }

    fn build_planning_context(&self, request: &PlannerRequest) -> String {
        let mut context = String::new();

        // Add previous steps
        context.push_str("Previous planning steps:\n");
        for (i, step) in self.step_history.iter().enumerate() {
            context.push_str(&format!("Step {}: {}\n", i + 1, step.content));
        }

        // Add branch information if relevant
        if request.is_branch_point {
            context.push_str("\nThis is a branch point - exploring an alternative approach.\n");
        }

        // Add revision context if relevant
        if request.is_step_revision && request.revises_step_number.is_some() {
            let revises_num = request.revises_step_number.unwrap();
            context.push_str(&format!(
                "\nThis step revises step {} based on new insights.\n",
                revises_num
            ));
        }

        context
    }

    fn build_system_prompt(&self, request: &PlannerRequest) -> String {
        let base_prompt =
            "You are an expert planner helping to create a comprehensive, step-by-step plan. \
            Your role is to generate ONE planning step that builds on previous steps. \
            Be specific, actionable, and maintain coherence with the overall planning trajectory.";

        if request.is_branch_point {
            format!("{}\n\nYou are exploring an ALTERNATIVE APPROACH. Think differently from the main path.", base_prompt)
        } else if request.is_step_revision {
            format!(
                "{}\n\nYou are REVISING a previous step. Improve upon it with new insights.",
                base_prompt
            )
        } else {
            base_prompt.to_string()
        }
    }

    fn build_user_prompt(&self, request: &PlannerRequest, context: &str) -> String {
        let step_type = if request.is_branch_point {
            "alternative approach"
        } else if request.is_step_revision {
            "revision"
        } else {
            "next step"
        };

        format!(
            "{}\n\nCurrent step number: {} of {}\n\
            Generate the {} in this planning process.\n\
            User guidance: {}\n\n\
            Provide a clear, actionable planning step that advances toward the goal.",
            context, request.step_number, request.total_steps, step_type, request.step
        )
    }

    fn build_planning_response(
        &self,
        request: &PlannerRequest,
        generated_content: String,
        model: &str,
    ) -> PlannerResponse {
        let metadata = PlannerMetadata {
            branches: self.branches.keys().cloned().collect(),
            step_history_length: self.step_history.len() as u32,
            is_step_revision: request.is_step_revision,
            revises_step_number: request.revises_step_number,
            is_branch_point: request.is_branch_point,
            branch_from_step: request.branch_from_step,
            branch_id: request.branch_id.clone(),
            more_steps_needed: request.more_steps_needed,
        };

        let status = if request.next_step_required {
            "pause_for_planner".to_string()
        } else {
            "planning_complete".to_string()
        };

        let next_steps = if request.next_step_required {
            let remaining_steps = request.total_steps - request.step_number;
            Some(format!(
                "Continue with step {}. Approximately {} steps remaining.",
                request.step_number + 1,
                remaining_steps
            ))
        } else {
            None
        };

        PlannerResponse {
            status,
            step_number: request.step_number,
            total_steps: request.total_steps,
            next_step_required: request.next_step_required,
            step_content: generated_content, // Use the generated content instead of request.step
            metadata,
            continuation_id: None, // Could be added if needed
            planning_complete: None,
            plan_summary: None,
            next_steps,
            thinking_required: None,
            required_thinking: None,
            planner_required: Some(true),
            model_used: Some(model.to_string()),
            synthesis_snapshot: None, // Will be set after building response
        }
    }

    fn add_deep_thinking_pause(
        &self,
        mut response: PlannerResponse,
        request: &PlannerRequest,
    ) -> PlannerResponse {
        response.status = "pause_for_deep_thinking".to_string();
        response.thinking_required = Some(true);

        let (required_thinking, next_steps) = match request.step_number {
            1 => {
                let thinking = vec![
                    "Think deeply about the complete scope and complexity of what needs to be planned".to_string(),
                    "Consider multiple approaches and their trade-offs".to_string(),
                    "Identify key constraints, dependencies, and potential challenges".to_string(),
                    "Think about stakeholders, success criteria, and critical requirements".to_string(),
                ];

                let steps = format!(
                    "MANDATORY: DO NOT call the planner tool again immediately. This is a complex plan ({} steps) \
                    that requires deep thinking. You MUST first spend time reflecting on the planning challenge:\n\n\
                    REQUIRED DEEP THINKING before calling planner step {}:\n\
                    1. Analyze the FULL SCOPE: What exactly needs to be accomplished?\n\
                    2. Consider MULTIPLE APPROACHES: What are 2-3 different ways to tackle this?\n\
                    3. Identify CONSTRAINTS & DEPENDENCIES: What limits our options?\n\
                    4. Think about SUCCESS CRITERIA: How will we know we've succeeded?\n\
                    5. Consider RISKS & MITIGATION: What could go wrong early vs late?\n\n\
                    Only call planner again with step_number: {} AFTER this deep analysis.",
                    request.total_steps, request.step_number + 1, request.step_number + 1
                );

                (thinking, steps)
            }
            2 => {
                let thinking = vec![
                    "Evaluate the approach from step 1 - are there better alternatives?"
                        .to_string(),
                    "Break down the major phases and identify critical decision points".to_string(),
                    "Consider resource requirements and potential bottlenecks".to_string(),
                    "Think about how different parts interconnect and affect each other"
                        .to_string(),
                ];

                let steps = format!(
                    "STOP! Complex planning requires reflection between steps. DO NOT call planner immediately.\n\n\
                    MANDATORY REFLECTION before planner step {}:\n\
                    1. EVALUATE YOUR APPROACH: Is the direction from step 1 still the best?\n\
                    2. IDENTIFY MAJOR PHASES: What are the 3-5 main chunks of work?\n\
                    3. SPOT DEPENDENCIES: What must happen before what?\n\
                    4. CONSIDER RESOURCES: What skills, tools, or access do we need?\n\
                    5. FIND CRITICAL PATHS: Where could delays hurt the most?\n\n\
                    Think deeply about these aspects, then call planner with step_number: {}.",
                    request.step_number + 1, request.step_number + 1
                );

                (thinking, steps)
            }
            3 => {
                let thinking = vec![
                    "Validate that the emerging plan addresses the original requirements"
                        .to_string(),
                    "Identify any gaps or assumptions that need clarification".to_string(),
                    "Consider how to validate progress and adjust course if needed".to_string(),
                    "Think about what the first concrete steps should be".to_string(),
                ];

                let steps = format!(
                    "PAUSE for final strategic reflection. DO NOT call planner yet.\n\n\
                    FINAL DEEP THINKING before planner step {}:\n\
                    1. VALIDATE COMPLETENESS: Does this plan address all original requirements?\n\
                    2. CHECK FOR GAPS: What assumptions need validation? What's unclear?\n\
                    3. PLAN FOR ADAPTATION: How will we know if we need to change course?\n\
                    4. DEFINE FIRST STEPS: What are the first 2-3 concrete actions?\n\
                    5. TRANSITION MINDSET: Ready to shift from strategic to tactical planning?\n\n\
                    After this reflection, call planner with step_number: {} to continue with tactical details.",
                    request.step_number + 1, request.step_number + 1
                );

                (thinking, steps)
            }
            _ => {
                // Should not reach here for deep thinking
                (vec![], "Continue planning...".to_string())
            }
        };

        response.required_thinking = Some(required_thinking);
        response.next_steps = Some(next_steps);

        response
    }

    fn generate_plan_summary(&self, request: &PlannerRequest) -> String {
        let mut summary = format!(
            "COMPLETE PLAN: {} (Total {} steps completed)\n\n",
            request.step, request.total_steps
        );

        // Add step history summary
        summary.push_str("PLANNING JOURNEY:\n");
        for (i, step) in self.step_history.iter().enumerate() {
            summary.push_str(&format!(
                "Step {}: {}\n",
                i + 1,
                if step.content.len() > 100 {
                    format!("{}...", &step.content[..100])
                } else {
                    step.content.clone()
                }
            ));

            if step.is_revision {
                summary.push_str(&format!(
                    "  (Revised step {})\n",
                    step.revises_step.unwrap_or(0)
                ));
            }
            if let Some(branch_id) = &step.branch_id {
                summary.push_str(&format!("  (Branch: {})\n", branch_id));
            }
        }

        // Add branch summary if any
        if !self.branches.is_empty() {
            summary.push_str("\nBRANCHES EXPLORED:\n");
            for (branch_id, steps) in &self.branches {
                summary.push_str(&format!("- {}: {} steps\n", branch_id, steps.len()));
            }
        }

        summary
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
        }
    }
}
