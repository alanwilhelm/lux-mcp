use anyhow::{Context, Result};
use serde_json::json;
use std::sync::Arc;
use tracing::{info, error, debug};

use crate::llm::{
    client::{ChatMessage, LLMClient, Role},
};
use crate::tools::biased_reasoning_synthesis::{
    EvolvingSynthesis, SynthesisPatch, SynthesisStore, EventType,
};
use crate::tools::biased_reasoning_prompts::{
    get_synthesis_function_definition,
    reasoning_prompt_with_synthesis,
    bias_check_prompt_with_synthesis,
    final_synthesis_prompt,
};

pub struct SynthesisIntegration {
    pub store: SynthesisStore,
}

impl SynthesisIntegration {
    pub fn new() -> Self {
        Self {
            store: SynthesisStore::new(),
        }
    }

    /// Call LLM with function calling enabled for synthesis updates
    pub async fn reasoning_with_synthesis(
        &mut self,
        query: &str,
        step_number: u32,
        client: &Arc<dyn LLMClient>,
    ) -> Result<(String, SynthesisPatch)> {
        let prompt = reasoning_prompt_with_synthesis(
            query,
            &self.store.current,
            step_number,
        );

        // Build messages with function calling
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

        // Make request with function definitions
        let response = self.call_with_functions(client, messages).await?;
        
        // Extract reasoning and synthesis patch
        let (reasoning, patch) = self.parse_function_response(&response)?;
        
        // Apply patch to store
        self.store.apply_patch(patch.clone(), step_number, EventType::ReasoningUpdate);
        
        Ok((reasoning, patch))
    }

