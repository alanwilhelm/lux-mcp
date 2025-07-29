use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, info, error};

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
    pub query: String,
    #[serde(default)]
    pub primary_model: Option<String>,
    #[serde(default)]
    pub verifier_model: Option<String>,
    #[serde(default = "default_max_steps")]
    pub max_steps: u32,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    #[serde(default)]
    pub bias_config: BiasCheckConfig,
    #[serde(default)]
    pub session_id: Option<String>,
}

fn default_max_steps() -> u32 { 10 }
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
    pub final_answer: String,
    pub reasoning_steps: Vec<VerifiedReasoningStep>,
    pub overall_assessment: OverallAssessment,
    pub primary_model_used: String,
    pub verifier_model_used: String,
    pub detailed_process_log: Vec<ProcessLogEntry>,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct OverallAssessment {
    pub total_steps: u32,
    pub biased_steps: u32,
    pub corrected_steps: u32,
    pub average_quality: f32,
    pub most_common_biases: Vec<BiasType>,
    pub final_quality_assessment: String,
}

pub struct BiasedReasoningTool {
    session_manager: Arc<SessionManager>,
    openai_client: Option<Arc<dyn LLMClient>>,
    openrouter_clients: Vec<(String, Arc<dyn LLMClient>)>,
    model_resolver: ModelResolver,
    config: LLMConfig,
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
        })
    }
    
    pub async fn biased_reasoning(&self, request: BiasedReasoningRequest) -> Result<BiasedReasoningResponse> {
        use chrono::Utc;
        
        // Get or create session
        let session_id = self.session_manager.get_or_create_session(request.session_id);
        let monitor = self.session_manager.get_monitor(&session_id)?;
        
        // Reset monitor for new request
        {
            let mut monitor_guard = monitor.lock();
            monitor_guard.reset_session();
        }
        
        // ALWAYS use configured defaults for biased_reasoning
        // This ensures consistent behavior regardless of request parameters
        let primary_model = self.model_resolver.resolve(&self.config.default_reasoning_model);
        let verifier_model = self.model_resolver.resolve(&self.config.default_bias_checker_model);
        
        // Log if user tried to override models
        if request.primary_model.is_some() || request.verifier_model.is_some() {
            info!("Note: biased_reasoning always uses configured defaults (primary: {}, verifier: {})", 
                  primary_model, verifier_model);
        }
        
        info!("Starting biased reasoning with primary: '{}', verifier: '{}'", primary_model, verifier_model);
        
        // Warn about o3 model usage
        if primary_model.starts_with("o3") || verifier_model.starts_with("o3") {
            info!("⏳ Using o3 models - expect longer processing times (30s-5min per step)");
            info!("💭 Deep reasoning in progress. This is normal and expected...");
        }
        
        // Get clients
        let primary_client = self.get_client_for_model(&primary_model)?;
        let verifier_client = self.get_client_for_model(&verifier_model)?;
        
        let mut reasoning_steps = Vec::new();
        let mut detailed_process_log = Vec::new();
        let mut primary_conversation = vec![
            ChatMessage {
                role: Role::System,
                content: "You are a reasoning assistant. Think through problems step-by-step, showing your thinking clearly.".to_string(),
            },
            ChatMessage {
                role: Role::User,
                content: format!(
                    "Query: {}\n\nPlease reason through this step-by-step.",
                    request.query
                ),
            },
        ];
        
        let mut step_count = 0;
        let mut final_answer = String::new();
        let mut bias_counts: std::collections::HashMap<BiasType, u32> = std::collections::HashMap::new();
        
        while step_count < request.max_steps {
            step_count += 1;
            
            // Get primary model's reasoning step
            info!("🔄 Step {}: Generating reasoning with {}", step_count, primary_model);
            let primary_start = Instant::now();
            // Use maximum tokens for all models to ensure no truncation
            let primary_max_tokens = 10000;
            
            let primary_response = primary_client
                .complete(
                    primary_conversation.clone(),
                    Some(request.temperature),
                    Some(primary_max_tokens),
                )
                .await
                .context("Failed to get primary reasoning step")?;
            let primary_duration = primary_start.elapsed();
            info!("✅ {} completed step {} in {:?}", primary_model, step_count, primary_duration);
            
            let primary_thought = primary_response.content.clone();
            
            // Log primary reasoning
            detailed_process_log.push(ProcessLogEntry {
                action_type: ProcessActionType::PrimaryReasoning,
                step_number: step_count,
                timestamp: Utc::now().to_rfc3339(),
                model_used: primary_model.clone(),
                content: format!("Generated reasoning step:\n{}", primary_thought),
                duration_ms: Some(primary_duration.as_millis() as u64),
            });
            
            // Check for bias using verifier model
            info!("🔍 Step {}: Checking for bias with {}", step_count, verifier_model);
            let bias_start = Instant::now();
            let bias_check = self.check_step_for_bias(
                &primary_thought,
                &request.query,
                step_count,
                &verifier_client,
                &request.bias_config,
                &verifier_model,
            ).await?;
            let bias_duration = bias_start.elapsed();
            info!("✅ Bias check completed in {:?}", bias_duration);
            
            // Log bias checking
            detailed_process_log.push(ProcessLogEntry {
                action_type: ProcessActionType::BiasChecking,
                step_number: step_count,
                timestamp: Utc::now().to_rfc3339(),
                model_used: verifier_model.clone(),
                content: format!(
                    "Bias check results:\n- Has bias: {}\n- Bias types: {:?}\n- Severity: {:?}\n- Explanation: {}\n- Suggestions: {}",
                    bias_check.has_bias,
                    bias_check.bias_types,
                    bias_check.severity,
                    bias_check.explanation,
                    bias_check.suggestions.join(", ")
                ),
                duration_ms: Some(bias_duration.as_millis() as u64),
            });
            
            // Track bias types
            for bias_type in &bias_check.bias_types {
                *bias_counts.entry(bias_type.clone()).or_insert(0) += 1;
            }
            
            // Generate corrected thought if needed
            let corrected_thought = if bias_check.has_bias && bias_check.severity as u8 >= Severity::Medium as u8 {
                let correction_start = Instant::now();
                let corrected = self.generate_corrected_thought(
                    &primary_thought,
                    &bias_check,
                    &verifier_client,
                ).await?;
                let correction_duration = correction_start.elapsed();
                
                // Log correction generation
                detailed_process_log.push(ProcessLogEntry {
                    action_type: ProcessActionType::CorrectionGeneration,
                    step_number: step_count,
                    timestamp: Utc::now().to_rfc3339(),
                    model_used: verifier_model.clone(),
                    content: format!("Generated corrected thought:\n{}", corrected),
                    duration_ms: Some(correction_duration.as_millis() as u64),
                });
                
                Some(corrected)
            } else {
                None
            };
            
            let step_quality = self.calculate_step_quality(&bias_check);
            
            // Log quality assessment
            detailed_process_log.push(ProcessLogEntry {
                action_type: ProcessActionType::QualityAssessment,
                step_number: step_count,
                timestamp: Utc::now().to_rfc3339(),
                model_used: "internal".to_string(),
                content: format!("Step quality score: {:.2}", step_quality),
                duration_ms: None,
            });
            
            reasoning_steps.push(VerifiedReasoningStep {
                step_number: step_count,
                primary_thought: primary_thought.clone(),
                bias_check,
                corrected_thought: corrected_thought.clone(),
                step_quality,
            });
            
            // Add to conversation (use corrected thought if available)
            let thought_to_continue = corrected_thought.as_ref().unwrap_or(&primary_thought);
            primary_conversation.push(ChatMessage {
                role: Role::Assistant,
                content: thought_to_continue.clone(),
            });
            
            // Check if we have a final answer
            if self.is_final_answer(&primary_thought) {
                final_answer = self.extract_final_answer(&primary_thought);
                break;
            }
            
            // Continue reasoning
            primary_conversation.push(ChatMessage {
                role: Role::User,
                content: "Continue your reasoning to the next step.".to_string(),
            });
        }
        
        // If no final answer yet, request one
        if final_answer.is_empty() {
            primary_conversation.push(ChatMessage {
                role: Role::User,
                content: "Based on your reasoning, provide a final answer.".to_string(),
            });
            
            let final_start = Instant::now();
            // Use maximum tokens for all models to ensure no truncation
            let final_max_tokens = 10000;
            
            let final_response = primary_client
                .complete(primary_conversation, Some(request.temperature), Some(final_max_tokens))
                .await?;
            let final_duration = final_start.elapsed();
                
            final_answer = final_response.content;
            
            // Log final answer generation
            detailed_process_log.push(ProcessLogEntry {
                action_type: ProcessActionType::FinalAnswerGeneration,
                step_number: step_count + 1,
                timestamp: Utc::now().to_rfc3339(),
                model_used: primary_model.clone(),
                content: format!("Generated final answer:\n{}", final_answer),
                duration_ms: Some(final_duration.as_millis() as u64),
            });
        }
        
        // Calculate overall assessment
        let overall_assessment = self.calculate_overall_assessment(&reasoning_steps, bias_counts);
        
        Ok(BiasedReasoningResponse {
            final_answer,
            reasoning_steps,
            overall_assessment,
            primary_model_used: primary_model,
            verifier_model_used: verifier_model,
            detailed_process_log,
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
        
        // Use maximum tokens for all models to ensure no truncation
        let max_tokens = 10000;
        
        // o4 models only support default temperature (1.0)
        let temperature = if verifier_model_name.starts_with("o4") {
            None  // Use default temperature for o4 models
        } else {
            Some(0.3)  // Use lower temperature for other models for consistency
        };
        
        let response = verifier_client
            .complete(messages, temperature, Some(max_tokens))
            .await
            .map_err(|e| {
                error!("Verifier model '{}' failed during bias check: {}", verifier_model_name, e);
                anyhow::anyhow!("Failed to check for bias with model '{}': {}", verifier_model_name, e)
            })?;
        
        // Parse the response into structured format
        self.parse_bias_check_response(&response.content)
    }
    
    fn parse_bias_check_response(&self, content: &str) -> Result<BiasCheckResult> {
        // Simple parsing - in production would use more sophisticated parsing
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
        
        // Extract suggestions (simplified)
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
    
    async fn generate_corrected_thought(
        &self,
        original_thought: &str,
        bias_check: &BiasCheckResult,
        verifier_client: &Arc<dyn LLMClient>,
    ) -> Result<String> {
        let correction_prompt = format!(
            "The following reasoning step has been identified as biased:\n\n\
            Original: {}\n\n\
            Issues found: {:?}\n\
            Explanation: {}\n\n\
            Please provide a corrected version that addresses these biases while maintaining the logical flow.",
            original_thought,
            bias_check.bias_types,
            bias_check.explanation
        );
        
        let messages = vec![
            ChatMessage {
                role: Role::System,
                content: "You are a reasoning assistant who corrects biased thinking.".to_string(),
            },
            ChatMessage {
                role: Role::User,
                content: correction_prompt,
            },
        ];
        
        // Use maximum tokens for all models to ensure no truncation
        let verifier_model = verifier_client.get_model_name();
        let max_tokens = 10000;
        
        // o4 models only support default temperature (1.0)
        let temperature = if verifier_model.starts_with("o4") {
            None  // Use default temperature for o4 models
        } else {
            Some(0.5)  // Use moderate temperature for corrections
        };
        
        let response = verifier_client
            .complete(messages, temperature, Some(max_tokens))
            .await
            .context("Failed to generate corrected thought")?;
        
        Ok(response.content)
    }
    
    fn calculate_step_quality(&self, bias_check: &BiasCheckResult) -> f32 {
        match bias_check.severity {
            Severity::None => 1.0,
            Severity::Low => 0.8,
            Severity::Medium => 0.6,
            Severity::High => 0.4,
            Severity::Critical => 0.2,
        }
    }
    
    fn is_final_answer(&self, content: &str) -> bool {
        let content_lower = content.to_lowercase();
        content_lower.contains("final answer") || 
        content_lower.contains("conclusion") ||
        content_lower.contains("therefore, the answer")
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
    
    fn calculate_overall_assessment(
        &self,
        steps: &[VerifiedReasoningStep],
        bias_counts: std::collections::HashMap<BiasType, u32>,
    ) -> OverallAssessment {
        let total_steps = steps.len() as u32;
        let biased_steps = steps.iter().filter(|s| s.bias_check.has_bias).count() as u32;
        let corrected_steps = steps.iter().filter(|s| s.corrected_thought.is_some()).count() as u32;
        
        let average_quality = if steps.is_empty() {
            0.0
        } else {
            steps.iter().map(|s| s.step_quality).sum::<f32>() / steps.len() as f32
        };
        
        let mut most_common_biases: Vec<(BiasType, u32)> = bias_counts.into_iter().collect();
        most_common_biases.sort_by(|a, b| b.1.cmp(&a.1));
        let most_common_biases: Vec<BiasType> = most_common_biases
            .into_iter()
            .take(3)
            .map(|(bias_type, _)| bias_type)
            .collect();
        
        let final_quality_assessment = if average_quality >= 0.9 {
            "Excellent reasoning with minimal bias"
        } else if average_quality >= 0.7 {
            "Good reasoning with some minor biases addressed"
        } else if average_quality >= 0.5 {
            "Moderate reasoning quality with significant biases corrected"
        } else {
            "Poor reasoning quality with substantial biases detected"
        }.to_string();
        
        OverallAssessment {
            total_steps,
            biased_steps,
            corrected_steps,
            average_quality,
            most_common_biases,
            final_quality_assessment,
        }
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