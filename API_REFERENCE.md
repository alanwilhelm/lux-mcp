# Lux MCP API Reference

## Table of Contents
1. [Overview](#overview)
2. [Revolutionary Direct File Access](#revolutionary-direct-file-access)
3. [MCP Tools](#mcp-tools)
   - [confer](#confer)
   - [traced_reasoning](#traced_reasoning)
   - [biased_reasoning](#biased_reasoning)
   - [planner](#planner)
   - [illumination_status](#illumination_status)
4. [Threading System](#threading-system)
5. [Monitoring System](#monitoring-system)
6. [Database Integration](#database-integration)
7. [Configuration](#configuration)

---

## Overview

Lux MCP is a Model Context Protocol server that provides metacognitive monitoring for AI reasoning. It exposes tools via MCP for conversational AI, step-by-step reasoning, bias detection, and planning.

### Key Features
- **📁 Direct File Access**: Third-party LLMs read files directly, bypassing the orchestrator
- **Metacognitive Monitoring**: Detects circular reasoning, distractor fixation, and quality degradation
- **Conversation Threading**: Maintains context across tool calls with `continuation_id`
- **Synthesis Integration**: Tracks insights and actions across reasoning sessions
- **Quality Metrics**: Monitors confidence, clarity, and coherence
- **Hybrid Storage**: In-memory with optional database persistence

---

## Revolutionary Direct File Access

### How It Works
**Lux MCP enables third-party LLMs (GPT-5, O3, Claude via OpenRouter, etc.) to read files directly on the server side**, completely bypassing the main orchestrator model (Claude Desktop, ChatGPT, etc.).

### Traditional MCP Flow (Inefficient)
```
1. User asks Claude to analyze code
2. Claude uses Read tool to fetch file contents
3. File contents consume Claude's context window
4. Claude passes everything to MCP tool
5. MCP forwards to external LLM with duplicated content
```

### Lux MCP Flow (Efficient)
```
1. User asks Claude to analyze code
2. Claude passes ONLY file paths to Lux MCP
3. Lux MCP reads files directly on server
4. External LLM processes files without consuming Claude's tokens
5. Results returned to Claude for final synthesis
```

### Benefits
- **🚀 Token Efficiency**: Save 50-90% of orchestrator tokens
- **🔒 Privacy**: Sensitive files never enter Claude/ChatGPT's context
- **⚡ Performance**: No serialization/deserialization overhead
- **📈 Scale**: Process massive codebases beyond context limits
- **🛡️ Security**: Files remain within your controlled server environment

---

## MCP Tools

### `confer`
Simple conversational AI with model selection, threading support, and **direct file reading**.

#### Request Parameters
```json
{
  "message": "string",           // Required: The message to send
  "model": "string",             // Optional: Model to use (default: LUX_MODEL_NORMAL)
  "temperature": "number",       // Optional: Temperature 0.0-1.0 (default: 0.7)
  "continuation_id": "string",   // Optional: Thread ID for conversation continuity
  "file_paths": ["string"],      // Optional: Files for the LLM to read directly (server-side)
  "include_file_contents": bool  // Optional: Whether to read files (default: true)
}
```

#### Direct File Access Example
```json
{
  "message": "Review this code for security issues",
  "file_paths": ["/app/auth.js", "/app/config.js"],
  "model": "gpt-5"
}
```
**Note**: The external LLM (GPT-5) reads these files directly on the server. The orchestrator (Claude) never sees the file contents!

#### Response
```json
{
  "response": "string",          // The AI's response
  "model_used": "string",        // Model that was actually used
  "thread_id": "string"          // Thread ID for continuation
}
```

#### Example Usage
```bash
# Start new conversation
{
  "tool": "confer",
  "arguments": {
    "message": "Explain metacognition in AI",
    "model": "gpt-4o"
  }
}

# Continue conversation
{
  "tool": "confer",
  "arguments": {
    "message": "How does this apply to reasoning?",
    "continuation_id": "thread-123"
  }
}
```

---

### `traced_reasoning`
Step-by-step reasoning with metacognitive monitoring and synthesis tracking.

#### Request Parameters
```json
{
  "thought": "string",                    // Required: Query (thought 1) or guidance (2+)
  "thought_number": "integer",            // Required: Current thought number (starts at 1)
  "total_thoughts": "integer",            // Required: Estimated total thoughts needed
  "next_thought_needed": "boolean",       // Required: Whether another thought is needed
  
  "continuation_id": "string",            // Optional: Thread ID for continuity
  "session_id": "string",                 // Optional: Session ID for monitoring
  "model": "string",                      // Optional: Model to use (default: LUX_MODEL_REASONING)
  "temperature": "number",                // Optional: Temperature (default: 0.7)
  
  "is_revision": "boolean",               // Optional: True if revising a previous thought
  "revises_thought": "integer",           // Optional: Which thought is being revised
  "branch_from_thought": "integer",       // Optional: Branching point for alternatives
  "branch_id": "string",                  // Optional: Branch identifier
  "needs_more_thoughts": "boolean",       // Optional: Extend beyond initial estimate
  
  "guardrails": {                         // Optional: Monitoring configuration
    "semantic_drift_check": "boolean",           // Default: true
    "semantic_drift_threshold": "number",        // Default: 0.3
    "perplexity_monitoring": "boolean",          // Default: true
    "perplexity_threshold": "number",            // Default: 50.0
    "circular_reasoning_detection": "boolean",   // Default: true
    "consistency_validation": "boolean",         // Default: true
    "attention_entropy_analysis": "boolean"      // Default: true
  }
}
```

#### Response
```json
{
  "thought_number": "integer",
  "content": "string",
  "thought_type": "string",              // Initial, Exploration, Synthesis, etc.
  "next_thought_needed": "boolean",
  "session_id": "string",
  "thread_id": "string",
  
  "monitoring": {
    "circular_score": "number",          // 0.0-1.0 (lower is better)
    "distractor_alert": "boolean",
    "quality_trend": "string",           // improving, stable, degrading
    "phase": "string",                   // exploration, synthesis, conclusion
    "intervention": "string"             // Optional intervention message
  },
  
  "synthesis": {
    "current_understanding": "string",
    "key_insights": ["string"],
    "next_actions": ["string"],
    "confidence_level": "string",
    "clarity_level": "string",
    "ready_for_conclusion": "boolean"
  },
  
  "metrics": {
    "semantic_coherence": "number",
    "information_density": "number",
    "reasoning_depth": "number",
    "confidence": "number"
  }
}
```

#### Example Usage
```bash
# Start reasoning
{
  "tool": "traced_reasoning",
  "arguments": {
    "thought": "How can we optimize database queries in a microservices architecture?",
    "thought_number": 1,
    "total_thoughts": 5,
    "next_thought_needed": true,
    "model": "o3-pro"
  }
}

# Continue reasoning
{
  "tool": "traced_reasoning",
  "arguments": {
    "thought": "Focus on caching strategies",
    "thought_number": 2,
    "total_thoughts": 5,
    "next_thought_needed": true,
    "session_id": "session-123"
  }
}
```

---

### `biased_reasoning`
Dual-model reasoning with step-by-step bias detection.

#### Request Parameters
```json
{
  "query": "string",                      // Required: The question to analyze
  "session_id": "string",                 // Optional: Session ID to continue
  "new_session": "boolean",               // Optional: Force new session (default: false)
  "max_analysis_rounds": "integer",       // Optional: Max rounds (default: 3)
  "primary_model": "string",              // Optional: Primary reasoning model
  "verifier_model": "string"              // Optional: Bias checking model
}
```

#### Response (per step)
```json
{
  "step_type": "string",                  // Query, Reasoning, BiasAnalysis, Correction, Synthesis
  "step_number": "integer",
  "content": "string",
  "model_used": "string",
  "next_action": "string",                // BiasCheck, ContinueReasoning, Synthesize, Complete
  "session_status": {
    "session_id": "string",
    "total_steps": "integer",
    "reasoning_steps": "integer",
    "bias_checks": "integer",
    "corrections_made": "integer",
    "current_round": "integer",
    "max_rounds": "integer"
  }
}
```

#### Example Usage
```bash
# Start bias analysis
{
  "tool": "biased_reasoning",
  "arguments": {
    "query": "What are the best programming languages for beginners?",
    "max_analysis_rounds": 3
  }
}

# Continue session
{
  "tool": "biased_reasoning",
  "arguments": {
    "query": "What are the best programming languages for beginners?",
    "session_id": "bias-session-123"
  }
}
```

---

### `planner`
Interactive sequential planning with LLM-generated steps.

#### Request Parameters
```json
{
  "step": "string",                       // Required: Task (step 1) or planning content (2+)
  "step_number": "integer",               // Required: Current step number (starts at 1)
  "total_steps": "integer",               // Required: Estimated total steps
  "next_step_required": "boolean",        // Required: Whether another step is needed
  
  "model": "string",                      // Optional: Model to use
  "temperature": "number",                // Optional: Temperature (default: 0.7)
  
  "is_branch_point": "boolean",          // Optional: True if branching
  "branch_from_step": "integer",         // Optional: Branching point
  "branch_id": "string",                 // Optional: Branch identifier
  
  "is_step_revision": "boolean",         // Optional: True if revising
  "revises_step_number": "integer",      // Optional: Step being revised
  "more_steps_needed": "boolean"         // Optional: Extend beyond estimate
}
```

#### Response
```json
{
  "step_number": "integer",
  "content": "string",
  "next_step_required": "boolean",
  "total_steps": "integer",
  "session_id": "string",
  
  "metadata": {
    "branch_id": "string",
    "is_revision": "boolean",
    "planning_depth": "integer"
  }
}
```

#### Example Usage
```bash
# Start planning
{
  "tool": "planner",
  "arguments": {
    "step": "Design a scalable authentication system",
    "step_number": 1,
    "total_steps": 7,
    "next_step_required": true
  }
}

# Continue planning
{
  "tool": "planner",
  "arguments": {
    "step": "Define user roles and permissions model",
    "step_number": 2,
    "total_steps": 7,
    "next_step_required": true
  }
}
```

---

### `illumination_status`
Check the current metacognitive monitoring status.

#### Request Parameters
None required.

#### Response
```json
{
  "status": "string",                    // active, idle
  "monitoring": {
    "cognitive_load": "number",          // 0.0-1.0
    "current_phase": "string",           // exploration, synthesis, conclusion
    "circular_reasoning_score": "number",
    "distractor_fixation_score": "number",
    "quality_metrics": {
      "coherence": "number",
      "information_density": "number",
      "relevance": "number",
      "trend": "string"
    },
    "intervention_history": [
      {
        "thought_number": "integer",
        "intervention_type": "string",
        "reason": "string"
      }
    ]
  },
  "threads": {
    "active_count": "integer",
    "oldest_age_minutes": "number",
    "average_turns": "number"
  }
}
```

---

## Threading System

The threading system maintains conversation context across tool calls.

### Key Components

#### ThreadManager
Manages conversation threads with automatic expiration.

```rust
pub struct ThreadManager {
    threads: Arc<Mutex<HashMap<Uuid, ThreadContext>>>,
    ttl: Duration,  // Default: 3 hours
}

// Key methods:
- create_thread() -> String
- get_thread(thread_id: &str) -> Option<ThreadContext>
- add_turn(thread_id: &str, role: Role, content: String)
- continue_from(from_id: &str, to_id: &str)
- cleanup_expired() -> usize
```

#### ThreadContext
Stores conversation history and metadata.

```rust
pub struct ThreadContext {
    thread_id: String,
    created_at: DateTime<Utc>,
    last_accessed: DateTime<Utc>,
    conversation_history: Vec<ConversationTurn>,
    metadata: serde_json::Value,
    quality_metrics: Option<QualityMetrics>,
}
```

#### ContextReconstructor
Intelligently reconstructs context within token limits.

```rust
pub struct ContextReconstructor {
    max_tokens: usize,  // Default: 4000
    prioritize_recent: bool,  // Default: true
}

// Reconstruction strategy:
1. Include all turns from last 5 minutes
2. Include turns with high quality scores
3. Summarize older context if space allows
```

---

## Monitoring System

### MetacognitiveMonitor
Core monitoring engine that tracks reasoning quality.

```rust
pub struct MetacognitiveMonitor {
    thought_history: VecDeque<String>,
    intervention_history: Vec<InterventionRecord>,
    quality_scores: VecDeque<f64>,
    circular_detector: CircularReasoningDetector,
    distractor_detector: DistractorFixationDetector,
    quality_detector: QualityDegradationDetector,
}

// Key methods:
- process_thought(thought: &str) -> MonitoringSignals
- should_intervene() -> bool
- get_intervention_message() -> Option<String>
- reset_session()
```

### Detection Algorithms

#### Circular Reasoning Detection
- Tracks semantic similarity between thoughts
- Triggers when similarity > 85% for 3+ consecutive thoughts
- Uses cosine similarity on TF-IDF vectors

#### Distractor Fixation Detection
- Monitors relevance to original query
- Triggers when relevance < 30% for 2+ thoughts
- Tracks topic drift using keyword analysis

#### Quality Degradation Detection
- Analyzes coherence, information density, relevance
- Triggers when quality drops > 40% from baseline
- Uses rolling average with exponential decay

---

## Configuration

### Environment Variables

#### Required (at least one)
- `OPENAI_API_KEY`: For OpenAI models
- `OPENROUTER_API_KEY`: For OpenRouter models

#### Model Configuration
- `LUX_MODEL_REASONING`: Main reasoning model (default: "gpt-5")
- `LUX_MODEL_NORMAL`: Main normal model (default: "gpt-5")
- `LUX_MODEL_MINI`: Mini model for fast tasks (default: "gpt-5-mini")

#### Named Model Aliases (Optional)
- `LUX_MODEL_OPUS`: Maps 'opus' to specific model (default: "anthropic/claude-4.1-opus")
- `LUX_MODEL_SONNET`: Maps 'sonnet' to specific model (default: "anthropic/claude-4-sonnet")
- `LUX_MODEL_GROK`: Maps 'grok' to specific model (default: "x-ai/grok-beta")

#### Logging
- `RUST_LOG`: Log level (error, warn, info, debug, trace)

### Model Aliases
```rust
// Convenience aliases resolved automatically:
"gpt4" -> "gpt-4"
"gpt4.1" -> "gpt-4-turbo-preview"
"claude" -> "anthropic/claude-3-opus"
"gemini" -> "google/gemini-2.5-pro"
"llama3" -> "meta-llama/llama-3-70b"
"o3" -> "o3-2025-01-17"
"o3-pro" -> "o3-pro-2025-06-10"
"o4-mini" -> "o4-mini-2025-04-16"
```

### Claude Desktop Configuration
```json
{
  "mcpServers": {
    "lux": {
      "command": "/path/to/lux-mcp",
      "env": {
        "OPENAI_API_KEY": "sk-...",
        "LUX_MODEL_REASONING": "gpt-5",
        "LUX_MODEL_NORMAL": "gpt-5",
        "LUX_MODEL_MINI": "gpt-5-mini",
        "RUST_LOG": "info"
      }
    }
  }
}
```

---

## Error Handling

All tools return errors in MCP format:
```json
{
  "error": {
    "code": -32603,
    "message": "Error description",
    "data": {
      "details": "Additional context"
    }
  }
}
```

Common error codes:
- `-32600`: Invalid request
- `-32601`: Method not found
- `-32602`: Invalid params
- `-32603`: Internal error

---

## Performance Considerations

### Token Limits
- Default max tokens: 10,000 for all operations
- O3 models: Support up to 32,768 tokens
- O4 models: Require high token limits due to reasoning overhead
- Context reconstruction: Limited to 4,000 tokens by default

### Timeouts
- O3 models: 30 seconds to 5 minutes response time
- Standard models: 5-30 seconds
- Database operations: 5 second timeout

### Memory Management
- Threads expire after 3 hours (configurable)
- Session monitors expire after 30 minutes
- Automatic cleanup runs every 10 minutes
- Database checkpoints every 5 minutes

---

## Examples

### Complete Reasoning Session
```python
# Step 1: Start reasoning
response1 = call_tool("traced_reasoning", {
    "thought": "How can we improve code review processes?",
    "thought_number": 1,
    "total_thoughts": 4,
    "next_thought_needed": True
})

# Step 2: Continue with guidance
response2 = call_tool("traced_reasoning", {
    "thought": "Focus on automation tools",
    "thought_number": 2,
    "total_thoughts": 4,
    "next_thought_needed": True,
    "session_id": response1["session_id"]
})

# Step 3: Branch to explore alternative
response3 = call_tool("traced_reasoning", {
    "thought": "What about human factors?",
    "thought_number": 3,
    "total_thoughts": 4,
    "next_thought_needed": True,
    "session_id": response1["session_id"],
    "branch_from_thought": 2,
    "branch_id": "human-factors"
})

# Step 4: Synthesize
response4 = call_tool("traced_reasoning", {
    "thought": "Combine both approaches",
    "thought_number": 4,
    "total_thoughts": 4,
    "next_thought_needed": False,
    "session_id": response1["session_id"]
})
```

### Threaded Conversation
```python
# Start conversation
response1 = call_tool("confer", {
    "message": "What is quantum computing?"
})

thread_id = response1["thread_id"]

# Continue in same thread
response2 = call_tool("confer", {
    "message": "How does it differ from classical computing?",
    "continuation_id": thread_id
})

# Switch to reasoning in same thread
response3 = call_tool("traced_reasoning", {
    "thought": "Let's explore quantum algorithms",
    "thought_number": 1,
    "total_thoughts": 3,
    "next_thought_needed": True,
    "continuation_id": thread_id
})
```

---

## Troubleshooting

### Common Issues

1. **Empty responses from O4 models**
   - Increase max_tokens to at least 10,000
   - O4 models use tokens for internal reasoning

2. **Slow responses from O3 models**
   - Normal behavior - O3 does deep reasoning
   - Expect 30 seconds to 5 minutes

3. **Thread context not persisting**
   - Ensure continuation_id is passed correctly
   - Check thread hasn't expired (3 hour TTL)

4. **Model not found errors**
   - Check API key is set for the provider
   - Verify model name or use an alias
   - Some models require specific API access

---

## Version History

### v0.1.0 (Current)
- Initial release with MCP 1.0 support
- Four main tools: confer, traced_reasoning, biased_reasoning, planner
- Threading system with continuation_id
- Metacognitive monitoring
- Optional database persistence
- Support for O3/O4 reasoning models