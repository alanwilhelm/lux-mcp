use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{self, BufRead, Write};
use tracing::{debug, error, info};
use tracing_subscriber::EnvFilter;

mod metachain;
mod monitoring;
mod models;
mod llm;
mod tools;

use metachain::MetachainEngine;
use monitoring::MetacognitiveMonitor;
use llm::LLMConfig;
use tools::{ChatTool, TracedReasoningTool, BiasedReasoningTool};

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Value,
    id: Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    result: Option<Value>,
    error: Option<JsonRpcError>,
    id: Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    data: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ServerInfo {
    name: String,
    version: String,
    protocol_version: String,
    capabilities: ServerCapabilities,
}

#[derive(Debug, Serialize, Deserialize)]
struct ServerCapabilities {
    tools: Vec<String>,
    prompts: Vec<String>,
}

struct LuxServer {
    metachain: MetachainEngine,
    monitor: MetacognitiveMonitor,
    chat_tool: ChatTool,
    traced_reasoning_tool: TracedReasoningTool,
    biased_reasoning_tool: BiasedReasoningTool,
    config: LLMConfig,
}

impl LuxServer {
    fn new() -> Result<Self> {
        let config = LLMConfig::from_env()?;
        config.validate()?;
        
        let chat_tool = ChatTool::new(config.clone())?;
        let traced_reasoning_tool = TracedReasoningTool::new(config.clone())?;
        let biased_reasoning_tool = BiasedReasoningTool::new(config.clone())?;
        
        Ok(Self {
            metachain: MetachainEngine::new(),
            monitor: MetacognitiveMonitor::new(),
            chat_tool,
            traced_reasoning_tool,
            biased_reasoning_tool,
            config,
        })
    }

