use crate::tools::traced_reasoning::{StepType, TracedReasoningResponse};

/// Formats reasoning output with cool visual indicators
pub struct ReasoningFormatter;

impl ReasoningFormatter {
    /// Format the reasoning response with enhanced visuals
    pub fn format_response(response: &TracedReasoningResponse) -> String {
        let mut output = String::new();

        // Header with step indicator
        output.push_str(&Self::format_header(response));

        // Main content with visual enhancement
        output.push_str(&Self::format_content(response));

        // Metrics and monitoring
        if response.intervention.is_some() || response.metrics.semantic_similarity.is_some() {
            output.push_str(&Self::format_monitoring(response));
        }

        // Synthesis state
        if let Some(ref synthesis) = response.synthesis_snapshot {
            output.push_str(&Self::format_synthesis(synthesis));
        }

        // Next steps or conclusion
        output.push_str(&Self::format_footer(response));

        output
    }

    fn format_header(response: &TracedReasoningResponse) -> String {
        let step_icon = match response.thought_type {
            StepType::Initial => "🚀",
            StepType::Exploration => "🔍",
            StepType::Analysis => "⚡",
            StepType::Synthesis => "🔮",
            StepType::Validation => "✔️",
            StepType::Conclusion => "🎯",
        };

        let progress_bar = Self::create_progress_bar(
            response.thought_number as usize,
            response.total_thoughts as usize,
        );

        format!(
            "\n{} **REASONING CHAIN [{}]** {}\n{}\n",
            "═".repeat(20),
            response.thought_type.to_string().to_uppercase(),
            "═".repeat(20),
            progress_bar
        )
    }

    fn create_progress_bar(current: usize, total: usize) -> String {
        let filled = "█".repeat(current);
        let empty = "░".repeat(total.saturating_sub(current));
        let percentage = (current as f32 / total as f32 * 100.0) as u32;

        format!(
            "⚡ Progress: [{}{}] {}/{}  ({}%)",
            filled, empty, current, total, percentage
        )
    }

    fn format_content(response: &TracedReasoningResponse) -> String {
        let mut output = String::new();

        // Thought number with visual indicator
        let thought_icon = match response.thought_number {
            1 => "🎬",                                 // Start
            n if n == response.total_thoughts => "🏁", // End
            _ => "💫",                                 // Middle
        };

        output.push_str(&format!(
            "\n{} **Thought #{}: {}**\n",
            thought_icon,
            response.thought_number,
            response.thought_type.to_string()
        ));

        // Main content with indentation
        let content_lines: Vec<&str> = response.thought_content.lines().collect();
        for line in content_lines {
            if !line.trim().is_empty() {
                output.push_str(&format!("   {}\n", line));
            } else {
                output.push_str("\n");
            }
        }

        output
    }

    fn format_monitoring(response: &TracedReasoningResponse) -> String {
        let mut output = String::new();

        output.push_str("\n📊 **METACOGNITIVE MONITORING**\n");
        output.push_str("┌─────────────────────────────────┐\n");

        // Semantic coherence
        if let Some(similarity) = response.metrics.semantic_similarity {
            let coherence_bar = Self::create_mini_bar(similarity);
            output.push_str(&format!(
                "│ Coherence:    {} {:.0}% │\n",
                coherence_bar,
                similarity * 100.0
            ));
        }

        // Perplexity (inverse for display - lower is better)
        if let Some(perplexity) = response.metrics.perplexity {
            let perplexity_score = (100.0 - perplexity.min(100.0)) / 100.0;
            let perplexity_bar = Self::create_mini_bar(perplexity_score);
            output.push_str(&format!(
                "│ Clarity:      {} {:.0}% │\n",
                perplexity_bar,
                perplexity_score * 100.0
            ));
        }

        // Confidence
        let confidence = response.metadata.current_confidence;
        let confidence_bar = Self::create_mini_bar(confidence);
        output.push_str(&format!(
            "│ Confidence:   {} {:.0}% │\n",
            confidence_bar,
            confidence * 100.0
        ));

        output.push_str("└─────────────────────────────────┘\n");

        // Intervention warning if present
        if let Some(ref intervention) = response.intervention {
            output.push_str(&format!(
                "\n⚠️  **INTERVENTION**: {}\n",
                intervention.description
            ));
        }

        output
    }

    fn create_mini_bar(value: f32) -> String {
        let filled = (value * 10.0) as usize;
        let empty = 10_usize.saturating_sub(filled);

        let bar = format!("{}{}", "▰".repeat(filled), "▱".repeat(empty));

        // Color indicator based on value
        let indicator = if value > 0.8 {
            "🟢"
        } else if value > 0.5 {
            "🟡"
        } else {
            "🔴"
        };

        format!("{} {}", indicator, bar)
    }

    fn format_synthesis(synthesis: &crate::tools::traced_reasoning::SynthesisSnapshot) -> String {
        let mut output = String::new();

        output.push_str("\n🔮 **SYNTHESIS STATE**\n");
        output.push_str("┌─────────────────────────────────────┐\n");

        // Understanding
        output.push_str(&format!(
            "│ 💡 Understanding: {} │\n",
            Self::truncate(&synthesis.current_understanding, 20)
        ));

        // Key insights count
        output.push_str(&format!(
            "│ 🎯 Insights: {} collected          │\n",
            synthesis.key_insights.len()
        ));

        // Actions
        output.push_str(&format!(
            "│ ⚡ Actions: {} identified           │\n",
            synthesis.next_actions.len()
        ));

        // Readiness
        let ready_icon = if synthesis.ready_for_conclusion {
            "✅"
        } else {
            "🔄"
        };
        output.push_str(&format!(
            "│ {} Ready: {}                    │\n",
            ready_icon,
            if synthesis.ready_for_conclusion {
                "Yes"
            } else {
                "No "
            }
        ));

        output.push_str("└─────────────────────────────────────┘\n");

        // List key insights if any
        if !synthesis.key_insights.is_empty() {
            output.push_str("\n   📌 Key Insights:\n");
            for (i, insight) in synthesis.key_insights.iter().enumerate().take(3) {
                output.push_str(&format!("   {}. {}\n", i + 1, insight));
            }
        }

        output
    }

    fn format_footer(response: &TracedReasoningResponse) -> String {
        let mut output = String::new();

        if response.reasoning_complete.unwrap_or(false) {
            output.push_str("\n");
            output.push_str(&"═".repeat(50));
            output.push_str("\n🏁 **REASONING COMPLETE**\n");

            if let Some(ref answer) = response.final_answer {
                output.push_str(&format!("\n🎯 **Final Answer:**\n{}\n", answer));
            }

            if let Some(ref metrics) = response.overall_metrics {
                output.push_str(&format!(
                    "\n📈 **Average Confidence:** {:.0}%\n",
                    metrics.average_confidence * 100.0
                ));
            }
        } else if let Some(ref next) = response.next_steps {
            output.push_str(&format!("\n⏭️  **Next:** {}\n", next));
        }

        output.push_str(&format!("\n{}\n", "═".repeat(50)));

        output
    }

    fn truncate(s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            s.to_string()
        } else {
            format!("{}...", &s[..max_len - 3])
        }
    }
}

impl StepType {
    fn to_string(&self) -> &str {
        match self {
            StepType::Initial => "Initial",
            StepType::Exploration => "Exploration",
            StepType::Analysis => "Analysis",
            StepType::Synthesis => "Synthesis",
            StepType::Validation => "Validation",
            StepType::Conclusion => "Conclusion",
        }
    }
}
