use crate::tools::biased_reasoning_synthesis::SynthesisPatch;
use lux_synthesis::SynthesisState;
use serde_json::json;

// Function definition for OpenAI function calling
pub fn get_synthesis_function_definition() -> serde_json::Value {
    json!({
        "name": "update_synthesis",
        "description": "Update the evolving synthesis with new insights from reasoning",
        "parameters": {
            "type": "object",
            "properties": {
                "current_understanding": {
                    "type": "string",
                    "description": "Current best understanding in ≤3 sentences. Be concise and actionable."
                },
                "key_insights": {
                    "type": "array",
                    "description": "Top insights discovered (max 5, each ≤15 words)",
                    "items": {
                        "type": "object",
                        "properties": {
                            "insight": { "type": "string", "maxLength": 100 },
                            "confidence": { "type": "number", "minimum": 0, "maximum": 1 },
                            "source_step": { "type": "integer" },
                            "supported_by_evidence": { "type": "boolean" }
                        },
                        "required": ["insight", "confidence", "source_step", "supported_by_evidence"]
                    },
                    "maxItems": 5
                },
                "action_items": {
                    "type": "array",
                    "description": "Concrete actions to take (max 5)",
                    "items": {
                        "type": "object",
                        "properties": {
                            "action": { "type": "string" },
                            "priority": { "type": "string", "enum": ["high", "medium", "low"] },
                            "rationale": { "type": "string" },
                            "dependencies": { "type": "array", "items": { "type": "string" } }
                        },
                        "required": ["action", "priority", "rationale"]
                    },
                    "maxItems": 5
                },
                "confidence_score": {
                    "type": "number",
                    "minimum": 0,
                    "maximum": 1,
                    "description": "Overall confidence in current understanding (0-1)"
                },
                "clarity_score": {
                    "type": "number",
                    "minimum": 0,
                    "maximum": 1,
                    "description": "How clear and actionable the synthesis is (0-1)"
                },
                "recommendations": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "recommendation": { "type": "string" },
                            "strength": { "type": "string", "enum": ["strong", "moderate", "weak"] },
                            "conditions": { "type": "array", "items": { "type": "string" } }
                        },
                        "required": ["recommendation", "strength"]
                    }
                },
                "context_factors": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Key contextual factors affecting the analysis"
                },
                "constraints": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Limitations, caveats, or biases detected"
                }
            }
        }
    })
}

pub fn reasoning_prompt_with_synthesis(
    query: &str,
    synthesis: &SynthesisState,
    step_number: u32,
) -> String {
    let synthesis_summary = if synthesis.version == 0 {
        "No synthesis yet - this is the first step".to_string()
    } else {
        let insights_str = if synthesis.key_insights.is_empty() {
            "None".to_string()
        } else {
            synthesis
                .key_insights
                .iter()
                .map(|i| format!("{} ({}%)", i.insight, (i.confidence * 100.0) as i32))
                .collect::<Vec<_>>()
                .join("; ")
        };
        format!(
            "Understanding: {}\nInsights: {} tracked\nConfidence: {:.2}\nTop Insights: {}",
            synthesis.current_understanding,
            synthesis.key_insights.len(),
            synthesis.confidence_score,
            insights_str
        )
    };

    format!(
        r#"Analyze this query step-by-step: {}

CURRENT SYNTHESIS (v{}):
{}

INSTRUCTIONS:
1. Reason through the query systematically
2. Focus on discovering actionable insights
3. Your response MUST end with a function call to update_synthesis
4. Include ONLY fields that have new/changed information
5. Keep insights under 15 words each

Step: {}/{}

Your reasoning:"#,
        query,
        synthesis.version,
        synthesis_summary,
        step_number,
        step_number + 2 // Estimate total steps
    )
}