    async fn handle_request(&mut self, request: JsonRpcRequest) -> JsonRpcResponse {
        debug!("Handling request: {:?}", request.method);
        
        match request.method.as_str() {
            "initialize" => self.handle_initialize(request).await,
            "initialized" => {
                // MCP protocol: client sends this after successful initialization
                JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: Some(Value::Null),
                    error: None,
                    id: request.id,
                }
            }
            "tools/list" => self.handle_list_tools(request).await,
            "tools/call" => self.handle_call_tool(request).await,
            "prompts/list" => self.handle_list_prompts(request).await,
            "prompts/get" => self.handle_get_prompt(request).await,
            _ => self.error_response(
                request.id,
                -32601,
                format!("Method not found: {}", request.method),
            ),
        }
    }

    async fn handle_initialize(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let server_info = ServerInfo {
            name: "lux".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            protocol_version: "1.0".to_string(),
            capabilities: ServerCapabilities {
                tools: vec![
                    "traced_reasoning".to_string(),
                    "biased_reasoning".to_string(),
                    "lux:chat".to_string(),
                    "illumination_status".to_string(),
                ],
                prompts: vec![
                    "illuminate_thinking".to_string(),
                    "analyze_illumination".to_string(),
                ],
            },
        };

        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(serde_json::to_value(server_info).unwrap()),
            error: None,
            id: request.id,
        }
    }

    async fn handle_list_tools(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let tools = vec![
            serde_json::json!({
                "name": "traced_reasoning",
                "description": "Advanced chain-of-thought reasoning with multi-metric monitoring and real-time interventions based on cutting-edge research",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "The question or problem to reason through"
                        },
                        "model": {
                            "type": "string",
                            "description": "LLM model to use (e.g., o3, gpt4.1, claude)"
                        },
                        "max_steps": {
                            "type": "integer",
                            "description": "Maximum reasoning steps allowed",
                            "default": 10
                        },
                        "temperature": {
                            "type": "number",
                            "description": "Temperature for reasoning (0.0-1.0)",
                            "default": 0.7
                        },
                        "guardrails": {
                            "type": "object",
                            "description": "Guardrail configuration",
                            "properties": {
                                "semantic_drift_check": {
                                    "type": "boolean",
                                    "default": true
                                },
                                "perplexity_monitoring": {
                                    "type": "boolean",
                                    "default": true
                                },
                                "circular_reasoning_detection": {
                                    "type": "boolean",
                                    "default": true
                                }
                            }
                        }
                    },
                    "required": ["query"]
                }
            }),
            serde_json::json!({
                "name": "biased_reasoning",
                "description": "Dual-model reasoning with step-by-step bias verification",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "thought": {
                            "type": "string",
                            "description": "Current thinking step"
                        },
                        "thought_number": {
                            "type": "integer",
                            "description": "Current thought number"
                        },
                        "total_thoughts": {
                            "type": "integer",
                            "description": "Estimated total thoughts"
                        },
                        "primary_model": {
                            "type": "string",
                            "description": "Primary reasoning model",
                            "default": "o3"
                        },
                        "verifier_model": {
                            "type": "string",
                            "description": "Bias verification model",
                            "default": "o4-mini"
                        }
                    },
                    "required": ["thought", "thought_number", "total_thoughts"]
                }
            }),
            serde_json::json!({
                "name": "lux:chat",
                "description": "Simple chat with external LLM models",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "message": {
                            "type": "string",
                            "description": "Message to send to the model"
                        },
                        "model": {
                            "type": "string",
                            "description": "Model to use (e.g., gpt4, claude, llama3)"
                        },
                        "temperature": {
                            "type": "number",
                            "description": "Temperature for response generation",
                            "default": 0.7
                        },
                        "max_tokens": {
                            "type": "integer",
                            "description": "Maximum tokens in response"
                        }
                    },
                    "required": ["message"]
                }
            }),
            serde_json::json!({
                "name": "illumination_status",
                "description": "Check the brightness of your cognitive illumination",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            })
        ];

        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(serde_json::json!({ "tools": tools })),
            error: None,
            id: request.id,
        }
    }

    async fn handle_call_tool(&mut self, request: JsonRpcRequest) -> JsonRpcResponse {
        let params = match request.params.as_object() {
            Some(p) => p,
            None => {
                return self.error_response(
                    request.id,
                    -32602,
                    "Invalid params".to_string(),
                );
            }
        };

        let tool_name = match params.get("name").and_then(|v| v.as_str()) {
            Some(name) => name,
            None => {
                return self.error_response(
                    request.id,
                    -32602,
                    "Missing tool name".to_string(),
                );
            }
        };

        let default_args = Value::Object(Default::default());
        let arguments = params.get("arguments").unwrap_or(&default_args);

        match tool_name {
            "traced_reasoning" => self.handle_traced_reasoning(request.id, arguments).await,
            "biased_reasoning" => self.handle_biased_reasoning(request.id, arguments).await,
            "lux:chat" => self.handle_chat(request.id, arguments).await,
            "illumination_status" => self.handle_monitor_status(request.id).await,
            _ => self.error_response(
                request.id,
                -32602,
                format!("Unknown tool: {}", tool_name),
            ),
        }
    }

    async fn handle_traced_reasoning(&mut self, id: Value, arguments: &Value) -> JsonRpcResponse {
        use crate::tools::traced_reasoning::TracedReasoningRequest;
        
        // Parse request
        let request: TracedReasoningRequest = match serde_json::from_value(arguments.clone()) {
            Ok(req) => req,
            Err(e) => {
                return self.error_response(
                    id,
                    -32602,
                    format!("Invalid traced reasoning parameters: {}", e),
                );
            }
        };
        
        // Save query before moving request
        let query = request.query.clone();
        
        // Execute traced reasoning
        match self.traced_reasoning_tool.trace_reasoning(request).await {
            Ok(response) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(serde_json::json!({
                    "content": [{
                        "type": "text",
                        "text": format!(
                            "# Traced Reasoning Analysis\n\n\
                            **Query**: {}\n\n\
                            **Final Answer**: {}\n\n\
                            **Confidence**: {:.2}\n\n\
                            ## Reasoning Steps ({} steps)\n\n{}\
                            \n## Metrics\n\
                            - Average Confidence: {:.2}\n\
                            - Semantic Coherence: {:.2}\n\
                            - Reasoning Quality: {:.2}\n\n\
                            ## Interventions ({})\n\n{}\
                            \n**Model Used**: {}",
                            query,
                            response.final_answer,
                            response.confidence_score,
                            response.reasoning_steps.len(),
                            response.reasoning_steps.iter()
                                .map(|step| format!(
                                    "### Step {} ({:?}) [Confidence: {:.2}]\n{}\n",
                                    step.step_number,
                                    step.step_type,
                                    step.confidence,
                                    step.thought
                                ))
                                .collect::<Vec<_>>()
                                .join("\n"),
                            response.metrics.average_confidence,
                            response.metrics.semantic_coherence,
                            response.metrics.reasoning_quality,
                            response.interventions.len(),
                            if response.interventions.is_empty() {
                                "None".to_string()
                            } else {
                                response.interventions.iter()
                                    .map(|i| format!(
                                        "- Step {}: {:?} ({:?}) - {}",
                                        i.step, i.intervention_type, i.severity, i.description
                                    ))
                                    .collect::<Vec<_>>()
                                    .join("\n")
                            },
                            response.model_used
                        )
                    }],
                    "reasoning_steps": response.reasoning_steps,
                    "metrics": response.metrics,
                    "interventions": response.interventions
                })),
                error: None,
                id,
            },
            Err(e) => self.error_response(
                id,
                -32603,
                format!("Traced reasoning failed: {}", e),
            ),
        }
    }
    
    async fn handle_biased_reasoning(&mut self, id: Value, arguments: &Value) -> JsonRpcResponse {
        use crate::tools::biased_reasoning::BiasedReasoningRequest;
        
        // Parse request
        let request: BiasedReasoningRequest = match serde_json::from_value(arguments.clone()) {
            Ok(req) => req,
            Err(e) => {
                return self.error_response(
                    id,
                    -32602,
                    format!("Invalid biased reasoning parameters: {}", e),
                );
            }
        };
        
        // Save query before moving request
        let query = request.query.clone();
        
        // Execute biased reasoning
        match self.biased_reasoning_tool.biased_reasoning(request).await {
            Ok(response) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(serde_json::json!({
                    "content": [{
                        "type": "text",
                        "text": format!(
                            "# Biased Reasoning Analysis\n\n\
                            **Query**: {}\n\n\
                            **Final Answer**: {}\n\n\
                            **Models Used**: Primary: {}, Verifier: {}\n\n\
                            ## Reasoning Steps ({} total, {} biased, {} corrected)\n\n{}\
                            \n## Overall Assessment\n\
                            - Total Steps: {}\n\
                            - Biased Steps: {}\n\
                            - Corrected Steps: {}\n\
                            - Average Quality: {:.2}\n\
                            - Most Common Biases: {}\n\n\
                            **Quality Assessment**: {}",
                            query,
                            response.final_answer,
                            response.primary_model_used,
                            response.verifier_model_used,
                            response.overall_assessment.total_steps,
                            response.overall_assessment.biased_steps,
                            response.overall_assessment.corrected_steps,
                            response.reasoning_steps.iter()
                                .map(|step| format!(
                                    "### Step {} [Quality: {:.2}]\n\
                                    **Thought**: {}\n\n\
                                    **Bias Check**: {} ({})\n{}\
                                    {}\n",
                                    step.step_number,
                                    step.step_quality,
                                    step.primary_thought,
                                    if step.bias_check.has_bias { "Bias Detected" } else { "No Bias" },
                                    format!("{:?}", step.bias_check.severity),
                                    if step.bias_check.has_bias {
                                        format!("- Bias Types: {:?}\n- Explanation: {}\n",
                                            step.bias_check.bias_types,
                                            step.bias_check.explanation)
                                    } else {
                                        String::new()
                                    },
                                    if let Some(ref corrected) = step.corrected_thought {
                                        format!("\n**Corrected Thought**: {}\n", corrected)
                                    } else {
                                        String::new()
                                    }
                                ))
                                .collect::<Vec<_>>()
                                .join("\n"),
                            response.overall_assessment.total_steps,
                            response.overall_assessment.biased_steps,
                            response.overall_assessment.corrected_steps,
                            response.overall_assessment.average_quality,
                            response.overall_assessment.most_common_biases
                                .iter()
                                .map(|b| format!("{:?}", b))
                                .collect::<Vec<_>>()
                                .join(", "),
                            response.overall_assessment.final_quality_assessment
                        )
                    }],
                    "reasoning_steps": response.reasoning_steps,
                    "overall_assessment": response.overall_assessment
                })),
                error: None,
                id,
            },
            Err(e) => self.error_response(
                id,
                -32603,
                format!("Biased reasoning failed: {}", e),
            ),
        }
    }
    
    async fn handle_chat(&mut self, id: Value, arguments: &Value) -> JsonRpcResponse {
        use crate::tools::chat::ChatRequest;
        
        // Parse request
        let request: ChatRequest = match serde_json::from_value(arguments.clone()) {
            Ok(req) => req,
            Err(e) => {
                return self.error_response(
                    id,
                    -32602,
                    format!("Invalid chat parameters: {}", e),
                );
            }
        };
        
        // Execute chat
        match self.chat_tool.chat(request).await {
            Ok(response) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(serde_json::json!({
                    "content": [{
                        "type": "text",
                        "text": response.content
                    }],
                    "model": response.model,
                    "usage": response.usage
                })),
                error: None,
                id,
            },
            Err(e) => self.error_response(
                id,
                -32603,
                format!("Chat failed: {}", e),
            ),
        }
    }
    
    async fn handle_metachain(&mut self, id: Value, arguments: &Value) -> JsonRpcResponse {
        // Extract arguments
        let thought = arguments.get("thought")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        let thought_number = arguments.get("thought_number")
            .and_then(|v| v.as_i64())
            .unwrap_or(1) as usize;
        
        let monitor_overthinking = arguments.get("monitor_overthinking")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        // Process thought with monitoring
        let monitoring_signals = if monitor_overthinking {
            Some(self.monitor.analyze_thought(thought, thought_number))
        } else {
            None
        };

        // Generate response
        let response = self.metachain.process_thought(
            thought,
            thought_number,
            monitoring_signals,
        ).await;

        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(serde_json::to_value(response).unwrap()),
            error: None,
            id,
        }
    }

    async fn handle_monitor_status(&self, id: Value) -> JsonRpcResponse {
        let status = self.monitor.get_status();
        
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(serde_json::to_value(status).unwrap()),
            error: None,
            id,
        }
    }

    async fn handle_list_prompts(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let prompts = vec![
            serde_json::json!({
                "name": "illuminate_thinking",
                "description": "Begin illuminating your thought process with cognitive light"
            }),
            serde_json::json!({
                "name": "analyze_illumination", 
                "description": "Analyze the brightness and clarity of your thinking"
            })
        ];

        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(serde_json::json!({ "prompts": prompts })),
            error: None,
            id: request.id,
        }
    }

    async fn handle_get_prompt(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let params = match request.params.as_object() {
            Some(p) => p,
            None => {
                return self.error_response(
                    request.id,
                    -32602,
                    "Invalid params".to_string(),
                );
            }
        };

        let prompt_name = match params.get("name").and_then(|v| v.as_str()) {
            Some(name) => name,
            None => {
                return self.error_response(
                    request.id,
                    -32602,
                    "Missing prompt name".to_string(),
                );
            }
        };

        let prompt = match prompt_name {
            "illuminate_thinking" => serde_json::json!({
                "name": "illuminate_thinking",
                "messages": [{
                    "role": "user",
                    "content": "Let me illuminate this thought process step by step, shining light on each reasoning path. Starting with: {{prompt}}"
                }]
            }),
            "analyze_illumination" => serde_json::json!({
                "name": "analyze_illumination",
                "messages": [{
                    "role": "user", 
                    "content": "Analyze the illumination of this thinking process - check for shadows, circular paths, and clarity: {{reasoning}}"
                }]
            }),
            _ => {
                return self.error_response(
                    request.id,
                    -32602,
                    format!("Unknown prompt: {}", prompt_name),
                );
            }
        };

        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(prompt),
            error: None,
            id: request.id,
        }
    }

    fn error_response(&self, id: Value, code: i32, message: String) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError {
                code,
                message,
                data: None,
            }),
            id,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging to stderr only
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .with_file(true)
        .with_line_number(true)
        .with_writer(std::io::stderr)
        .init();

    info!("Lux MCP Server starting - Illuminating your thinking...");

    let mut server = match LuxServer::new() {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to initialize Lux server: {}", e);
            std::process::exit(1);
        }
    };
    let stdin = io::stdin();
    let stdout = io::stdout();

    // Process JSON-RPC messages over stdio
    let reader = stdin.lock();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        match serde_json::from_str::<JsonRpcRequest>(&line) {
            Ok(request) => {
                let response = server.handle_request(request).await;
                let mut stdout = stdout.lock();
                serde_json::to_writer(&mut stdout, &response)?;
                writeln!(stdout)?;
                stdout.flush()?;
            }
            Err(e) => {
                error!("Failed to parse request: {}", e);
                let error_response = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32700,
                        message: "Parse error".to_string(),
                        data: None,
                    }),
                    id: Value::Null,
                };
                let mut stdout = stdout.lock();
                serde_json::to_writer(&mut stdout, &error_response)?;
                writeln!(stdout)?;
                stdout.flush()?;
            }
        }
    }

    Ok(())
}