    /// Bias check with synthesis refinement
    pub async fn bias_check_with_synthesis(
        &mut self,
        last_reasoning: &str,
        step_number: u32,
        client: &Arc<dyn LLMClient>,
    ) -> Result<(String, SynthesisPatch)> {
        let prompt = bias_check_prompt_with_synthesis(
            last_reasoning,
            &self.store.current,
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

        let response = self.call_with_functions(client, messages).await?;
        let (analysis, patch) = self.parse_function_response(&response)?;
        
        self.store.apply_patch(patch.clone(), step_number, EventType::BiasCheckRefinement);
        
        Ok((analysis, patch))
    }

    /// Generate final synthesis
    pub async fn generate_final_synthesis(
        &mut self,
        query: &str,
        total_steps: u32,
        client: &Arc<dyn LLMClient>,
    ) -> Result<(String, SynthesisPatch)> {
        let prompt = final_synthesis_prompt(
            query,
            &self.store.current,
            total_steps,
        );

        let messages = vec![
            ChatMessage {
                role: Role::System,
                content: "Generate a complete, actionable synthesis of the analysis.".to_string(),
            },
            ChatMessage {
                role: Role::User,
                content: prompt,
            },
        ];

        let response = self.call_with_functions(client, messages).await?;
        let (summary, patch) = self.parse_function_response(&response)?;
        
        self.store.apply_patch(patch.clone(), total_steps, EventType::FinalCompilation);
        
        Ok((summary, patch))
    }

    /// Make API call with function definitions
    async fn call_with_functions(
        &self,
        client: &Arc<dyn LLMClient>,
        messages: Vec<ChatMessage>,
    ) -> Result<String> {
        // For OpenAI-compatible APIs, we need to add the functions parameter
        // This is a simplified version - actual implementation would use the proper API
        
        // Build the request with functions
        let function_def = get_synthesis_function_definition();
        
        // For now, we'll use a workaround by appending function instructions to the last message
        // In production, this would use the actual function calling API
        let mut modified_messages = messages.clone();
        if let Some(last_msg) = modified_messages.last_mut() {
            last_msg.content.push_str(&format!(
                "\n\nYou MUST call the update_synthesis function at the end of your response with this exact format:\n```json\n{}\n```",
                serde_json::to_string_pretty(&json!({
                    "function": "update_synthesis",
                    "arguments": {
                        "current_understanding": "Your understanding here",
                        "confidence_score": 0.0
                    }
                })).unwrap()
            ));
        }

        let response = client.complete(modified_messages, None, Some(10000)).await?;
        Ok(response.content)
    }

    /// Parse function call from response
    fn parse_function_response(&self, response: &str) -> Result<(String, SynthesisPatch)> {
        // Split response into reasoning and function call
        let parts: Vec<&str> = response.split("```json").collect();
        
        let reasoning = if parts.len() > 1 {
            parts[0].trim().to_string()
        } else {
            // Try to find the reasoning before any JSON
            if let Some(json_start) = response.find("{") {
                response[..json_start].trim().to_string()
            } else {
                response.to_string()
            }
        };

        // Extract JSON
        let json_str = if parts.len() > 1 {
            // Find the JSON between ```json and ```
            if let Some(end) = parts[1].find("```") {
                &parts[1][..end]
            } else {
                parts[1]
            }
        } else {
            // Try to extract raw JSON
            if let Some(start) = response.find("{") {
                if let Some(end) = response.rfind("}") {
                    &response[start..=end]
                } else {
                    return Err(anyhow::anyhow!("No valid JSON found in response"));
                }
            } else {
                return Err(anyhow::anyhow!("No JSON found in response"));
            }
        };

        // Parse the JSON
        let json_value: serde_json::Value = serde_json::from_str(json_str)
            .context("Failed to parse JSON from response")?;

        // Extract the arguments
        let patch = if let Some(args) = json_value.get("arguments") {
            serde_json::from_value(args.clone())?
        } else {
            // Try direct parsing
            serde_json::from_value(json_value)?
        };

        Ok((reasoning, patch))
    }

    /// Format the current synthesis for display
    pub fn format_current_synthesis(&self) -> String {
        let synthesis = &self.store.current;
        let snapshot = synthesis.to_snapshot();
        
        let mut output = String::new();
        
        // Header with version
        output.push_str("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
        output.push_str(&format!("📊 **EVOLVING SYNTHESIS** (Version {})\n", synthesis.version));
        output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n");
        
        // Understanding section
        output.push_str("🎯 **Current Understanding:**\n");
        output.push_str(&format!("   {}\n\n", 
            if snapshot.current_understanding.is_empty() { 
                "🔄 Building understanding..." 
            } else { 
                &snapshot.current_understanding 
            }
        ));
        
        // Insights section with full details
        output.push_str("💡 **Key Insights Collected:**\n");
        if synthesis.key_insights.is_empty() {
            output.push_str("   • No insights collected yet\n");
        } else {
            for (i, insight) in synthesis.key_insights.iter().enumerate() {
                let confidence_emoji = match (insight.confidence * 100.0) as i32 {
                    0..=30 => "🔴",
                    31..=60 => "🟡",
                    61..=85 => "🟢",
                    _ => "✅"
                };
                output.push_str(&format!(
                    "   {}. {} {} [Confidence: {:.0}%] (Step {})\n", 
                    i + 1, 
                    confidence_emoji,
                    insight.insight,
                    insight.confidence * 100.0,
                    insight.source_step
                ));
            }
        }
        output.push_str("\n");
        
        // Actions section with priority and rationale
        output.push_str("📌 **Recommended Actions:**\n");
        if synthesis.action_items.is_empty() {
            output.push_str("   • No actions identified yet\n");
        } else {
            for action in &synthesis.action_items {
                let priority_emoji = match action.priority {
                    crate::tools::biased_reasoning_synthesis::Priority::High => "🔴",
                    crate::tools::biased_reasoning_synthesis::Priority::Medium => "🟡",
                    crate::tools::biased_reasoning_synthesis::Priority::Low => "🟢",
                };
                output.push_str(&format!(
                    "   {} {} (Priority: {:?})\n      → Rationale: {}\n", 
                    priority_emoji, 
                    action.action,
                    action.priority,
                    action.rationale
                ));
                if !action.dependencies.is_empty() {
                    output.push_str(&format!("      → Dependencies: {}\n", action.dependencies.join(", ")));
                }
            }
        }
        output.push_str("\n");
        
        // Metrics section
        output.push_str("📈 **Analysis Metrics:**\n");
        output.push_str(&format!("   • Confidence: {:.0}% {}\n", 
            synthesis.confidence_score * 100.0,
            match (synthesis.confidence_score * 100.0) as i32 {
                0..=30 => "🔴 (Low - Exploring)",
                31..=60 => "🟡 (Medium - Developing)", 
                61..=85 => "🟢 (High - Converging)",
                _ => "✅ (Very High - Ready)"
            }
        ));
        output.push_str(&format!("   • Clarity: {:.0}% {}\n", 
            synthesis.clarity_score * 100.0,
            match (synthesis.clarity_score * 100.0) as i32 {
                0..=30 => "🌫️ (Unclear)",
                31..=60 => "⛅ (Partially Clear)", 
                61..=85 => "🌤️ (Mostly Clear)",
                _ => "☀️ (Crystal Clear)"
            }
        ));
        
        // Context and constraints if present
        if !synthesis.context_factors.is_empty() {
            output.push_str("\n🔍 **Context Factors:**\n");
            for factor in &synthesis.context_factors {
                output.push_str(&format!("   • {}\n", factor));
            }
        }
        
        if !synthesis.constraints.is_empty() {
            output.push_str("\n⚠️ **Constraints:**\n");
            for constraint in &synthesis.constraints {
                output.push_str(&format!("   • {}\n", constraint));
            }
        }
        
        // Decision readiness
        let ready = synthesis.confidence_score > 0.7 && synthesis.clarity_score > 0.7;
        output.push_str(&format!("\n🎯 **Decision Readiness:** {}\n",
            if ready {
                "✅ Ready for final decision"
            } else {
                "🔄 More analysis needed"
            }
        ));
        
        // Evolution indicator
        if synthesis.version > 1 {
            output.push_str(&format!("\n📊 **Evolution:** {} synthesis updates tracked\n", synthesis.version));
        }
        
        output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
        
        output
    }

    /// Format final synthesis for MCP response
    pub fn format_final_synthesis(&self, total_steps: u32, models_used: Vec<String>) -> String {
        let final_synthesis = self.store.current.to_final_synthesis(total_steps, models_used);
        
        format!(
            r#"✅ **FINAL SYNTHESIS** ✅

## Executive Summary
{}

## Conclusion
**Main Finding:** {}

**Supporting Points:**
{}

**Important Caveats:**
{}

## Action Plan

### Immediate Actions (High Priority)
{}

### Medium-term Actions
{}

### Additional Considerations
{}

## Context & Methodology
- **Key Factors:** {}
- **Total Analysis Steps:** {}
- **Final Confidence:** {:.0}%
- **Biases Addressed:** {}

---

🎯 **READY FOR IMPLEMENTATION** - All necessary context and actions have been provided."#,
            final_synthesis.executive_summary,
            final_synthesis.conclusion.main_finding,
            final_synthesis.conclusion.supporting_points
                .iter()
                .map(|p| format!("• {}", p))
                .collect::<Vec<_>>()
                .join("\n"),
            if final_synthesis.conclusion.caveats.is_empty() {
                "• None identified".to_string()
            } else {
                final_synthesis.conclusion.caveats
                    .iter()
                    .map(|c| format!("• {}", c))
                    .collect::<Vec<_>>()
                    .join("\n")
            },
            final_synthesis.action_plan.immediate_actions
                .iter()
                .map(|a| format!("• {} - {}", a.action, a.rationale))
                .collect::<Vec<_>>()
                .join("\n"),
            if final_synthesis.action_plan.medium_term_actions.is_empty() {
                "• None identified".to_string()
            } else {
                final_synthesis.action_plan.medium_term_actions
                    .iter()
                    .map(|a| format!("• {} - {}", a.action, a.rationale))
                    .collect::<Vec<_>>()
                    .join("\n")
            },
            final_synthesis.action_plan.considerations
                .iter()
                .map(|c| format!("• {}", c))
                .collect::<Vec<_>>()
                .join("\n"),
            final_synthesis.full_context.key_factors.join(", "),
            final_synthesis.meta.total_steps,
            final_synthesis.meta.confidence_score * 100.0,
            if final_synthesis.meta.biases_detected.is_empty() {
                "None detected".to_string()
            } else {
                final_synthesis.meta.biases_detected.join(", ")
            }
        )
    }
}