pub fn bias_check_prompt_with_synthesis(
    last_reasoning: &str,
    synthesis: &SynthesisState,
) -> String {
    format!(
        r#"Review this reasoning step for biases:

REASONING:
{}

CURRENT SYNTHESIS:
{}

Check for:
1. Confirmation bias - seeking only supporting evidence
2. Anchoring bias - over-relying on first information
3. Availability bias - overweighting recent/memorable examples
4. Overgeneralization - drawing broad conclusions from limited data
5. False certainty - unwarranted confidence

If biases found, call update_synthesis with:
- Reduced confidence_score
- New constraints listing the biases
- Corrected insights if needed

Be specific about which biases you detect and why."#,
        last_reasoning,
        serde_json::to_string_pretty(synthesis).unwrap_or_default()
    )
}

pub fn final_synthesis_prompt(query: &str, synthesis: &SynthesisState, total_steps: u32) -> String {
    format!(
        r#"Complete the final synthesis for this analysis.

ORIGINAL QUERY: {}

ANALYSIS SUMMARY:
- Total steps: {}
- Current confidence: {:.2}
- Insights gathered: {}
- Constraints identified: {}

Produce a COMPLETE update_synthesis call with:
1. Executive summary (current_understanding)
2. All key insights ranked by importance
3. All actionable items with clear priorities
4. Final confidence assessment
5. Complete list of constraints and caveats

Make this the definitive answer that gives the caller:
- WHAT they should conclude
- HOW they should act on it
- ALL context needed to understand the reasoning"#,
        query,
        total_steps,
        synthesis.confidence_score,
        synthesis.key_insights.len(),
        synthesis.action_items.len()
    )
}

