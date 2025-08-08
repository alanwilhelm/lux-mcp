use rmcp::{
    model::{
        CallToolRequestParam, CallToolResult, Content, GetPromptRequestParam, GetPromptResult,
        ListPromptsResult, ListToolsResult, PaginatedRequestParam, Prompt, PromptMessage,
        PromptMessageContent, PromptMessageRole, ServerInfo, Tool,
    },
    service::RequestContext,
    Error as McpError, ServerHandler,
};
use serde_json::{json, Map, Value};
use std::sync::Arc;
use tracing::{debug, error, info};

use super::LuxServer;
use crate::tools::{
    BiasedReasoningRequest, ChatRequest, PlannerRequest, StepType, TracedReasoningRequest,
};
use lux_synthesis_db::PostgresSink;

fn json_to_arc_map(value: Value) -> Arc<Map<String, Value>> {
    Arc::new(value.as_object().cloned().unwrap_or_default())
}

impl ServerHandler for LuxServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: Default::default(),
            capabilities: rmcp::model::ServerCapabilities {
                tools: Some(rmcp::model::ToolsCapability { list_changed: None }),
                prompts: Some(rmcp::model::PromptsCapability { list_changed: None }),
                ..Default::default()
            },
            server_info: rmcp::model::Implementation {
                name: "lux-mcp".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            instructions: Some(
                "Illuminate your thinking - A metacognitive monitoring MCP server".into(),
            ),
        }
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<rmcp::service::RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        let tools = vec![
            Tool {
                name: "confer".into(),
                description: Some("Simple conversational AI with model selection".into()),
                input_schema: json_to_arc_map(json!({
                    "type": "object",
                    "properties": {
                        "message": {
                            "type": "string",
                            "description": "The message to send to the AI"
                        },
                        "model": {
                            "type": "string",
                            "description": "Optional model to use (e.g., 'gpt4.1', 'claude', 'gemini')"
                        },
                        "temperature": {
                            "type": "number",
                            "description": "Optional temperature (0.0-1.0)"
                        },
                        "continuation_id": {
                            "type": "string",
                            "description": "Optional thread ID to continue a previous conversation"
                        }
                    },
                    "required": ["message"]
                })),
                annotations: None,
            },
            Tool {
                name: "traced_reasoning".into(),
                description: Some("Multi-call step-by-step reasoning with metacognitive monitoring - Generate variable thoughts with detailed output for each".into()),
                input_schema: json_to_arc_map(json!({
                    "type": "object",
                    "properties": {
                        "thought": {
                            "type": "string",
                            "description": "For thought 1: the query/problem. For thoughts 2+: guidance or continuation from previous thought"
                        },
                        "thought_number": {
                            "type": "integer",
                            "minimum": 1,
                            "description": "Current thought number in the reasoning sequence (starts at 1)"
                        },
                        "total_thoughts": {
                            "type": "integer",
                            "minimum": 1,
                            "description": "Estimated total thoughts needed (can be adjusted as reasoning progresses)"
                        },
                        "next_thought_needed": {
                            "type": "boolean",
                            "description": "Whether another thought is required after this one"
                        },
                        "is_revision": {
                            "type": "boolean",
                            "description": "True if this thought revises a previous thought"
                        },
                        "revises_thought": {
                            "type": "integer",
                            "description": "If is_revision is true, which thought number is being revised"
                        },
                        "branch_from_thought": {
                            "type": "integer",
                            "description": "If branching, which thought number is the branching point"
                        },
                        "branch_id": {
                            "type": "string",
                            "description": "Identifier for the current branch (e.g., 'alternative-reasoning', 'hypothesis-B')"
                        },
                        "needs_more_thoughts": {
                            "type": "boolean",
                            "description": "True if more thoughts are needed beyond the initial estimate"
                        },
                        "model": {
                            "type": "string",
                            "description": "Optional model to use for reasoning"
                        },
                        "temperature": {
                            "type": "number",
                            "description": "Optional temperature (0.0-1.0, default: 0.7)"
                        },
                        "guardrails": {
                            "type": "object",
                            "properties": {
                                "semantic_drift_check": {
                                    "type": "boolean",
                                    "description": "Enable semantic drift checking (default: true)"
                                },
                                "circular_reasoning_detection": {
                                    "type": "boolean",
                                    "description": "Enable circular reasoning detection (default: true)"
                                },
                                "perplexity_monitoring": {
                                    "type": "boolean",
                                    "description": "Enable perplexity monitoring (default: true)"
                                }
                            }
                        },
                        "continuation_id": {
                            "type": "string",
                            "description": "Optional thread ID to continue a previous conversation"
                        }
                    },
                    "required": ["thought", "thought_number", "total_thoughts", "next_thought_needed"]
                })),
                annotations: None,
            },
            Tool {
                name: "biased_reasoning".into(),
                description: Some("Step-by-step dual-model reasoning with bias detection. Returns one step per call with session_id for continuity.".into()),
                input_schema: json_to_arc_map(json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "The question or problem to analyze for bias"
                        },
                        "session_id": {
                            "type": "string",
                            "description": "Optional session ID to continue an existing reasoning session"
                        },
                        "new_session": {
                            "type": "boolean",
                            "description": "Force start a new session even if query matches existing (default: false)"
                        },
                        "primary_model": {
                            "type": "string",
                            "description": "Optional primary model for reasoning"
                        },
                        "verifier_model": {
                            "type": "string",
                            "description": "Optional verifier model for bias checking"
                        },
                        "max_analysis_rounds": {
                            "type": "integer",
                            "description": "Maximum analysis rounds (default: 3)"
                        }
                    },
                    "required": ["query"]
                })),
                annotations: None,
            },
            Tool {
                name: "illumination_status".into(),
                description: Some("Check the current illumination status and metacognitive state".into()),
                input_schema: json_to_arc_map(json!({
                    "type": "object",
                    "properties": {}
                })),
                annotations: None,
            },
            Tool {
                name: "planner".into(),
                description: Some("Interactive sequential planner - Break down complex tasks through step-by-step planning".into()),
                input_schema: json_to_arc_map(json!({
                    "type": "object",
                    "properties": {
                        "step": {
                            "type": "string",
                            "description": "Your current planning step. For step 1, describe the task/problem to plan. For subsequent steps, provide the actual planning step content."
                        },
                        "step_number": {
                            "type": "integer",
                            "minimum": 1,
                            "description": "Current step number in the planning sequence (starts at 1)"
                        },
                        "total_steps": {
                            "type": "integer",
                            "minimum": 1,
                            "description": "Current estimate of total steps needed (can be adjusted up/down as planning progresses)"
                        },
                        "next_step_required": {
                            "type": "boolean",
                            "description": "Whether another planning step is required after this one"
                        },
                        "is_step_revision": {
                            "type": "boolean",
                            "description": "True if this step revises/replaces a previous step"
                        },
                        "revises_step_number": {
                            "type": "integer",
                            "description": "If is_step_revision is true, which step number is being revised"
                        },
                        "is_branch_point": {
                            "type": "boolean",
                            "description": "True if this step branches from a previous step to explore alternatives"
                        },
                        "branch_from_step": {
                            "type": "integer",
                            "description": "If is_branch_point is true, which step number is the branching point"
                        },
                        "branch_id": {
                            "type": "string",
                            "description": "Identifier for the current branch (e.g., 'approach-A', 'microservices-path')"
                        },
                        "more_steps_needed": {
                            "type": "boolean",
                            "description": "True if more steps are needed beyond the initial estimate"
                        },
                        "model": {
                            "type": "string",
                            "description": "Optional model to use for planning"
                        },
                        "temperature": {
                            "type": "number",
                            "description": "Optional temperature (0.0-1.0, default: 0.7)"
                        }
                    },
                    "required": ["step", "step_number", "total_steps", "next_step_required"]
                })),
                annotations: None,
            },
        ];

        Ok(ListToolsResult {
            next_cursor: None,
            tools,
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        _context: RequestContext<rmcp::service::RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        info!("Tool request: {}", request.name);
        debug!("Tool arguments: {:?}", request.arguments);

        match request.name.as_ref() {
            "confer" => {
                let mut args = request.arguments.clone().unwrap_or_default();

                // Extract and handle continuation_id
                let continuation_id = args
                    .get("continuation_id")
                    .and_then(|v| v.as_str())
                    .and_then(|s| uuid::Uuid::parse_str(s).ok());

                // Reconstruct context if continuation exists
                let thread_id = if let Some(id) = continuation_id {
                    if let Some(context) =
                        crate::threading::reconstruction::ContextReconstructor::reconstruct(
                            &self.thread_manager,
                            &id,
                        )
                    {
                        // Prepend context to message
                        if let Some(message) = args.get_mut("message") {
                            if let Some(msg_str) = message.as_str() {
                                let enhanced_message =
                                    format!("{}\n\nCurrent message: {}", context, msg_str);
                                *message = serde_json::Value::String(enhanced_message);
                            }
                        }
                    }
                    id
                } else {
                    // Create new thread
                    self.thread_manager.create_thread("confer")
                };

                let req: ChatRequest = match serde_json::from_value(serde_json::Value::Object(args))
                {
                    Ok(req) => req,
                    Err(e) => {
                        error!("Failed to parse chat request: {}", e);
                        return Err(McpError::invalid_params(
                            format!("Invalid chat params: {}", e),
                            None,
                        ));
                    }
                };

                info!(
                    "Processing confer request with message: {} (thread: {})",
                    req.message, thread_id
                );
                match self.chat_tool.chat(req.clone()).await {
                    Ok(response) => {
                        info!("Confer request successful - Model used: {}", response.model);

                        // Store the conversation turn
                        let turn = crate::threading::ConversationTurn {
                            role: crate::threading::context::Role::Assistant,
                            content: response.content.clone(),
                            tool_used: Some("confer".to_string()),
                            synthesis_snapshot: None,
                            quality_metrics: None,
                            timestamp: std::time::Instant::now(),
                        };
                        self.thread_manager.add_turn(&thread_id, turn);

                        // Format response with continuation_id
                        let formatted_response = format!(
                            "🔍 **LUX ANALYSIS COMPLETE** 🔍\n\n\
                            ⚠️ **IMPORTANT: You MUST now SYNTHESIZE and ACT on the following analysis from the {} model:**\n\n\
                            ---\n\n{}\n\n---\n\n\
                            🎯 **REQUIRED ACTIONS:**\n\
                            1. ✅ SUMMARIZE the key findings\n\
                            2. ✅ IDENTIFY the most critical recommendations\n\
                            3. ✅ CREATE an actionable plan based on this analysis\n\
                            4. ✅ RESPOND with clear next steps for the user\n\n\
                            ⚡ **DO NOT just acknowledge this output - YOU MUST PROCESS AND ACT ON IT!** ⚡\n\n\
                            📎 **Continuation ID**: {} (Use this to continue the conversation)",
                            response.model,
                            response.content,
                            thread_id
                        );

                        Ok(CallToolResult {
                            content: vec![Content::text(formatted_response)],
                            is_error: Some(false),
                        })
                    }
                    Err(e) => {
                        error!("Chat tool failed: {}", e);
                        Err(McpError::internal_error(format!("Chat error: {}", e), None))
                    }
                }
            }

            "traced_reasoning" => {
                let req: TracedReasoningRequest = if let Some(args) = request.arguments {
                    serde_json::from_value(serde_json::Value::Object(args)).map_err(|e| {
                        McpError::invalid_params(format!("Invalid reasoning params: {}", e), None)
                    })?
                } else {
                    return Err(McpError::invalid_params(
                        "Missing arguments for traced reasoning",
                        None,
                    ));
                };

                info!(
                    "Processing traced reasoning - thought {} of {}",
                    req.thought_number, req.total_thoughts
                );
                if let Some(ref model) = req.model {
                    info!("Using specified model: {}", model);
                }

                // Lock the mutex to access the mutable traced reasoning tool
                let mut reasoning_tool = self.traced_reasoning_tool.lock().await;

                // Set up synthesis sink if database is available
                if let Some(db) = &self.db_service {
                    let pool = db.pool();
                    let sink = Arc::new(PostgresSink::new(pool));
                    reasoning_tool.set_synthesis_sink(sink);
                }

                let response = reasoning_tool.process_thought(req).await.map_err(|e| {
                    McpError::internal_error(format!("Reasoning error: {}", e), None)
                })?;

                // Drop the lock immediately after use
                drop(reasoning_tool);

                info!("Response model_used: {:?}", response.model_used);

                // Format the response based on status
                let formatted_response = match response.status.as_str() {
                    "thinking" => {
                        // Always show the model being used
                        let model_name = response
                            .model_used
                            .as_ref()
                            .cloned()
                            .unwrap_or_else(|| "ERROR: Model not specified".to_string());
                        let model_display = format!("Model: {} 🤖\n", model_name);

                        let mut result = format!(
                            "💭 Thought {} of {}: {}\n\
                            {}\
                            Confidence: {:.2}\n\n\
                            ---\n\n\
                            {}\n\n\
                            ---\n\n\
                            📊 Metrics:\n\
                            • Semantic Coherence: {:.2}\n\
                            • Current Confidence: {:.2}\n\
                            • Interventions Count: {}",
                            response.thought_number,
                            response.total_thoughts,
                            response.thought_type,
                            model_display,
                            response.metadata.current_confidence,
                            response.thought_content,
                            response.metadata.semantic_coherence,
                            response.metadata.current_confidence,
                            response.metadata.interventions_count
                        );

                        // Add synthesis information if available
                        if let Some(synthesis) = &response.synthesis_snapshot {
                            result.push_str(&format!(
                                "\n\n🎯 **Synthesis State:**\n\
                                • Understanding: {}\n\
                                • Confidence: {}\n\
                                • Clarity: {}\n\
                                • Ready for Conclusion: {}",
                                synthesis.current_understanding,
                                synthesis.confidence_level,
                                synthesis.clarity_level,
                                if synthesis.ready_for_conclusion {
                                    "Yes"
                                } else {
                                    "No"
                                }
                            ));

                            if !synthesis.key_insights.is_empty() {
                                result.push_str("\n\n💡 **Key Insights:**\n");
                                for insight in &synthesis.key_insights {
                                    result.push_str(&format!("• {}\n", insight));
                                }
                            }
                        }

                        result.push_str(&format!(
                            "\n\n➡️ **Next Action:** {}\n\n\
                            Use traced_reasoning again with thought_number: {} to continue.",
                            response
                                .next_steps
                                .as_ref()
                                .unwrap_or(&"Continue reasoning".to_string()),
                            response.thought_number + 1
                        ));

                        result
                    }
                    "intervention_needed" => {
                        let intervention = response.intervention.as_ref().unwrap();
                        // Always show the model being used
                        let model_name = response
                            .model_used
                            .as_ref()
                            .cloned()
                            .unwrap_or_else(|| "ERROR: Model not specified".to_string());
                        let model_display = format!("Model: {}", model_name);

                        format!(
                            "⚠️ **REASONING INTERVENTION** ⚠️\n\n\
                            Thought {} of {}: INTERVENTION REQUIRED\n\
                            {}\n\n\
                            ---\n\n\
                            🚨 **Issue Detected:** {:?}\n\
                            **Severity:** {:?}\n\
                            **Description:** {}\n\n\
                            💭 **Thought Content:**\n{}\n\n\
                            ---\n\n\
                            🔧 **Required Action:** Adjust your reasoning to address the intervention.\n\n\
                            Continue with thought_number: {} after considering the intervention.",
                            response.thought_number,
                            response.total_thoughts,
                            model_display,
                            intervention.intervention_type,
                            intervention.severity,
                            intervention.description,
                            response.thought_content,
                            response.thought_number + 1
                        )
                    }
                    "conclusion_reached" => {
                        // Always show the model being used
                        let model_name = response
                            .model_used
                            .as_ref()
                            .cloned()
                            .unwrap_or_else(|| "ERROR: Model not specified".to_string());
                        let model_display = format!("Model: {}", model_name);

                        format!(
                            "✅ **REASONING COMPLETE** ✅\n\n\
                            Final Thought ({} of {})\n\
                            {}\n\n\
                            ---\n\n\
                            💡 **Final Answer:**\n{}\n\n\
                            ---\n\n\
                            📊 **Overall Metrics:**\n\
                            • Total Thoughts: {}\n\
                            • Average Confidence: {:.2}\n\
                            • Reasoning Quality: {:.2}\n\
                            • Semantic Coherence: {:.2}\n\n\
                            🔍 **Instructions:**\n{}",
                            response.thought_number,
                            response.total_thoughts,
                            model_display,
                            response
                                .final_answer
                                .as_ref()
                                .unwrap_or(&response.thought_content),
                            response
                                .overall_metrics
                                .as_ref()
                                .map(|m| m.total_steps)
                                .unwrap_or(response.thought_number),
                            response
                                .overall_metrics
                                .as_ref()
                                .map(|m| m.average_confidence)
                                .unwrap_or(0.0),
                            response
                                .overall_metrics
                                .as_ref()
                                .map(|m| m.reasoning_quality)
                                .unwrap_or(0.0),
                            response
                                .overall_metrics
                                .as_ref()
                                .map(|m| m.semantic_coherence)
                                .unwrap_or(0.0),
                            response
                                .next_steps
                                .as_ref()
                                .unwrap_or(&"Present the reasoning to the user".to_string())
                        )
                    }
                    _ => {
                        // Always show the model being used
                        let model_name = response
                            .model_used
                            .as_ref()
                            .cloned()
                            .unwrap_or_else(|| "ERROR: Model not specified".to_string());
                        format!(
                            "Reasoning Status: {}\n\n{}\n\nModel: {}",
                            response.status, response.thought_content, model_name
                        )
                    }
                };

                Ok(CallToolResult {
                    content: vec![Content::text(formatted_response)],
                    is_error: Some(false),
                })
            }

            "biased_reasoning" => {
                let req: BiasedReasoningRequest = if let Some(args) = request.arguments {
                    serde_json::from_value(serde_json::Value::Object(args)).map_err(|e| {
                        McpError::invalid_params(
                            format!("Invalid biased reasoning params: {}", e),
                            None,
                        )
                    })?
                } else {
                    return Err(McpError::invalid_params(
                        "Missing arguments for biased reasoning",
                        None,
                    ));
                };

                // Set up synthesis sink if database is available
                if let Some(db) = &self.db_service {
                    let pool = db.pool();
                    let sink = Arc::new(PostgresSink::new(pool));
                    self.biased_reasoning_tool.set_synthesis_sink(sink);
                }

                let start_time = std::time::Instant::now();
                let response = self
                    .biased_reasoning_tool
                    .process_step(req.clone())
                    .await
                    .map_err(|e| {
                        McpError::internal_error(format!("Biased reasoning error: {}", e), None)
                    })?;
                let thinking_time_ms = start_time.elapsed().as_millis() as i32;

                // Log to database if available
                if let Some(db) = &self.db_service {
                    if let Err(e) = db
                        .log_biased_reasoning_step(&req, &response, thinking_time_ms)
                        .await
                    {
                        error!("Failed to log biased reasoning step to database: {}", e);
                    }
                }

                // Format response based on step type
                let formatted_response = match response.step_type {
                    StepType::Query => {
                        format!(
                            "📝 **Query Received**\n\n\
                            {}\n\n\
                            Session ID: {}\n\
                            Status: {} total steps\n\
                            Next: {:?}",
                            response.content,
                            response.session_id,
                            response.session_status.total_steps,
                            response.next_action
                        )
                    }
                    StepType::Reasoning => {
                        let metadata = response.reasoning_metadata.as_ref();
                        let metadata_info = if let Some(m) = metadata {
                            let tokens_info = m
                                .tokens_generated
                                .map(|t| format!("\n            Tokens: {}", t))
                                .unwrap_or_default();
                            format!(
                                "\n            Thinking Time: {}ms\n\
                                            Depth: {}\n\
                                            Confidence: {:.2}{}",
                                m.thinking_time_ms,
                                m.reasoning_depth,
                                m.confidence_level,
                                tokens_info
                            )
                        } else {
                            String::new()
                        };

                        format!(
                            "Step {}: {}\n\n\
                            {}\n\n\
                            Session ID: {}\n\
                            Model: {} 🤖{}\n\
                            Next: {:?}\n\n\
                            Session Progress: {}/{} steps completed",
                            response.step_number,
                            response.step_type,
                            response.content,
                            response.session_id,
                            response.model_used,
                            if response.step_type == StepType::BiasAnalysis {
                                " (Bias Checker)"
                            } else {
                                ""
                            },
                            response.next_action,
                            response.session_status.reasoning_steps + response.session_status.bias_checks,
                            response.session_status.total_steps
                        )
                    }
                    StepType::BiasAnalysis => {
                        let bias_info = if let Some(ref bias) = response.bias_analysis {
                            format!(
                                "\n\n📊 **Detailed Analysis:**\n\
                                • Confidence: {:.2}\n\
                                • Severity: {:?}\n\
                                • Bias Types: {:?}",
                                bias.confidence, bias.severity, bias.bias_types
                            )
                        } else {
                            String::new()
                        };

                        format!(
                            "🔍 **Bias Analysis Step {}**\n\n\
                            {}\n\n\
                            Session ID: {}\n\
                            Model: {}\n\
                            Next: {:?}{}\n\n\
                            Quality Score: {:.2}",
                            response.step_number,
                            response.content,
                            response.session_id,
                            response.model_used,
                            response.next_action,
                            bias_info,
                            response.session_status.overall_quality
                        )
                    }
                    StepType::Correction => {
                        let details = response.correction_details.as_ref();
                        format!(
                            "✏️ **Correction Step {}**\n\n\
                            {}\n\n\
                            Model: {}\n\
                            Improvement Score: {:.2}\n\
                            Next: {:?}",
                            response.step_number,
                            response.content,
                            response.model_used,
                            details.map(|d| d.improvement_score).unwrap_or(0.0),
                            response.next_action
                        )
                    }
                    StepType::Guidance => {
                        format!(
                            "📝 **User Guidance Step {}**\n\n\
                            {}\n\n\
                            Next: {:?}",
                            response.step_number, response.content, response.next_action
                        )
                    }
                    StepType::Synthesis => {
                        format!(
                            "🎯 **Final Synthesis**\n\n\
                            {}\n\n\
                            Model: {}\n\n\
                            📊 **Session Summary:**\n\
                            • Total Steps: {}\n\
                            • Reasoning Steps: {}\n\
                            • Bias Checks: {}\n\
                            • Corrections Made: {}\n\
                            • Overall Quality: {:.2}\n\n\
                            Status: Complete ✅",
                            response.content,
                            response.model_used,
                            response.session_status.total_steps,
                            response.session_status.reasoning_steps,
                            response.session_status.bias_checks,
                            response.session_status.corrections_made,
                            response.session_status.overall_quality
                        )
                    }
                };

                Ok(CallToolResult {
                    content: vec![Content::text(formatted_response)],
                    is_error: Some(false),
                })
            }

            "illumination_status" => {
                let status = json!({
                    "illumination": "active",
                    "brightness": 0.95,
                    "shadows_detected": "none",
                    "metacognitive_state": "clear",
                    "message": "Your thinking is illuminated and clear 🔦"
                });

                Ok(CallToolResult {
                    content: vec![Content::text(
                        serde_json::to_string_pretty(&status).map_err(|e| {
                            McpError::internal_error(
                                format!("Failed to serialize status: {}", e),
                                None,
                            )
                        })?,
                    )],
                    is_error: Some(false),
                })
            }

            "planner" => {
                let req: PlannerRequest = if let Some(args) = request.arguments {
                    serde_json::from_value(serde_json::Value::Object(args)).map_err(|e| {
                        McpError::invalid_params(format!("Invalid planner params: {}", e), None)
                    })?
                } else {
                    return Err(McpError::invalid_params(
                        "Missing arguments for planner",
                        None,
                    ));
                };

                info!(
                    "Processing planner request - step {} of {}",
                    req.step_number, req.total_steps
                );

                // Lock the mutex to access the mutable planner tool
                let mut planner = self.planner_tool.lock().await;

                // Set up synthesis sink if database is available
                if let Some(db) = &self.db_service {
                    let pool = db.pool();
                    let sink = Arc::new(PostgresSink::new(pool));
                    planner.set_synthesis_sink(sink);
                }

                let response = planner
                    .create_plan(req)
                    .await
                    .map_err(|e| McpError::internal_error(format!("Planner error: {}", e), None))?;

                // Drop the lock immediately after use
                drop(planner);

                // Always show the actual model being used
                let model_name = response
                    .model_used
                    .as_ref()
                    .cloned()
                    .unwrap_or_else(|| "ERROR: Model not specified".to_string());

                // Format the response based on status
                let formatted_response = match response.status.as_str() {
                    "pause_for_deep_thinking" => {
                        format!(
                            "🤔 DEEP THINKING REQUIRED\n\n\
                            Step {} of {}: {}\n\
                            Model: {} 🤖\n\n\
                            ---\n\n\
                            ⏸️ MANDATORY PAUSE FOR REFLECTION\n\n\
                            {}\n\n\
                            🎯 Required Thinking:\n{}\n\n\
                            ⚠️ DO NOT PROCEED until you have completed this deep analysis!",
                            response.step_number,
                            response.total_steps,
                            response.step_content,
                            model_name,
                            response
                                .next_steps
                                .as_ref()
                                .unwrap_or(&"Continue planning...".to_string()),
                            response
                                .required_thinking
                                .as_ref()
                                .map(|rt| rt
                                    .iter()
                                    .map(|t| format!("• {}", t))
                                    .collect::<Vec<_>>()
                                    .join("\n"))
                                .unwrap_or_default()
                        )
                    }
                    "pause_for_planner" => {
                        let mut result = format!(
                            "📋 **PLANNING STEP RECORDED** 📋\n\n\
                            Step {} of {}: {}\n\
                            Model: {} 🤖\n\n\
                            ---\n\n\
                            📊 **Planning Progress:**\n\
                            • Steps completed: {}\n\
                            • Branches explored: {}\n\
                            • Is revision: {}\n\
                            • Is branch: {}\n\n",
                            response.step_number,
                            response.total_steps,
                            response.step_content,
                            model_name,
                            response.metadata.step_history_length,
                            response.metadata.branches.len(),
                            if response.metadata.is_step_revision {
                                "Yes"
                            } else {
                                "No"
                            },
                            if response.metadata.is_branch_point {
                                "Yes"
                            } else {
                                "No"
                            },
                        );

                        // Add synthesis information if available
                        if let Some(synthesis) = &response.synthesis_snapshot {
                            result.push_str(&format!(
                                "🎯 **Synthesis State:**\n\
                                • Current Plan: {}\n\
                                • Confidence: {}\n\
                                • Ready for Execution: {}\n",
                                synthesis.current_plan,
                                synthesis.confidence_level,
                                if synthesis.ready_for_execution {
                                    "Yes"
                                } else {
                                    "No"
                                }
                            ));

                            if !synthesis.key_decisions.is_empty() {
                                result.push_str("\n💡 **Key Decisions:**\n");
                                for decision in &synthesis.key_decisions {
                                    result.push_str(&format!("• {}\n", decision));
                                }
                            }

                            if !synthesis.next_actions.is_empty() {
                                result.push_str("\n📌 **Next Actions:**\n");
                                for action in &synthesis.next_actions {
                                    result.push_str(&format!("• {}\n", action));
                                }
                            }
                            result.push_str("\n");
                        }

                        result.push_str(&format!(
                            "➡️ **Next Action:** {}\n\n\
                            Use the planner tool again with step_number: {} to continue.",
                            response
                                .next_steps
                                .as_ref()
                                .unwrap_or(&"Continue planning".to_string()),
                            response.step_number + 1
                        ));

                        result
                    }
                    "planning_complete" => {
                        format!(
                            "✅ **PLANNING COMPLETE** ✅\n\n\
                            Model: {}\n\n\
                            {}\n\n\
                            ---\n\n\
                            📋 **Instructions:**\n{}\n\n\
                            🎯 **Ready for Implementation!**",
                            model_name,
                            response
                                .plan_summary
                                .as_ref()
                                .unwrap_or(&"Plan completed".to_string()),
                            response
                                .next_steps
                                .as_ref()
                                .unwrap_or(&"Present the plan to the user".to_string())
                        )
                    }
                    _ => {
                        format!(
                            "Planning Status: {}\n\
                            Model: {}\n\n{}",
                            response.status, model_name, response.step_content
                        )
                    }
                };

                Ok(CallToolResult {
                    content: vec![Content::text(formatted_response)],
                    is_error: Some(false),
                })
            }

            _ => Err(McpError::invalid_params(
                format!("Tool '{}' not found", request.name),
                None,
            )),
        }
    }

    async fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<rmcp::service::RoleServer>,
    ) -> Result<ListPromptsResult, McpError> {
        let prompts = vec![
            Prompt {
                name: "confer".to_string(),
                description: Some("Start a conversation with metacognitive awareness".to_string()),
                arguments: Some(vec![
                    rmcp::model::PromptArgument {
                        name: "message".to_string(),
                        description: Some("What you want to chat about".to_string()),
                        required: Some(true),
                    }
                ]),
            },
            Prompt {
                name: "traced_reasoning".to_string(),
                description: Some("Multi-call step-by-step reasoning with metacognitive monitoring - Generate variable thoughts".to_string()),
                arguments: Some(vec![
                    rmcp::model::PromptArgument {
                        name: "thought".to_string(),
                        description: Some("Initial query or problem to reason through".to_string()),
                        required: Some(true),
                    }
                ]),
            },
            Prompt {
                name: "biased_reasoning".to_string(),
                description: Some("Dual-model reasoning with bias detection".to_string()),
                arguments: Some(vec![
                    rmcp::model::PromptArgument {
                        name: "query".to_string(),
                        description: Some("The question or problem to analyze for bias".to_string()),
                        required: Some(true),
                    }
                ]),
            },
            Prompt {
                name: "planner".to_string(),
                description: Some("Interactive sequential planner - Break down complex tasks through step-by-step planning".to_string()),
                arguments: Some(vec![
                    rmcp::model::PromptArgument {
                        name: "step".to_string(),
                        description: Some("Your planning step or task description".to_string()),
                        required: Some(true),
                    }
                ]),
            },
        ];

        Ok(ListPromptsResult {
            next_cursor: None,
            prompts,
        })
    }

    async fn get_prompt(
        &self,
        request: GetPromptRequestParam,
        _context: RequestContext<rmcp::service::RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        let prompt_text = match request.name.as_ref() {
            "confer" => {
                let message = request
                    .arguments
                    .as_ref()
                    .and_then(|args| args.get("message"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                format!("Start a conversation about: {}", message)
            }

            "traced_reasoning" => {
                let thought = request
                    .arguments
                    .as_ref()
                    .and_then(|args| args.get("thought"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                format!("Begin multi-step reasoning about: {}", thought)
            }

            "biased_reasoning" => {
                let query = request
                    .arguments
                    .as_ref()
                    .and_then(|args| args.get("query"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                format!("Analyze for potential biases: {}", query)
            }

            "planner" => {
                let step = request
                    .arguments
                    .as_ref()
                    .and_then(|args| args.get("step"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                format!("Create an interactive sequential plan for: {}", step)
            }

            "illumination_status" => {
                "Check the current metacognitive monitoring status".to_string()
            }

            _ => {
                return Err(McpError::invalid_params(
                    format!("Prompt '{}' not found", request.name),
                    None,
                ))
            }
        };

        let message = PromptMessage {
            role: PromptMessageRole::User,
            content: PromptMessageContent::text(prompt_text),
        };

        Ok(GetPromptResult {
            description: Some("Metacognitive guidance for illuminated thinking".to_string()),
            messages: vec![message],
        })
    }
}
