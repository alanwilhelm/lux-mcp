# Sequential Thinking Tools Documentation

## Overview

Lux MCP provides two sequential thinking tools that enable step-by-step reasoning and problem-solving. These tools are inspired by Anthropic's sequential-thinking-mcp but offer enhanced capabilities and tighter integration with the Lux ecosystem.

## Table of Contents
1. [Introduction](#introduction)
2. [Tool Comparison](#tool-comparison)
3. [Sequential Thinking (Simple)](#sequential-thinking-simple)
4. [Sequential Thinking External (AI-Powered)](#sequential-thinking-external-ai-powered)
5. [Architecture](#architecture)
6. [Best Practices](#best-practices)
7. [Migration Guide](#migration-guide)

## Introduction

Sequential thinking is a cognitive approach where complex problems are broken down into manageable steps, each building upon the previous one. This methodology is particularly effective for:

- **Problem Decomposition**: Breaking complex challenges into smaller, solvable pieces
- **Iterative Refinement**: Revising and improving thoughts as understanding deepens
- **Alternative Exploration**: Branching to explore different solution paths
- **Auditable Reasoning**: Creating a clear trace of the thinking process

## Tool Comparison

Lux MCP offers three main reasoning tools, each with different capabilities:

| Feature | `sequential_thinking` | `sequential_thinking_external` | `traced_reasoning` |
|---------|----------------------|-------------------------------|-------------------|
| **Purpose** | Manual thought organization | AI-assisted sequential reasoning | Deep analysis with monitoring |
| **LLM Usage** | None | Optional (configurable) | Always required |
| **State Management** | Session-based | Session-based | Session-based |
| **Branching Support** | Full | Full | Limited |
| **Revision Capability** | Yes | Yes | Yes |
| **Confidence Scoring** | No | Yes (heuristic) | Yes (comprehensive) |
| **Metacognitive Monitoring** | No | No | Yes |
| **Quality Guardrails** | No | No | Yes (semantic drift, perplexity, etc.) |
| **Synthesis Engine** | No | No | Yes |
| **Response Time** | Instant | Fast (single LLM call) | Slower (monitoring overhead) |
| **Cost** | Free | Low (single call per step) | Higher (multiple analyses) |
| **Best Use Cases** | Auditable traces, sensitive data | Guided AI reasoning | Critical analysis, research |

## Sequential Thinking (Simple)

### Purpose
A lightweight state tracker for organizing thoughts without any LLM involvement. Perfect for scenarios requiring full control, auditability, or handling sensitive data.

### Key Features
- **Zero LLM Dependency**: No external API calls or model dependencies
- **Full State Control**: Complete control over thought content and structure
- **Branch Management**: Support for exploring alternative reasoning paths
- **Revision Tracking**: Update and improve previous thoughts
- **Session Persistence**: Maintain context across multiple interactions
- **Logging Control**: Respects `DISABLE_THOUGHT_LOGGING` environment variable

### Use Cases
1. **Compliance & Auditing**: Create verifiable reasoning traces for regulatory requirements
2. **Sensitive Data Processing**: Handle PII or confidential information locally
3. **Educational Tools**: Teach step-by-step problem-solving methodologies
4. **Low-Latency Applications**: Instant response without API overhead
5. **Offline Operation**: Full functionality without internet connectivity

### API Reference

#### Request Structure
```rust
pub struct SequentialThinkingRequest {
    pub thought: String,              // The content of the current thought
    pub thought_number: u32,          // Current step number (1-indexed)
    pub total_thoughts: u32,          // Estimated total steps needed
    pub next_thought_needed: bool,    // Whether continuation is required
    
    // Optional fields
    pub is_revision: bool,            // Indicates revision of previous thought
    pub revises_thought: Option<u32>, // Which thought is being revised
    pub branch_from_thought: Option<u32>, // Branch point for alternatives
    pub branch_id: Option<String>,    // Unique identifier for the branch
    pub needs_more_thoughts: bool,    // Dynamic expansion beyond estimate
    pub session_id: Option<String>,   // Session identifier for continuity
}
```

#### Response Structure
```rust
pub struct SequentialThinkingResponse {
    pub thought_number: u32,          // Echo of current thought number
    pub total_thoughts: u32,          // Potentially adjusted total
    pub next_thought_needed: bool,    // Whether to continue
    pub branches: Vec<String>,        // List of active branch IDs
    pub thought_history_length: usize, // Total thoughts in session
    pub session_id: Option<String>,   // Session ID if provided
    pub status: String,               // "recorded", "revision", "branch", "complete"
}
```

### Example Workflow

```json
// Step 1: Initial problem statement
{
  "thought": "Design a caching strategy for our API",
  "thought_number": 1,
  "total_thoughts": 5,
  "next_thought_needed": true
}

// Step 2: Analysis
{
  "thought": "Identify cache invalidation patterns",
  "thought_number": 2,
  "total_thoughts": 5,
  "next_thought_needed": true,
  "session_id": "cache-design-001"
}

// Step 3: Revision of step 2
{
  "thought": "Consider event-driven invalidation instead",
  "thought_number": 3,
  "total_thoughts": 5,
  "next_thought_needed": true,
  "is_revision": true,
  "revises_thought": 2,
  "session_id": "cache-design-001"
}

// Step 4: Branch to explore alternative
{
  "thought": "Explore Redis as cache backend",
  "thought_number": 4,
  "total_thoughts": 6,  // Adjusted estimate
  "next_thought_needed": true,
  "branch_from_thought": 3,
  "branch_id": "redis-approach",
  "session_id": "cache-design-001"
}
```

## Sequential Thinking External (AI-Powered)

### Purpose
Combines the structural benefits of sequential thinking with AI-powered thought generation. Provides guided reasoning while maintaining full control over the process flow.

### Key Features
- **Flexible LLM Integration**: Toggle AI assistance on/off per request
- **Model Selection**: Choose from any supported model (GPT-4, O3, Claude, etc.)
- **Confidence Scoring**: Heuristic-based confidence assessment
- **Context Preservation**: Maintains conversation history for coherent reasoning
- **Reasoning Hints**: AI-generated suggestions for next steps
- **Temperature Control**: Fine-tune creativity vs. consistency

### Use Cases
1. **Interactive Problem Solving**: Guide AI through specific reasoning paths
2. **Code Architecture Planning**: Generate structured technical designs
3. **Research & Analysis**: Explore topics with AI assistance
4. **Creative Brainstorming**: Generate ideas with structural organization
5. **Documentation Generation**: Create step-by-step explanations

### API Reference

#### Request Structure
```rust
pub struct SequentialThinkingExternalRequest {
    pub thought: String,              // Step 1: problem, Step 2+: guidance
    pub thought_number: u32,          // Current step number
    pub total_thoughts: u32,          // Estimated total steps
    pub next_thought_needed: bool,    // Whether to continue
    
    // Revision and branching
    pub is_revision: bool,
    pub revises_thought: Option<u32>,
    pub branch_from_thought: Option<u32>,
    pub branch_id: Option<String>,
    pub needs_more_thoughts: bool,
    
    // AI configuration
    pub session_id: Option<String>,
    pub model: Option<String>,        // Model selection (e.g., "gpt-4o", "o3-mini")
    pub temperature: f32,             // 0.0-1.0, default 0.7
    pub use_llm: bool,               // Enable/disable AI (default: true)
}
```

#### Response Structure
```rust
pub struct SequentialThinkingExternalResponse {
    pub thought_number: u32,
    pub total_thoughts: u32,
    pub next_thought_needed: bool,
    pub thought_content: String,      // Generated or provided thought
    pub branches: Vec<String>,
    pub thought_history_length: usize,
    pub status: String,               // "thinking", "revision", "branch", "complete"
    
    // AI-specific fields
    pub session_id: Option<String>,
    pub model_used: Option<String>,   // Actual model used
    pub confidence: Option<f32>,      // 0.0-1.0 confidence score
    pub reasoning_hint: Option<String>, // Suggestion for next step
}
```

### Confidence Scoring Algorithm

The tool uses a heuristic approach to estimate confidence:

```rust
fn estimate_confidence(content: &str) -> f32 {
    let mut confidence: f32 = 0.7;  // Base confidence
    
    // Increase for certainty indicators
    if content.contains("clearly") || 
       content.contains("definitely") || 
       content.contains("certain") {
        confidence += 0.1;
    }
    
    // Decrease for uncertainty
    if content.contains("however") || 
       content.contains("but") || 
       content.contains("although") {
        confidence -= 0.1;
    }
    
    if content.contains("might") || 
       content.contains("possibly") || 
       content.contains("perhaps") {
        confidence -= 0.15;
    }
    
    confidence.max(0.2).min(0.95)  // Clamp to valid range
}
```

### Example Workflow

```json
// Step 1: AI analyzes the problem
{
  "thought": "How do I implement OAuth2 with PKCE for a mobile app?",
  "thought_number": 1,
  "total_thoughts": 5,
  "next_thought_needed": true,
  "model": "gpt-4o",
  "temperature": 0.7
}
// Response: Detailed analysis of OAuth2 PKCE requirements...

// Step 2: Guide AI to focus area
{
  "thought": "Focus on the token refresh mechanism",
  "thought_number": 2,
  "total_thoughts": 5,
  "next_thought_needed": true,
  "session_id": "oauth-impl-42"
}
// Response: Deep dive into refresh token handling...

// Step 3: Explore alternative with branch
{
  "thought": "What about using device flow instead?",
  "thought_number": 3,
  "total_thoughts": 6,
  "next_thought_needed": true,
  "branch_from_thought": 2,
  "branch_id": "device-flow",
  "session_id": "oauth-impl-42"
}
// Response: Analysis of device flow alternative...
```

## Architecture

### Component Overview

```
┌─────────────────────────────────────────────────┐
│                 LUX MCP Server                  │
├─────────────────────────────────────────────────┤
│                                                 │
│  ┌───────────────┐  ┌────────────────────────┐ │
│  │  Sequential   │  │ Sequential Thinking    │ │
│  │   Thinking    │  │      External          │ │
│  │   (Simple)    │  │    (AI-Powered)        │ │
│  └───────┬───────┘  └──────────┬─────────────┘ │
│          │                      │               │
│  ┌───────▼──────────────────────▼──────────┐   │
│  │        Session Manager (Arc<Mutex>)      │   │
│  │  ┌─────────────┐  ┌─────────────────┐   │   │
│  │  │  Sessions   │  │  Thought        │   │   │
│  │  │  HashMap    │  │  History        │   │   │
│  │  └─────────────┘  └─────────────────┘   │   │
│  └──────────────────────────────────────────┘   │
│                                                 │
│  ┌──────────────────────────────────────────┐   │
│  │           LLM Integration                │   │
│  │  ┌──────────┐  ┌────────────────────┐   │   │
│  │  │  OpenAI  │  │   OpenRouter       │   │   │
│  │  │  Client  │  │     Client         │   │   │
│  │  └──────────┘  └────────────────────┘   │   │
│  └──────────────────────────────────────────┘   │
└─────────────────────────────────────────────────┘
```

### Session Management

Both tools use Arc<Mutex<HashMap>> for thread-safe session storage:

```rust
pub struct SequentialThinkingTool {
    sessions: Arc<Mutex<HashMap<String, SessionState>>>,
}

struct SessionState {
    thought_history: Vec<ThoughtData>,
    branches: HashMap<String, Vec<ThoughtData>>,
}
```

### Thread Safety

The implementation ensures thread safety through:
1. **Mutex Guards**: Properly scoped to avoid deadlocks
2. **Data Cloning**: Clone data before async operations
3. **Lock Release**: Explicit drop() before await points

## Best Practices

### 1. Session Management
- Always use consistent session IDs for related thoughts
- Clear sessions when reasoning is complete to free memory
- Use meaningful session IDs for debugging

### 2. Branch Management
- Use descriptive branch IDs (e.g., "optimization-approach", "security-review")
- Document branch points in thoughts for clarity
- Consider merging insights from branches back to main path

### 3. Model Selection
- Use faster models (gpt-4o-mini) for exploration
- Use powerful models (o3-pro) for critical analysis
- Consider cost/performance tradeoffs

### 4. Error Handling
- Check response status before continuing
- Handle session timeouts gracefully
- Implement retry logic for transient failures

### 5. Context Preservation
- Include relevant context in first thought
- Reference specific previous thoughts when revising
- Summarize branch findings before continuing main path

## Migration Guide

### From Anthropic's Sequential Thinking

If migrating from Anthropic's TypeScript implementation:

#### Key Differences
1. **Language**: Rust instead of TypeScript
2. **Structure**: Tool-based instead of standalone server
3. **Integration**: Built into Lux MCP ecosystem
4. **Features**: Added AI generation option

#### Migration Steps

1. **Update tool name**: 
   - From: `sequentialthinking`
   - To: `sequential_thinking` or `sequential_thinking_external`

2. **Adjust field names** (camelCase to snake_case):
   ```json
   // Before
   {
     "thoughtNumber": 1,
     "totalThoughts": 5,
     "nextThoughtNeeded": true
   }
   
   // After
   {
     "thought_number": 1,
     "total_thoughts": 5,
     "next_thought_needed": true
   }
   ```

3. **Response handling**:
   - Parse response `status` field for state
   - Use `thought_history_length` instead of calculating
   - Check `confidence` field in external version

### From Traced Reasoning

If you're currently using `traced_reasoning` but want simpler control:

#### When to Switch
- You don't need metacognitive monitoring
- You want more control over the reasoning flow
- Cost is a concern (fewer LLM calls)
- You need faster response times

#### Feature Mapping
| Traced Reasoning | Sequential Thinking Alternative |
|-----------------|--------------------------------|
| `thought` | `thought` (same) |
| `thought_number` | `thought_number` (same) |
| `guardrails` | Not available (use manual review) |
| `synthesis_snapshot` | Track manually in thoughts |
| `intervention` | Handle via branching |

## Advanced Topics

### Custom Confidence Scoring
Implement domain-specific confidence scoring by analyzing thought content for your use case.

### Integration with Other Tools
Combine with `planner` for high-level structure, then use sequential thinking for detailed implementation.

### Persistence Strategies
Consider implementing database-backed sessions for long-running reasoning processes.

### Performance Optimization
- Batch related thoughts to reduce round trips
- Use caching for repeated LLM calls
- Implement pagination for large thought histories

## Troubleshooting

### Common Issues

1. **Session Not Found**
   - Ensure consistent session_id usage
   - Check for session timeout (default: none)
   - Verify server hasn't restarted

2. **Branches Not Tracking**
   - Provide both `branch_from_thought` and `branch_id`
   - Ensure branch point exists in history

3. **LLM Generation Failing**
   - Check API keys are configured
   - Verify model name is correct
   - Ensure sufficient token limits

### Debug Logging

Enable detailed logging:
```bash
RUST_LOG=debug ./target/release/lux-mcp
```

Disable thought logging for production:
```bash
DISABLE_THOUGHT_LOGGING=true ./target/release/lux-mcp
```

## See Also

- [API Reference](./sequential-thinking-api.md) - Detailed API documentation
- [Examples](./sequential-thinking-examples.md) - Complete usage examples
- [Comparison Guide](./sequential-thinking-comparison.md) - Detailed tool comparison
- [Lux MCP Overview](../README.md) - Main project documentation