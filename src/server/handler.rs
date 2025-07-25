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
use tracing::{info, debug};
use serde_json::{json, Map, Value};
use anyhow::Context as AnyhowContext;
use std::sync::Arc;

use super::LuxServer;
use crate::tools::{ChatRequest, TracedReasoningRequest, BiasedReasoningRequest};

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
                description: Some("Step-by-step reasoning with metacognitive monitoring".into()),
                input_schema: json_to_arc_map(json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "The question or problem to reason through"
                        },
                        "model": {
                            "type": "string",
                            "description": "Optional model to use for reasoning"
                        },
                        "max_thinking_steps": {
                            "type": "integer",
                            "description": "Maximum number of thinking steps (default: 10)"
                        }
                    },
                    "required": ["query"]
                })),
                annotations: None,
            },
            Tool {
                name: "biased_reasoning".into(),
                description: Some("Dual-model reasoning with bias detection and verification".into()),
                input_schema: json_to_arc_map(json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "The question or problem to analyze for bias"
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
                    serde_json::from_value(serde_json::Value::Object(args))
                        .map_err(|e| McpError::invalid_params(format!("Invalid chat params: {}", e), None))?
                } else {
                    return Err(McpError::invalid_params("Missing arguments for chat", None));
                };
                
                let response = self.chat_tool.chat(req).await
                    .map_err(|e| McpError::internal_error(format!("Chat error: {}", e), None))?;
                
                Ok(CallToolResult {
                    content: vec![Content::text(response.content)],
                    is_error: Some(false),
                })
            }
            
            "traced_reasoning" => {
                let req: TracedReasoningRequest = if let Some(args) = request.arguments {
                    serde_json::from_value(serde_json::Value::Object(args))
                        .map_err(|e| McpError::invalid_params(format!("Invalid reasoning params: {}", e), None))?
                } else {
                    return Err(McpError::invalid_params("Missing arguments for traced reasoning", None));
                };
                
                let response = self.traced_reasoning_tool.trace_reasoning(req).await
                    .map_err(|e| McpError::internal_error(format!("Reasoning error: {}", e), None))?;
                
                Ok(CallToolResult {
                    content: vec![Content::text(response.final_answer)],
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
                
                let response = self.biased_reasoning_tool.biased_reasoning(req).await
                    .map_err(|e| McpError::internal_error(format!("Biased reasoning error: {}", e), None))?;
                
                Ok(CallToolResult {
                    content: vec![Content::text(response.final_answer)],
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
                    content: vec![Content::text(serde_json::to_string_pretty(&status).unwrap())],
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
                name: "illuminate_thinking".to_string(),
                description: Some("Guide your thinking with metacognitive illumination".to_string()),
                arguments: Some(vec![
                    rmcp::model::PromptArgument {
                        name: "topic".to_string(),
                        description: Some("The topic or problem to think about".to_string()),
                        required: Some(true),
                    }
                ]),
            },
            Prompt {
                name: "analyze_illumination".to_string(),
                description: Some("Analyze the current state of your cognitive illumination".to_string()),
                arguments: Some(vec![]),
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
            "lux_illuminate_thinking" => {
                let topic = request.arguments.as_ref()
                    .and_then(|args| args.get("topic"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("the problem at hand");
                
                format!(
                    "🔦 Let's illuminate your thinking about {}.\n\n\
                    Take a moment to:\n\
                    1. Clearly state what you're trying to understand or solve\n\
                    2. Identify any assumptions or biases that might cloud your thinking\n\
                    3. Break down the problem into clear, manageable steps\n\
                    4. Consider alternative perspectives or approaches\n\n\
                    Remember: When thoughts drift into shadow, refocus on the core issue.",
                    topic
                )
            }
            
            "lux_analyze_illumination" => {
                "🔍 Let's analyze your current cognitive illumination:\n\n\
                - Are your thoughts clear and focused, or circling in darkness?\n\
                - Have you been fixating on distractors instead of the core issue?\n\
                - Is the quality of your reasoning improving or degrading?\n\
                - What shadows or biases might be affecting your thinking?\n\n\
                Take a moment to reflect on your mental state and identify areas that need more light."
                .to_string()
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