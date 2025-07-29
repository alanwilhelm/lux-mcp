use rmcp::{
    ServerHandler,
    model::{
        ServerInfo, ListToolsResult, CallToolResult, CallToolRequestParam,
        PaginatedRequestParam, Prompt, ListPromptsResult,
        GetPromptRequestParam, GetPromptResult, PromptMessage, PromptMessageContent,
        PromptMessageRole, Tool, Content,
    },
    service::RequestContext,
    Error as McpError
};
use tracing::{info, debug, error};
use serde_json::{json, Map, Value};
use anyhow::Context as AnyhowContext;
use std::sync::Arc;

use super::LuxServer;
use crate::tools::{
    ChatRequest, TracedReasoningRequest, BiasedReasoningRequest, PlannerRequest,
    StepType, NextAction
};

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
            instructions: Some("Illuminate your thinking - A metacognitive monitoring MCP server".into()),
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
                        "max_tokens": {
                            "type": "integer",
                            "description": "Optional max tokens for response"
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
                            "description": "Optional guardrail configuration for monitoring",
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
                let req: ChatRequest = if let Some(args) = request.arguments {
                    match serde_json::from_value(serde_json::Value::Object(args)) {
                        Ok(req) => req,
                        Err(e) => {
                            error!("Failed to parse chat request: {}", e);
                            return Err(McpError::invalid_params(format!("Invalid chat params: {}", e), None));
                        }
                    }
                } else {
                    error!("Missing arguments for confer tool");
                    return Err(McpError::invalid_params("Missing arguments for chat", None));
                };
                
                info!("Processing confer request with message: {}", req.message);
                match self.chat_tool.chat(req).await {
                    Ok(response) => {
                        info!("Confer request successful - Model used: {}", response.model);
                        
                        // Format response with explicit action directives
                        let formatted_response = format!(
                            "🔍 **LUX ANALYSIS COMPLETE** 🔍\n\n\
                            ⚠️ **IMPORTANT: You MUST now SYNTHESIZE and ACT on the following analysis from the {} model:**\n\n\
                            ---\n\n{}\n\n---\n\n\
                            🎯 **REQUIRED ACTIONS:**\n\
                            1. ✅ SUMMARIZE the key findings\n\
                            2. ✅ IDENTIFY the most critical recommendations\n\
                            3. ✅ CREATE an actionable plan based on this analysis\n\
                            4. ✅ RESPOND with clear next steps for the user\n\n\
                            ⚡ **DO NOT just acknowledge this output - YOU MUST PROCESS AND ACT ON IT!** ⚡",
                            response.model,
                            response.content
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
                    serde_json::from_value(serde_json::Value::Object(args))
                        .map_err(|e| McpError::invalid_params(format!("Invalid reasoning params: {}", e), None))?
                } else {
                    return Err(McpError::invalid_params("Missing arguments for traced reasoning", None));
                };
                
                info!("Processing traced reasoning - thought {} of {}", req.thought_number, req.total_thoughts);
                if let Some(ref model) = req.model {
                    info!("Using specified model: {}", model);
                }
                
                // Lock the mutex to access the mutable traced reasoning tool
                let mut reasoning_tool = self.traced_reasoning_tool.lock().await;
                let response = reasoning_tool.process_thought(req).await
                    .map_err(|e| McpError::internal_error(format!("Reasoning error: {}", e), None))?;
                
                // Drop the lock immediately after use
                drop(reasoning_tool);
                
                info!("Response model_used: {:?}", response.model_used);
                
                // Format the response based on status
                let formatted_response = match response.status.as_str() {
                    "thinking" => {
                        // Always show the model being used
                        let model_name = response.model_used.as_ref()
                            .cloned()
                            .unwrap_or_else(|| "ERROR: Model not specified".to_string());
                        let model_display = format!("Model: {}\n", model_name);
                        
                        format!(
                            "🧠 **REASONING THOUGHT** 🧠\n\n\
                            Thought {} of {}: [Type: {:?}]\n\
                            {}\
                            Confidence: {:.2}\n\n\
                            ---\n\n\
                            {}\n\n\
                            ---\n\n\
                            📊 **Metrics:**\n\
                            • Semantic Coherence: {:.2}\n\
                            • Current Confidence: {:.2}\n\
                            • Interventions Count: {}\n\n\
                            ➡️ **Next Action:** {}\n\n\
                            Use traced_reasoning again with thought_number: {} to continue.",
                            response.thought_number,
                            response.total_thoughts,
                            response.thought_type,
                            model_display,
                            response.metadata.current_confidence,
                            response.thought_content,
                            response.metadata.semantic_coherence,
                            response.metadata.current_confidence,
                            response.metadata.interventions_count,
                            response.next_steps.as_ref().unwrap_or(&"Continue reasoning".to_string()),
                            response.thought_number + 1
                        )
                    },
                    "intervention_needed" => {
                        let intervention = response.intervention.as_ref().unwrap();
                        // Always show the model being used
                        let model_name = response.model_used.as_ref()
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
                    },
                    "conclusion_reached" => {
                        // Always show the model being used
                        let model_name = response.model_used.as_ref()
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
                            response.final_answer.as_ref().unwrap_or(&response.thought_content),
                            response.overall_metrics.as_ref().map(|m| m.total_steps).unwrap_or(response.thought_number),
                            response.overall_metrics.as_ref().map(|m| m.average_confidence).unwrap_or(0.0),
                            response.overall_metrics.as_ref().map(|m| m.reasoning_quality).unwrap_or(0.0),
                            response.overall_metrics.as_ref().map(|m| m.semantic_coherence).unwrap_or(0.0),
                            response.next_steps.as_ref().unwrap_or(&"Present the reasoning to the user".to_string())
                        )
                    },
                    _ => {
                        // Always show the model being used
                        let model_name = response.model_used.as_ref()
                            .cloned()
                            .unwrap_or_else(|| "ERROR: Model not specified".to_string());
                        format!(
                            "Reasoning Status: {}\n\n{}\n\nModel: {}",
                            response.status,
                            response.thought_content,
                            model_name
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
                    serde_json::from_value(serde_json::Value::Object(args))
                        .map_err(|e| McpError::invalid_params(format!("Invalid biased reasoning params: {}", e), None))?
                } else {
                    return Err(McpError::invalid_params("Missing arguments for biased reasoning", None));
                };
                
                let response = self.biased_reasoning_tool.process_step(req).await
                    .map_err(|e| McpError::internal_error(format!("Biased reasoning error: {}", e), None))?;
                
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
                    },
                    StepType::Reasoning => {
                        let metadata = response.reasoning_metadata.as_ref();
                        format!(
                            "🧠 **Reasoning Step {}**\n\n\
                            {}\n\n\
                            Session ID: {}\n\
                            Model: {}\n\
                            Thinking Time: {}ms\n\
                            Depth: {}\n\
                            Next: {:?}\n\n\
                            Session Progress: {}/{} steps",
                            response.step_number,
                            response.content,
                            response.session_id,
                            response.model_used,
                            metadata.map(|m| m.thinking_time_ms).unwrap_or(0),
                            metadata.map(|m| m.reasoning_depth.clone()).unwrap_or_else(|| "unknown".to_string()),
                            response.next_action,
                            response.session_status.reasoning_steps,
                            response.session_status.total_steps
                        )
                    },
                    StepType::BiasAnalysis => {
                        format!(
                            "🔍 **Bias Analysis Step {}**\n\n\
                            {}\n\n\
                            Session ID: {}\n\
                            Model: {}\n\
                            Next: {:?}\n\n\
                            Quality Score: {:.2}",
                            response.step_number,
                            response.content,
                            response.session_id,
                            response.model_used,
                            response.next_action,
                            response.session_status.overall_quality
                        )
                    },
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
                    },
                    StepType::Guidance => {
                        format!(
                            "📝 **User Guidance Step {}**\n\n\
                            {}\n\n\
                            Next: {:?}",
                            response.step_number,
                            response.content,
                            response.next_action
                        )
                    },
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
                    },
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
                        serde_json::to_string_pretty(&status)
                            .map_err(|e| McpError::internal_error(format!("Failed to serialize status: {}", e), None))?
                    )],
                    is_error: Some(false),
                })
            }
            
            "planner" => {
                let req: PlannerRequest = if let Some(args) = request.arguments {
                    serde_json::from_value(serde_json::Value::Object(args))
                        .map_err(|e| McpError::invalid_params(format!("Invalid planner params: {}", e), None))?
                } else {
                    return Err(McpError::invalid_params("Missing arguments for planner", None));
                };
                
                info!("Processing planner request - step {} of {}", req.step_number, req.total_steps);
                
                // Lock the mutex to access the mutable planner tool
                let mut planner = self.planner_tool.lock().await;
                let response = planner.create_plan(req).await
                    .map_err(|e| McpError::internal_error(format!("Planner error: {}", e), None))?;
                
                // Drop the lock immediately after use
                drop(planner);
                
                // Always show the actual model being used
                let model_name = response.model_used.as_ref()
                    .cloned()
                    .unwrap_or_else(|| "ERROR: Model not specified".to_string());
                
                // Format the response based on status
                let formatted_response = match response.status.as_str() {
                    "pause_for_deep_thinking" => {
                        format!(
                            "🧠 **DEEP THINKING REQUIRED** 🧠\n\n\
                            Step {} of {}: {}\n\
                            Model: {}\n\n\
                            ---\n\n\
                            ⚠️ **MANDATORY PAUSE FOR REFLECTION**\n\n\
                            {}\n\n\
                            🔍 **Required Thinking:**\n{}\n\n\
                            ⏸️ **DO NOT PROCEED until you have completed this deep analysis!**",
                            response.step_number,
                            response.total_steps,
                            response.step_content,
                            model_name,
                            response.next_steps.as_ref().unwrap_or(&"Continue planning...".to_string()),
                            response.required_thinking.as_ref()
                                .map(|rt| rt.iter().map(|t| format!("• {}", t)).collect::<Vec<_>>().join("\n"))
                                .unwrap_or_default()
                        )
                    },
                    "pause_for_planner" => {
                        format!(
                            "📋 **PLANNING STEP RECORDED** 📋\n\n\
                            Step {} of {}: {}\n\
                            Model: {}\n\n\
                            ---\n\n\
                            📊 **Planning Progress:**\n\
                            • Steps completed: {}\n\
                            • Branches explored: {}\n\
                            • Is revision: {}\n\
                            • Is branch: {}\n\n\
                            ➡️ **Next Action:** {}\n\n\
                            Use the planner tool again with step_number: {} to continue.",
                            response.step_number,
                            response.total_steps,
                            response.step_content,
                            model_name,
                            response.metadata.step_history_length,
                            response.metadata.branches.len(),
                            if response.metadata.is_step_revision { "Yes" } else { "No" },
                            if response.metadata.is_branch_point { "Yes" } else { "No" },
                            response.next_steps.as_ref().unwrap_or(&"Continue planning".to_string()),
                            response.step_number + 1
                        )
                    },
                    "planning_complete" => {
                        format!(
                            "✅ **PLANNING COMPLETE** ✅\n\n\
                            Model: {}\n\n\
                            {}\n\n\
                            ---\n\n\
                            📋 **Instructions:**\n{}\n\n\
                            🎯 **Ready for Implementation!**",
                            model_name,
                            response.plan_summary.as_ref().unwrap_or(&"Plan completed".to_string()),
                            response.next_steps.as_ref().unwrap_or(&"Present the plan to the user".to_string())
                        )
                    },
                    _ => {
                        format!(
                            "Planning Status: {}\n\
                            Model: {}\n\n{}",
                            response.status,
                            model_name,
                            response.step_content
                        )
                    }
                };
                
                Ok(CallToolResult {
                    content: vec![Content::text(formatted_response)],
                    is_error: Some(false),
                })
            }
            
            _ => Err(McpError::invalid_params(format!("Tool '{}' not found", request.name), None)),
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
                let message = request.arguments.as_ref()
                    .and_then(|args| args.get("message"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                
                format!("Start a conversation about: {}", message)
            }
            
            "traced_reasoning" => {
                let thought = request.arguments.as_ref()
                    .and_then(|args| args.get("thought"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                
                format!("Begin multi-step reasoning about: {}", thought)
            }
            
            "biased_reasoning" => {
                let query = request.arguments.as_ref()
                    .and_then(|args| args.get("query"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                
                format!("Analyze for potential biases: {}", query)
            }
            
            "planner" => {
                let step = request.arguments.as_ref()
                    .and_then(|args| args.get("step"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                
                format!("Create an interactive sequential plan for: {}", step)
            }
            
            "illumination_status" => {
                "Check the current metacognitive monitoring status".to_string()
            }
            
            _ => return Err(McpError::invalid_params(format!("Prompt '{}' not found", request.name), None)),
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