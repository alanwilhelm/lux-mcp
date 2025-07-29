use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::collections::HashSet;
use tracing::{debug, info, error, warn};

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
pub struct TracedReasoningRequest {
    pub thought: String,  // For thought 1: the query, for 2+: guidance/previous thought
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
    
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    #[serde(default)]
    pub guardrails: GuardrailConfig,
}

fn default_temperature() -> f32 { 0.7 }

#[derive(Debug, Serialize, Deserialize)]
pub struct GuardrailConfig {
    #[serde(default = "default_true")]
    pub semantic_drift_check: bool,
    #[serde(default = "default_semantic_drift_threshold")]
    pub semantic_drift_threshold: f32,
    #[serde(default = "default_true")]
    pub perplexity_monitoring: bool,
    #[serde(default = "default_perplexity_threshold")]
    pub perplexity_threshold: f32,
    #[serde(default = "default_true")]
    pub circular_reasoning_detection: bool,
    #[serde(default = "default_true")]
    pub consistency_validation: bool,
    #[serde(default = "default_true")]
    pub attention_entropy_analysis: bool,
}

fn default_true() -> bool { true }
fn default_semantic_drift_threshold() -> f32 { 0.3 }
fn default_perplexity_threshold() -> f32 { 50.0 }

impl Default for GuardrailConfig {
    fn default() -> Self {
        Self {
            semantic_drift_check: true,
            semantic_drift_threshold: 0.3,
            perplexity_monitoring: true,
            perplexity_threshold: 50.0,
            circular_reasoning_detection: true,
            consistency_validation: true,
            attention_entropy_analysis: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TracedReasoningResponse {
    pub status: String,  // "thinking", "intervention_needed", "conclusion_reached"
    pub thought_number: u32,
    pub total_thoughts: u32,
    pub next_thought_needed: bool,
    pub thought_content: String,
    pub thought_type: StepType,
    pub metrics: StepMetrics,
    pub metadata: TracedReasoningMetadata,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continuation_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_complete: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub final_answer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_steps: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intervention: Option<Intervention>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overall_metrics: Option<ReasoningMetrics>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_used: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TracedReasoningMetadata {
    pub thought_history_length: u32,
    pub interventions_count: u32,
    pub semantic_coherence: f32,
    pub current_confidence: f32,
    pub is_revision: bool,
    pub revises_thought: Option<u32>,
    pub branch_id: Option<String>,
    pub needs_more_thoughts: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReasoningStep {
    pub step_number: u32,
    pub thought: String,
    pub step_type: StepType,
    pub confidence: f32,
    pub metrics: StepMetrics,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum StepType {
    Initial,
    Exploration,
    Analysis,
    Synthesis,
    Validation,
    Conclusion,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StepMetrics {
    pub semantic_similarity: Option<f32>,
    pub perplexity: Option<f32>,
    pub attention_entropy: Option<f32>,
    pub consistency_score: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReasoningMetrics {
    pub total_steps: u32,
    pub average_confidence: f32,
    pub semantic_coherence: f32,
    pub reasoning_quality: f32,
    pub path_consensus: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Intervention {
    pub step: u32,
    pub intervention_type: InterventionType,
    pub description: String,
    pub severity: Severity,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum InterventionType {
    SemanticDrift,
    HighPerplexity,
    CircularReasoning,
    InconsistentLogic,
    AttentionScatter,
    HallucinationRisk,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct ThoughtData {
    pub thought_number: u32,
    pub content: String,
    pub thought_type: StepType,
    pub metrics: StepMetrics,
    pub confidence: f32,
    pub is_revision: bool,
    pub revises_thought: Option<u32>,
    pub branch_id: Option<String>,
}

pub struct TracedReasoningTool {
    session_manager: Arc<SessionManager>,
    openai_client: Option<Arc<dyn LLMClient>>,
    openrouter_clients: Vec<(String, Arc<dyn LLMClient>)>,
    model_resolver: ModelResolver,
    config: LLMConfig,
    thought_history: Vec<ThoughtData>,
    interventions: Vec<Intervention>,
    branches: std::collections::HashMap<String, Vec<ThoughtData>>,
    original_query: Option<String>,
}

impl TracedReasoningTool {
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
            thought_history: Vec::new(),
            interventions: Vec::new(),
            branches: std::collections::HashMap::new(),
            original_query: None,
        })
    }
    
    pub async fn process_thought(&mut self, request: TracedReasoningRequest) -> Result<TracedReasoningResponse> {
        let session_id = self.session_manager.get_or_create_session(request.session_id.clone());
        let monitor = self.session_manager.get_monitor(&session_id)?;
        
        // Validate thought number
        if request.thought_number < 1 {
            anyhow::bail!("thought_number must be at least 1");
        }
        
        if request.total_thoughts < 1 {
            anyhow::bail!("total_thoughts must be at least 1");
        }
        
        // Get model for reasoning
        let model = request.model
            .as_ref()
            .map(|m| self.model_resolver.resolve(m))
            .unwrap_or_else(|| self.config.default_reasoning_model.clone());
        
        info!("Processing thought {} with model '{}'", request.thought_number, model);
        
        // Warn about o3 model usage
        if model.starts_with("o3") {
            info!("⏳ Using {} for deep reasoning - this may take 30 seconds to 5 minutes", model);
            info!("💭 Metacognitive reasoning in progress...");
        }
        
        // For first thought, store the original query
        if request.thought_number == 1 {
            self.original_query = Some(request.thought.clone());
            // Reset state for new reasoning session
            self.thought_history.clear();
            self.interventions.clear();
            self.branches.clear();
            
            // Reset monitor
            let mut monitor_guard = monitor.lock();
            monitor_guard.reset_session();
        }
        
        // Generate thought content using LLM
        let (generated_content, thought_type) = if request.thought_number == 1 {
            // For first thought, acknowledge the query and begin exploration
            let initial_response = format!(
                "Beginning analysis of: {}\n\nLet me explore this step by step.",
                request.thought
            );
            (initial_response, StepType::Initial)
        } else {
            // Build context from previous thoughts
            let context = self.build_reasoning_context(&request);
            
            // Create prompt for LLM
            let system_prompt = self.build_system_prompt(&request.guardrails);
            let user_prompt = self.build_user_prompt(&request, &context);
            
            // Create messages for LLM
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
            
            // Call LLM with fallback logic
            let (response_content, actual_model) = self.call_llm_with_fallback(
                &model, 
                messages, 
                request.temperature, 
                request.thought_number
            ).await?;
            
            let (step_type, _, _) = self.parse_step_response(&response_content);
            (response_content, step_type)
        };
        
        // Calculate metrics for this thought
        let step_metrics = self.calculate_step_metrics(
            &generated_content,
            self.original_query.as_ref().unwrap_or(&"".to_string()),
            &self.thought_history,
            &request.guardrails,
            monitor.clone(),
        ).await?;
        
        let confidence = self.calculate_step_confidence(&step_metrics);
        
        // Check for interventions
        let intervention = self.check_thought_interventions(
            request.thought_number,
            &step_metrics,
            &generated_content,
            &request.guardrails,
            monitor.clone(),
        );
        
        if let Some(ref interv) = intervention {
            self.interventions.push(interv.clone());
        }
        
        // Store thought data
        let thought_data = ThoughtData {
            thought_number: request.thought_number,
            content: generated_content.clone(),
            thought_type: thought_type.clone(),
            metrics: step_metrics.clone(),
            confidence,
            is_revision: request.is_revision,
            revises_thought: request.revises_thought,
            branch_id: request.branch_id.clone(),
        };
        
        // Handle branching
        if request.branch_from_thought.is_some() && request.branch_id.is_some() {
            let branch_id = request.branch_id.as_ref().unwrap();
            self.branches.entry(branch_id.clone())
                .or_insert_with(Vec::new)
                .push(thought_data.clone());
        }
        
        // Add to main history (revisions replace the original thought)
        if request.is_revision && request.revises_thought.is_some() {
            let revises_idx = request.revises_thought.unwrap() as usize - 1;
            if revises_idx < self.thought_history.len() {
                self.thought_history[revises_idx] = thought_data;
            }
        } else {
            self.thought_history.push(thought_data);
        }
        
        // Build response
        let mut response = self.build_reasoning_response(&request, generated_content, thought_type.clone(), step_metrics, confidence, &model);
        
        // Set intervention if needed
        response.intervention = intervention;
        
        // Check if we've reached a conclusion
        let is_conclusion = thought_type == StepType::Conclusion || 
                           response.thought_content.to_lowercase().contains("final answer") ||
                           response.thought_content.to_lowercase().contains("conclusion");
        
        // Handle reasoning completion
        if !request.next_thought_needed || is_conclusion {
            response.reasoning_complete = Some(true);
            response.final_answer = Some(self.extract_final_answer(&response.thought_content));
            response.overall_metrics = Some(self.calculate_overall_metrics(&self.thought_history));
            response.next_steps = Some(
                "Reasoning complete. Present the final answer and reasoning chain to the user with:\n\
                1. Clear conclusion based on the reasoning\n\
                2. Summary of key insights from each thought\n\
                3. Confidence assessment\n\
                4. Any caveats or limitations identified".to_string()
            );
        }
        
        Ok(response)
    }
    
    fn build_reasoning_context(&self, request: &TracedReasoningRequest) -> String {
        let mut context = String::new();
        
        // Add original query
        if let Some(ref query) = self.original_query {
            context.push_str(&format!("Original Query: {}\n\n", query));
        }
        
        // Add previous thoughts
        context.push_str("Previous reasoning thoughts:\n");
        for thought in &self.thought_history {
            context.push_str(&format!(
                "Thought {}: [Type: {:?}, Confidence: {:.2}]\n{}\n\n",
                thought.thought_number,
                thought.thought_type,
                thought.confidence,
                thought.content
            ));
        }
        
        // Add intervention history if any
        if !self.interventions.is_empty() {
            context.push_str("\nInterventions triggered:\n");
            for intervention in &self.interventions {
                context.push_str(&format!(
                    "- Thought {}: {:?} - {}\n",
                    intervention.step,
                    intervention.intervention_type,
                    intervention.description
                ));
            }
        }
        
        // Add branch information if relevant
        if request.branch_from_thought.is_some() {
            context.push_str("\nThis is a branch exploring an alternative reasoning path.\n");
        }
        
        // Add revision context if relevant
        if request.is_revision && request.revises_thought.is_some() {
            let revises_num = request.revises_thought.unwrap();
            context.push_str(&format!("\nThis thought revises thought {} based on new insights.\n", revises_num));
        }
        
        context
    }
    
    fn build_user_prompt(&self, request: &TracedReasoningRequest, context: &str) -> String {
        let thought_type = if request.thought_number <= 2 {
            "exploration"
        } else if request.thought_number > request.total_thoughts - 2 {
            "synthesis/conclusion"
        } else {
            "analysis"
        };
        
        format!(
            "{}Current thought number: {} of {}\n\
            Generate the next {} thought in this reasoning process.\n\
            User guidance: {}\n\n\
            Provide a clear reasoning thought that:\n\
            1. Builds on previous insights\n\
            2. Adds new analysis or perspective\n\
            3. Moves toward answering the original query\n\
            4. Maintains logical consistency\n\n\
            Format: Step {}: [Type: exploration/analysis/synthesis/validation/conclusion]\n\
            [Your detailed reasoning for this step]",
            context,
            request.thought_number,
            request.total_thoughts,
            thought_type,
            request.thought,
            request.thought_number
        )
    }
    
    fn check_thought_interventions(
        &self,
        thought_number: u32,
        metrics: &StepMetrics,
        thought_content: &str,
        guardrails: &GuardrailConfig,
        monitor: Arc<parking_lot::Mutex<crate::monitoring::MetacognitiveMonitor>>,
    ) -> Option<Intervention> {
        // Check monitor signals
        {
            let mut monitor_guard = monitor.lock();
            let signals = monitor_guard.analyze_thought(thought_content, thought_number as usize);
            
            if let Some(intervention_msg) = signals.intervention {
                let intervention_type = if signals.circular_score > 0.85 {
                    InterventionType::CircularReasoning
                } else if signals.distractor_alert {
                    InterventionType::SemanticDrift
                } else {
                    InterventionType::InconsistentLogic
                };
                
                return Some(Intervention {
                    step: thought_number,
                    intervention_type,
                    description: intervention_msg,
                    severity: Severity::Medium,
                });
            }
        }
        
        // Semantic drift check
        if guardrails.semantic_drift_check {
            if let Some(similarity) = metrics.semantic_similarity {
                if similarity < guardrails.semantic_drift_threshold {
                    return Some(Intervention {
                        step: thought_number,
                        intervention_type: InterventionType::SemanticDrift,
                        description: format!(
                            "Reasoning drifting from original query (similarity: {:.2})",
                            similarity
                        ),
                        severity: if similarity < 0.2 { Severity::High } else { Severity::Medium },
                    });
                }
            }
        }
        
        // Perplexity check
        if guardrails.perplexity_monitoring {
            if let Some(perplexity) = metrics.perplexity {
                if perplexity > guardrails.perplexity_threshold {
                    return Some(Intervention {
                        step: thought_number,
                        intervention_type: InterventionType::HighPerplexity,
                        description: format!("High perplexity detected: {:.1}", perplexity),
                        severity: if perplexity > 80.0 { Severity::High } else { Severity::Medium },
                    });
                }
            }
        }
        
        // Circular reasoning check
        if guardrails.circular_reasoning_detection && self.thought_history.len() > 2 {
            for prev_thought in self.thought_history.iter().rev().take(3) {
                if self.text_similarity(thought_content, &prev_thought.content) > 0.85 {
                    return Some(Intervention {
                        step: thought_number,
                        intervention_type: InterventionType::CircularReasoning,
                        description: "Potential circular reasoning detected".to_string(),
                        severity: Severity::Medium,
                    });
                }
            }
        }
        
        None
    }
    
    fn build_reasoning_response(
        &self,
        request: &TracedReasoningRequest,
        generated_content: String,
        thought_type: StepType,
        metrics: StepMetrics,
        confidence: f32,
        model: &str,
    ) -> TracedReasoningResponse {
        let metadata = TracedReasoningMetadata {
            thought_history_length: self.thought_history.len() as u32,
            interventions_count: self.interventions.len() as u32,
            semantic_coherence: metrics.semantic_similarity.unwrap_or(1.0),
            current_confidence: confidence,
            is_revision: request.is_revision,
            revises_thought: request.revises_thought,
            branch_id: request.branch_id.clone(),
            needs_more_thoughts: request.needs_more_thoughts,
        };
        
        let status = if request.next_thought_needed {
            "thinking".to_string()
        } else {
            "conclusion_reached".to_string()
        };
        
        let next_steps = if request.next_thought_needed {
            let remaining = request.total_thoughts - request.thought_number;
            Some(format!(
                "Continue with thought {}. Approximately {} thoughts remaining.",
                request.thought_number + 1, remaining
            ))
        } else {
            None
        };
        
        TracedReasoningResponse {
            status,
            thought_number: request.thought_number,
            total_thoughts: request.total_thoughts,
            next_thought_needed: request.next_thought_needed,
            thought_content: generated_content,
            thought_type,
            metrics,
            metadata,
            continuation_id: None,
            reasoning_complete: None,
            final_answer: None,
            next_steps,
            intervention: None,
            overall_metrics: None,
            model_used: Some(model.to_string()),
        }
    }
    
    fn build_system_prompt(&self, guardrails: &GuardrailConfig) -> String {
        let mut prompt = String::from(
            "You are a reasoning assistant that thinks step-by-step through problems. \
            Structure your responses clearly with explicit reasoning steps.\n\n"
        );
        
        if guardrails.semantic_drift_check {
            prompt.push_str("- Stay focused on the original query throughout your reasoning\n");
        }
        if guardrails.circular_reasoning_detection {
            prompt.push_str("- Avoid circular reasoning and ensure each step adds new insight\n");
        }
        if guardrails.consistency_validation {
            prompt.push_str("- Maintain logical consistency across all reasoning steps\n");
        }
        
        prompt.push_str("\nFormat each step as:\nStep N: [Type: exploration/analysis/synthesis/validation/conclusion]\n[Your reasoning for this step]\n");
        
        prompt
    }
    
    fn parse_step_response(&self, content: &str) -> (StepType, String, bool) {
        let content_lower = content.to_lowercase();
        
        let step_type = if content_lower.contains("exploration") || content_lower.contains("exploring") {
            StepType::Exploration
        } else if content_lower.contains("analysis") || content_lower.contains("analyzing") {
            StepType::Analysis
        } else if content_lower.contains("synthesis") || content_lower.contains("combining") {
            StepType::Synthesis
        } else if content_lower.contains("validation") || content_lower.contains("checking") {
            StepType::Validation
        } else if content_lower.contains("conclusion") || content_lower.contains("final answer") {
            StepType::Conclusion
        } else {
            StepType::Analysis
        };
        
        let is_final = content_lower.contains("final answer") || 
                      content_lower.contains("conclusion") ||
                      content_lower.contains("therefore, the answer");
        
        (step_type, content.to_string(), is_final)
    }
    
    async fn calculate_step_metrics(
        &self,
        thought: &str,
        _original_query: &str,
        previous_thoughts: &[ThoughtData],
        _guardrails: &GuardrailConfig,
        monitor: Arc<parking_lot::Mutex<crate::monitoring::MetacognitiveMonitor>>,
    ) -> Result<StepMetrics> {
        // Use real monitor to analyze the thought
        let mut monitor_guard = monitor.lock();
        let thought_number = previous_thoughts.len() + 1;
        let signals = monitor_guard.analyze_thought(thought, thought_number);
        
        // Map MonitoringSignals to StepMetrics
        // circular_score of 0 means no circular reasoning (good), 1 means high circular (bad)
        // So semantic_similarity should be 1.0 - circular_score
        let semantic_similarity = Some((1.0 - signals.circular_score) as f32);
        
        // Monitor doesn't calculate perplexity yet, keep placeholder
        let perplexity = Some(20.0 + (thought.len() as f32 / 100.0));
        
        // Monitor doesn't calculate attention entropy yet
        let attention_entropy = Some(0.7);
        
        // Use quality trend to estimate consistency
        let consistency_score = match signals.quality_trend.as_str() {
            "improving" => Some(1.0),
            "stable" => Some(0.9),
            "degrading" => Some(0.7),
            _ => Some(0.8),
        };
        
        Ok(StepMetrics {
            semantic_similarity,
            perplexity,
            attention_entropy,
            consistency_score,
        })
    }
    
    fn calculate_step_confidence(&self, metrics: &StepMetrics) -> f32 {
        let mut confidence = 1.0;
        
        if let Some(similarity) = metrics.semantic_similarity {
            confidence *= similarity;
        }
        
        if let Some(perplexity) = metrics.perplexity {
            confidence *= (50.0 - perplexity.min(50.0)) / 50.0;
        }
        
        if let Some(consistency) = metrics.consistency_score {
            confidence *= consistency;
        }
        
        confidence.max(0.1).min(1.0)
    }
    
    fn check_interventions(
        &self,
        step: u32,
        metrics: &StepMetrics,
        thought: &str,
        previous_steps: &[ReasoningStep],
        guardrails: &GuardrailConfig,
    ) -> Option<Intervention> {
        // Semantic drift check
        if guardrails.semantic_drift_check {
            if let Some(similarity) = metrics.semantic_similarity {
                if similarity < guardrails.semantic_drift_threshold {
                    return Some(Intervention {
                        step,
                        intervention_type: InterventionType::SemanticDrift,
                        description: format!(
                            "Reasoning drifting from original query (similarity: {:.2})",
                            similarity
                        ),
                        severity: if similarity < 0.2 { Severity::High } else { Severity::Medium },
                    });
                }
            }
        }
        
        // Perplexity check
        if guardrails.perplexity_monitoring {
            if let Some(perplexity) = metrics.perplexity {
                if perplexity > guardrails.perplexity_threshold {
                    return Some(Intervention {
                        step,
                        intervention_type: InterventionType::HighPerplexity,
                        description: format!("High perplexity detected: {:.1}", perplexity),
                        severity: if perplexity > 80.0 { Severity::High } else { Severity::Medium },
                    });
                }
            }
        }
        
        // Circular reasoning check
        if guardrails.circular_reasoning_detection && previous_steps.len() > 2 {
            for prev_step in previous_steps.iter().rev().take(3) {
                if self.text_similarity(thought, &prev_step.thought) > 0.85 {
                    return Some(Intervention {
                        step,
                        intervention_type: InterventionType::CircularReasoning,
                        description: "Potential circular reasoning detected".to_string(),
                        severity: Severity::Medium,
                    });
                }
            }
        }
        
        None
    }
    
    fn text_similarity(&self, text1: &str, text2: &str) -> f32 {
        // Simple word overlap similarity
        let words1: std::collections::HashSet<_> = text1.split_whitespace().collect();
        let words2: std::collections::HashSet<_> = text2.split_whitespace().collect();
        let intersection = words1.intersection(&words2).count();
        let union = words1.union(&words2).count();
        
        if union == 0 { 0.0 } else { intersection as f32 / union as f32 }
    }
    
    fn extract_final_answer(&self, content: &str) -> String {
        if let Some(idx) = content.find("Final Answer:") {
            content[idx + 13..].trim().to_string()
        } else if let Some(idx) = content.find("Therefore") {
            content[idx..].trim().to_string()
        } else {
            content.trim().to_string()
        }
    }
    
    fn calculate_overall_metrics(&self, thoughts: &[ThoughtData]) -> ReasoningMetrics {
        let total_steps = thoughts.len() as u32;
        let average_confidence = if thoughts.is_empty() {
            0.0
        } else {
            thoughts.iter().map(|t| t.confidence).sum::<f32>() / thoughts.len() as f32
        };
        
        let semantic_coherence = thoughts
            .iter()
            .filter_map(|t| t.metrics.semantic_similarity)
            .fold(0.0, |acc, x| acc + x) / thoughts.len().max(1) as f32;
        
        let reasoning_quality = average_confidence * semantic_coherence;
        
        ReasoningMetrics {
            total_steps,
            average_confidence,
            semantic_coherence,
            reasoning_quality,
            path_consensus: None, // Would require multiple reasoning paths
        }
    }
    
    fn calculate_final_confidence(
        &self,
        metrics: &ReasoningMetrics,
        interventions: &[Intervention],
    ) -> f32 {
        let base_confidence = metrics.reasoning_quality;
        
        let intervention_penalty = interventions.iter().map(|i| match i.severity {
            Severity::Low => 0.05,
            Severity::Medium => 0.1,
            Severity::High => 0.2,
            Severity::Critical => 0.4,
        }).sum::<f32>();
        
        (base_confidence - intervention_penalty).max(0.1).min(1.0)
    }
    
    async fn call_llm_with_fallback(
        &self, 
        requested_model: &str,
        messages: Vec<ChatMessage>,
        temperature: f32,
        thought_number: u32,
    ) -> Result<(String, String)> {
        info!("🚀 Sending thought {} to LLM for reasoning", thought_number);
        let start_time = std::time::Instant::now();
        
        // Define fallback models
        let fallback_models = self.get_fallback_models(requested_model);
        
        // Try requested model and fallbacks
        let mut last_error = None;
        for (attempt, model) in std::iter::once(requested_model.to_string())
            .chain(fallback_models.iter().cloned())
            .enumerate() 
        {
            if attempt > 0 {
                info!("🔄 Attempting fallback model: {} (attempt {})", model, attempt + 1);
            }
            
            match self.try_llm_call(&model, messages.clone(), temperature).await {
                Ok(response) => {
                    let elapsed = start_time.elapsed();
                    if attempt > 0 {
                        info!("✅ Thought {} generated using fallback model {} in {:?}", 
                            thought_number, model, elapsed);
                    } else {
                        info!("✅ Thought {} generated in {:?}", thought_number, elapsed);
                    }
                    return Ok((response.content, model));
                }
                Err(e) => {
                    let error_str = e.to_string();
                    if error_str.contains("does not exist") || 
                       error_str.contains("do not have access") ||
                       error_str.contains("model_not_found") {
                        warn!("❌ Model '{}' not available: {}", model, error_str);
                        last_error = Some(e);
                        continue;
                    } else {
                        // Non-recoverable error, return immediately
                        let elapsed = start_time.elapsed();
                        error!("LLM call failed after {:?}: {}", elapsed, e);
                        return Err(anyhow::anyhow!("Failed to generate reasoning thought after {:?}: {}", elapsed, e));
                    }
                }
            }
        }
        
        // All models failed
        Err(last_error.unwrap_or_else(|| {
            anyhow::anyhow!("All models failed. Requested: {}, tried fallbacks: {:?}", 
                requested_model, fallback_models)
        }))
    }
    
    async fn try_llm_call(
        &self,
        model: &str,
        messages: Vec<ChatMessage>,
        temperature: f32,
    ) -> Result<crate::llm::client::LLMResponse> {
        let client = self.get_client_for_model(model)
            .context("Failed to get LLM client")?;
        
        client.complete(messages, Some(temperature), Some(10000)).await
    }
    
    fn get_fallback_models(&self, requested_model: &str) -> Vec<String> {
        let mut fallbacks = Vec::new();
        
        // If a Claude model was requested, try other Claude variants
        if requested_model.contains("claude") {
            fallbacks.push("anthropic/claude-3.5-sonnet".to_string());
            fallbacks.push("anthropic/claude-3-opus".to_string());
            fallbacks.push("anthropic/claude-3-sonnet".to_string());
        }
        
        // Add default reasoning model as fallback if not already requested
        if requested_model != self.config.default_reasoning_model {
            fallbacks.push(self.config.default_reasoning_model.clone());
        }
        
        // Add o3 as fallback for reasoning tasks
        if !requested_model.starts_with("o3") {
            fallbacks.push("o3".to_string());
        }
        
        // Add some reliable fallbacks
        if !requested_model.contains("gpt-4") {
            fallbacks.push("gpt-4o-mini".to_string());
        }
        if !requested_model.contains("gemini") {
            fallbacks.push("google/gemini-2.5-pro".to_string());
        }
        
        // Remove duplicates while preserving order
        let mut seen = HashSet::new();
        fallbacks.retain(|model| seen.insert(model.clone()));
        
        fallbacks
    }
    
    fn get_client_for_model(&self, model: &str) -> Result<Arc<dyn LLMClient>> {
        debug!("Getting client for model: {}", model);
        debug!("OpenAI key available: {}", self.config.openai_api_key.is_some());
        debug!("OpenRouter key available: {}", self.config.openrouter_api_key.is_some());
        
        if self.model_resolver.is_openrouter_model(model) {
            info!("Using OpenRouter for model: {}", model);
            if self.config.openrouter_api_key.is_none() {
                error!("OpenRouter API key not configured for model: {}", model);
                anyhow::bail!("OpenRouter API key not configured");
            }
            
            if let Some((_, client)) = self.openrouter_clients
                .iter()
                .find(|(m, _)| m == model) {
                debug!("Found pre-created OpenRouter client for model: {}", model);
                Ok(client.clone())
            } else {
                debug!("Creating new OpenRouter client for model: {}", model);
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
            info!("Using OpenAI for model: {}", model);
            if self.openai_client.is_some() {
                if let Some(api_key) = &self.config.openai_api_key {
                    debug!("Creating OpenAI client for model: {}", model);
                    let new_client = OpenAIClient::new(
                        api_key.clone(),
                        model.to_string(),
                        self.config.openai_base_url.clone(),
                    )?;
                    Ok(Arc::new(new_client) as Arc<dyn LLMClient>)
                } else {
                    error!("OpenAI API key not configured but client exists");
                    anyhow::bail!("OpenAI API key not configured");
                }
            } else {
                error!("OpenAI client not initialized - API key missing");
                anyhow::bail!("OpenAI API key not configured");
            }
        }
    }
}