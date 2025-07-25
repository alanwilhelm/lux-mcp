use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tracing::debug;

use crate::llm::{
    client::{ChatMessage, LLMClient},
    config::LLMConfig,
    model_aliases::ModelResolver,
    openai::OpenAIClient,
    openrouter::OpenRouterClient,
    Role,
};
use crate::monitoring::MetacognitiveMonitor;

#[derive(Debug, Serialize, Deserialize)]
pub struct TracedReasoningRequest {
    pub query: String,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default = "default_max_steps")]
    pub max_steps: u32,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    #[serde(default)]
    pub guardrails: GuardrailConfig,
}

fn default_max_steps() -> u32 { 10 }
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
    pub final_answer: String,
    pub reasoning_steps: Vec<ReasoningStep>,
    pub metrics: ReasoningMetrics,
    pub interventions: Vec<Intervention>,
    pub confidence_score: f32,
    pub model_used: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReasoningStep {
    pub step_number: u32,
    pub thought: String,
    pub step_type: StepType,
    pub confidence: f32,
    pub metrics: StepMetrics,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Intervention {
    pub step: u32,
    pub intervention_type: InterventionType,
    pub description: String,
    pub severity: Severity,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InterventionType {
    SemanticDrift,
    HighPerplexity,
    CircularReasoning,
    InconsistentLogic,
    AttentionScatter,
    HallucinationRisk,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

pub struct TracedReasoningTool {
    monitor: Arc<Mutex<MetacognitiveMonitor>>,
    openai_client: Option<Arc<dyn LLMClient>>,
    openrouter_clients: Vec<(String, Arc<dyn LLMClient>)>,
    model_resolver: ModelResolver,
    config: LLMConfig,
}

impl TracedReasoningTool {
    pub fn new(config: LLMConfig, monitor: Arc<Mutex<MetacognitiveMonitor>>) -> Result<Self> {
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
            monitor,
            openai_client,
            openrouter_clients,
            model_resolver,
            config,
        })
    }
    
    pub async fn trace_reasoning(&self, request: TracedReasoningRequest) -> Result<TracedReasoningResponse> {
        // Reset monitor for new session
        {
            let mut monitor = self.monitor.lock().unwrap();
            monitor.reset_session();
        }
        
        let model = request.model
            .as_ref()
            .map(|m| self.model_resolver.resolve(m))
            .unwrap_or_else(|| self.config.default_reasoning_model.clone());
        
        debug!("Starting traced reasoning with model '{}'", model);
        
        let client = self.get_client_for_model(&model)?;
        
        let mut reasoning_steps = Vec::new();
        let mut interventions = Vec::new();
        let mut conversation_history = vec![
            ChatMessage {
                role: Role::System,
                content: self.build_system_prompt(&request.guardrails),
            },
            ChatMessage {
                role: Role::User,
                content: format!(
                    "Query: {}\n\nPlease reason through this step-by-step, showing your thinking process clearly.",
                    request.query
                ),
            },
        ];
        
        let mut step_count = 0;
        let mut final_answer = String::new();
        
        while step_count < request.max_steps {
            step_count += 1;
            
            let response = client
                .complete(
                    conversation_history.clone(),
                    Some(request.temperature),
                    Some(2000),
                )
                .await
                .context("Failed to get reasoning step")?;
            
            let (step_type, thought, is_final) = self.parse_step_response(&response.content);
            
            let step_metrics = self.calculate_step_metrics(
                &thought,
                &request.query,
                &reasoning_steps,
                &request.guardrails,
            ).await?;
            
            let confidence = self.calculate_step_confidence(&step_metrics);
            
            // Check monitor for interventions
            {
                let mut monitor = self.monitor.lock().unwrap();
                let thought_number = step_count as usize;
                let signals = monitor.analyze_thought(&thought, thought_number);
                
                if let Some(intervention_msg) = signals.intervention {
                    let intervention_type = if signals.circular_score > 0.85 {
                        InterventionType::CircularReasoning
                    } else if signals.distractor_alert {
                        InterventionType::SemanticDrift
                    } else {
                        InterventionType::InconsistentLogic
                    };
                    
                    interventions.push(Intervention {
                        step: step_count,
                        intervention_type,
                        description: intervention_msg,
                        severity: Severity::Medium,
                    });
                }
            }
            
            // Check for additional interventions
            if let Some(intervention) = self.check_interventions(
                step_count,
                &step_metrics,
                &thought,
                &reasoning_steps,
                &request.guardrails,
            ) {
                interventions.push(intervention);
                
                // Add corrective prompt if needed
                if step_metrics.semantic_similarity.unwrap_or(1.0) < request.guardrails.semantic_drift_threshold {
                    conversation_history.push(ChatMessage {
                        role: Role::System,
                        content: "Please refocus on the original query and ensure your reasoning stays relevant.".to_string(),
                    });
                }
            }
            
            let reasoning_step = ReasoningStep {
                step_number: step_count,
                thought: thought.clone(),
                step_type,
                confidence,
                metrics: step_metrics,
            };
            
            reasoning_steps.push(reasoning_step);
            
            conversation_history.push(ChatMessage {
                role: Role::Assistant,
                content: response.content.clone(),
            });
            
            if is_final {
                final_answer = self.extract_final_answer(&response.content);
                break;
            }
            
            // Add continuation prompt
            conversation_history.push(ChatMessage {
                role: Role::User,
                content: "Continue your reasoning to the next step.".to_string(),
            });
        }
        
        // If we haven't reached a conclusion, request one
        if final_answer.is_empty() && step_count >= request.max_steps {
            conversation_history.push(ChatMessage {
                role: Role::User,
                content: "Based on your reasoning so far, please provide a final answer.".to_string(),
            });
            
            let final_response = client
                .complete(conversation_history, Some(request.temperature), Some(1000))
                .await
                .context("Failed to get final answer")?;
            
            final_answer = final_response.content;
        }
        
        let metrics = self.calculate_overall_metrics(&reasoning_steps);
        let confidence_score = self.calculate_final_confidence(&metrics, &interventions);
        
        Ok(TracedReasoningResponse {
            final_answer,
            reasoning_steps,
            metrics,
            interventions,
            confidence_score,
            model_used: model,
        })
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
        previous_steps: &[ReasoningStep],
        _guardrails: &GuardrailConfig,
    ) -> Result<StepMetrics> {
        // Use real monitor to analyze the thought
        let mut monitor = self.monitor.lock().unwrap();
        let thought_number = previous_steps.len() + 1;
        let signals = monitor.analyze_thought(thought, thought_number);
        
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
    
    fn calculate_overall_metrics(&self, steps: &[ReasoningStep]) -> ReasoningMetrics {
        let total_steps = steps.len() as u32;
        let average_confidence = if steps.is_empty() {
            0.0
        } else {
            steps.iter().map(|s| s.confidence).sum::<f32>() / steps.len() as f32
        };
        
        let semantic_coherence = steps
            .iter()
            .filter_map(|s| s.metrics.semantic_similarity)
            .fold(0.0, |acc, x| acc + x) / steps.len().max(1) as f32;
        
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
                let api_key = self.config.openrouter_api_key.as_ref().unwrap();
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