pub fn extract_synthesis_update(llm_response: &str) -> Result<SynthesisPatch, anyhow::Error> {
    // Look for function call in the response

    // Try to find update_synthesis function call
    if let Some(start) = llm_response.find("update_synthesis(") {
        let remainder = &llm_response[start + 17..];

        // Handle Python-style keyword arguments (Understanding="...", Insights=[...])
        if remainder
            .trim_start()
            .starts_with(|c: char| c.is_alphabetic())
        {
            // Convert Python-style to JSON
            let mut json_obj = serde_json::Map::new();

            // Extract Understanding
            if let Some(understanding_start) = remainder.find("Understanding=\"") {
                if let Some(understanding_end) = remainder[understanding_start + 15..].find("\"") {
                    let understanding = &remainder
                        [understanding_start + 15..understanding_start + 15 + understanding_end];
                    json_obj.insert(
                        "current_understanding".to_string(),
                        serde_json::Value::String(understanding.to_string()),
                    );
                }
            }

            // Extract Insights array
            if let Some(insights_start) = remainder.find("Insights=[") {
                let insights_section = &remainder[insights_start + 10..];
                if let Some(insights_end) = insights_section.find("]") {
                    let insights_str = &insights_section[..insights_end];
                    let mut insights = Vec::new();

                    // Parse each quoted string
                    let mut current = insights_str;
                    while let Some(quote_start) = current.find("\"") {
                        let after_quote = &current[quote_start + 1..];
                        if let Some(quote_end) = after_quote.find("\"") {
                            let insight = &after_quote[..quote_end];
                            insights.push(serde_json::json!({
                                "content": insight,
                                "confidence": 0.7
                            }));
                            current = &after_quote[quote_end + 1..];
                        } else {
                            break;
                        }
                    }

                    json_obj.insert(
                        "key_insights".to_string(),
                        serde_json::Value::Array(insights),
                    );
                }
            }

            // Extract Confidence
            if let Some(confidence_start) = remainder.find("Confidence=") {
                let confidence_section = &remainder[confidence_start + 11..];
                if let Some(end) = confidence_section.find(|c: char| !c.is_numeric() && c != '.') {
                    if let Ok(confidence) = confidence_section[..end].parse::<f64>() {
                        json_obj.insert(
                            "confidence_score".to_string(),
                            serde_json::Value::Number(
                                serde_json::Number::from_f64(confidence)
                                    .unwrap_or(serde_json::Number::from(0)),
                            ),
                        );
                    }
                }
            }

            // Convert json_obj to SynthesisPatch
            let patch = SynthesisPatch {
                current_understanding: json_obj
                    .get("current_understanding")
                    .and_then(|v| v.as_str().map(|s| s.to_string())),
                key_insights: json_obj
                    .get("key_insights")
                    .and_then(|v| serde_json::from_value(v.clone()).ok()),
                action_items: json_obj
                    .get("action_items")
                    .and_then(|v| serde_json::from_value(v.clone()).ok()),
                confidence_score: json_obj
                    .get("confidence_score")
                    .and_then(|v| v.as_f64().map(|f| f as f32)),
                clarity_score: json_obj
                    .get("clarity_score")
                    .and_then(|v| v.as_f64().map(|f| f as f32)),
                recommendations: json_obj
                    .get("recommendations")
                    .and_then(|v| serde_json::from_value(v.clone()).ok()),
                context_factors: json_obj
                    .get("context_factors")
                    .and_then(|v| serde_json::from_value(v.clone()).ok()),
                constraints: json_obj
                    .get("constraints")
                    .and_then(|v| serde_json::from_value(v.clone()).ok()),
                last_updated_step: json_obj
                    .get("last_updated_step")
                    .and_then(|v| v.as_u64().map(|u| u as u32)),
            };

            return Ok(patch);
        }

        // Try standard JSON parsing
        if let Some(end) = remainder.find(")") {
            let json_str = &remainder[..end].trim();
            if let Ok(patch) = serde_json::from_str::<SynthesisPatch>(json_str) {
                return Ok(patch);
            }
        }
    }

    // Try to find raw JSON block
    if let Some(start) = llm_response.find("{") {
        if let Some(end) = llm_response.rfind("}") {
            let json_str = &llm_response[start..=end];
            if let Ok(mut json_value) = serde_json::from_str::<serde_json::Value>(json_str) {
                // Handle various formats and normalize keys
                if let Some(obj) = json_value.as_object_mut() {
                    let mut patch = serde_json::Map::new();

                    // Map common variations to standard keys
                    if let Some(understanding) = obj
                        .get("understanding")
                        .or_else(|| obj.get("Understanding"))
                        .or_else(|| obj.get("current_understanding"))
                    {
                        patch.insert("current_understanding".to_string(), understanding.clone());
                    }

                    if let Some(insights) = obj
                        .get("insights")
                        .or_else(|| obj.get("Insights"))
                        .or_else(|| obj.get("key_insights"))
                    {
                        patch.insert("key_insights".to_string(), insights.clone());
                    }

                    if let Some(confidence) = obj
                        .get("confidence")
                        .or_else(|| obj.get("Confidence"))
                        .or_else(|| obj.get("confidence_score"))
                    {
                        patch.insert("confidence_score".to_string(), confidence.clone());
                    }

                    // Copy over any other fields as-is
                    for (key, value) in obj.iter() {
                        if !patch.contains_key(key) {
                            patch.insert(key.clone(), value.clone());
                        }
                    }

                    // Convert to SynthesisPatch struct
                    let synthesis_patch = SynthesisPatch {
                        current_understanding: patch
                            .get("current_understanding")
                            .and_then(|v| v.as_str().map(|s| s.to_string())),
                        key_insights: patch
                            .get("key_insights")
                            .and_then(|v| serde_json::from_value(v.clone()).ok()),
                        action_items: patch
                            .get("action_items")
                            .and_then(|v| serde_json::from_value(v.clone()).ok()),
                        confidence_score: patch
                            .get("confidence_score")
                            .and_then(|v| v.as_f64().map(|f| f as f32)),
                        clarity_score: patch
                            .get("clarity_score")
                            .and_then(|v| v.as_f64().map(|f| f as f32)),
                        recommendations: patch
                            .get("recommendations")
                            .and_then(|v| serde_json::from_value(v.clone()).ok()),
                        context_factors: patch
                            .get("context_factors")
                            .and_then(|v| serde_json::from_value(v.clone()).ok()),
                        constraints: patch
                            .get("constraints")
                            .and_then(|v| serde_json::from_value(v.clone()).ok()),
                        last_updated_step: patch
                            .get("last_updated_step")
                            .and_then(|v| v.as_u64().map(|u| u as u32)),
                    };

                    return Ok(synthesis_patch);
                }
            }
        }
    }

    Err(anyhow::anyhow!("No synthesis update found in response"))
}
