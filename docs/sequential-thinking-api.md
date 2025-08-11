# Sequential Thinking API Reference

## Table of Contents
1. [Tool Registration](#tool-registration)
2. [Sequential Thinking API](#sequential-thinking-api)
3. [Sequential Thinking External API](#sequential-thinking-external-api)
4. [Session Management APIs](#session-management-apis)
5. [Error Codes](#error-codes)
6. [Type Definitions](#type-definitions)

## Tool Registration

Both sequential thinking tools are registered with the MCP server and available via the standard MCP protocol.

### List Available Tools
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/list"
}
```

Response includes:
```json
{
  "tools": [
    {
      "name": "sequential_thinking",
      "description": "Simple sequential thinking tool for organizing thoughts step-by-step..."
    },
    {
      "name": "sequential_thinking_external",
      "description": "AI-powered sequential thinking with external LLM integration..."
    }
  ]
}
```

## Sequential Thinking API

### Tool Name
`sequential_thinking`

### Call Tool
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "sequential_thinking",
    "arguments": {
      // See parameters below
    }
  }
}
```

### Required Parameters

| Parameter | Type | Description | Constraints |
|-----------|------|-------------|-------------|
| `thought` | string | The content of the current thinking step | Non-empty string |
| `thought_number` | u32 | Current thought number in sequence | Minimum: 1 |
| `total_thoughts` | u32 | Estimated total number of thoughts needed | Minimum: 1 |
| `next_thought_needed` | boolean | Whether another thought step is required | true/false |

### Optional Parameters

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| `is_revision` | boolean | Indicates this thought revises a previous one | false |
| `revises_thought` | u32 | Which thought number is being revised | null |
| `branch_from_thought` | u32 | Starting point for a new branch | null |
| `branch_id` | string | Unique identifier for the branch | null |
| `needs_more_thoughts` | boolean | Indicates need for more thoughts beyond estimate | false |
| `session_id` | string | Session identifier for continuity | "default" |

### Response Format

```typescript
interface SequentialThinkingResponse {
  thought_number: number;
  total_thoughts: number;
  next_thought_needed: boolean;
  branches: string[];           // List of active branch IDs
  thought_history_length: number;
  session_id?: string;          // Returned if non-default
  status: "recorded" | "revision" | "branch" | "complete";
}
```

### Example Calls

#### Basic Thought
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "sequential_thinking",
    "arguments": {
      "thought": "Analyze the current system architecture",
      "thought_number": 1,
      "total_thoughts": 5,
      "next_thought_needed": true
    }
  }
}
```

#### Revision
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "sequential_thinking",
    "arguments": {
      "thought": "Actually, we should consider microservices instead",
      "thought_number": 3,
      "total_thoughts": 5,
      "next_thought_needed": true,
      "is_revision": true,
      "revises_thought": 2,
      "session_id": "arch-review-001"
    }
  }
}
```

#### Branching
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "sequential_thinking",
    "arguments": {
      "thought": "Explore serverless as an alternative",
      "thought_number": 4,
      "total_thoughts": 6,
      "next_thought_needed": true,
      "branch_from_thought": 3,
      "branch_id": "serverless-exploration",
      "session_id": "arch-review-001"
    }
  }
}
```

## Sequential Thinking External API

### Tool Name
`sequential_thinking_external`

### Call Tool
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "sequential_thinking_external",
    "arguments": {
      // See parameters below
    }
  }
}
```

### Required Parameters

| Parameter | Type | Description | Constraints |
|-----------|------|-------------|-------------|
| `thought` | string | Step 1: problem/query, Step 2+: guidance/continuation | Non-empty string |
| `thought_number` | u32 | Current thought number in sequence | Minimum: 1 |
| `total_thoughts` | u32 | Estimated total number of thoughts needed | Minimum: 1 |
| `next_thought_needed` | boolean | Whether another thought step is required | true/false |

### Optional Parameters

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| `is_revision` | boolean | Indicates this thought revises a previous one | false |
| `revises_thought` | u32 | Which thought number is being revised | null |
| `branch_from_thought` | u32 | Starting point for a new branch | null |
| `branch_id` | string | Unique identifier for the branch | null |
| `needs_more_thoughts` | boolean | Indicates need for more thoughts beyond estimate | false |
| `session_id` | string | Session identifier for continuity | "default" |
| `model` | string | LLM model to use | LUX_MODEL_NORMAL |
| `temperature` | f32 | Temperature for LLM generation | 0.7 |
| `use_llm` | boolean | Whether to use LLM for generation | true |

### Response Format

```typescript
interface SequentialThinkingExternalResponse {
  thought_number: number;
  total_thoughts: number;
  next_thought_needed: boolean;
  thought_content: string;       // Generated or provided thought
  branches: string[];            // List of active branch IDs
  thought_history_length: number;
  status: "thinking" | "revision" | "branch" | "complete";
  
  // Optional fields
  session_id?: string;           // Returned if non-default
  model_used?: string;           // Model that generated the thought
  confidence?: number;           // 0.0-1.0 confidence score
  reasoning_hint?: string;       // Suggestion for next step
}
```

### Example Calls

#### Initial Problem with AI
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "sequential_thinking_external",
    "arguments": {
      "thought": "How do I implement rate limiting in a distributed system?",
      "thought_number": 1,
      "total_thoughts": 5,
      "next_thought_needed": true,
      "model": "gpt-4o",
      "temperature": 0.7
    }
  }
}
```

#### Continue with Guidance
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "sequential_thinking_external",
    "arguments": {
      "thought": "Focus on Redis-based solutions",
      "thought_number": 2,
      "total_thoughts": 5,
      "next_thought_needed": true,
      "session_id": "rate-limit-design"
    }
  }
}
```

#### Manual Mode (No LLM)
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "sequential_thinking_external",
    "arguments": {
      "thought": "This is my manual thought content",
      "thought_number": 3,
      "total_thoughts": 5,
      "next_thought_needed": true,
      "use_llm": false,
      "session_id": "rate-limit-design"
    }
  }
}
```

## Session Management APIs

### Session Lifecycle

Sessions are created automatically when a thought is processed. They persist in memory until:
1. The server is restarted
2. The session is explicitly cleared
3. Memory limits are reached (implementation-dependent)

### Session ID Guidelines

- **Format**: Any valid string, recommended: UUID or descriptive-id
- **Default**: "default" is used when no session_id is provided
- **Scope**: Sessions are isolated - no cross-session data access
- **Persistence**: In-memory only (not persisted to disk)

### Clearing Sessions

While not exposed via MCP, sessions can be managed programmatically:

```rust
// Internal API (not exposed via MCP)
tool.clear_session(Some("session-id".to_string()))?;
```

## Error Codes

### MCP Error Responses

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32602,
    "message": "Invalid params",
    "data": "Missing required field: thought_number"
  }
}
```

### Common Error Codes

| Code | Name | Description |
|------|------|-------------|
| -32700 | Parse error | Invalid JSON |
| -32600 | Invalid Request | Not a valid MCP request |
| -32601 | Method not found | Unknown method |
| -32602 | Invalid params | Invalid method parameters |
| -32603 | Internal error | Server error during processing |

### Tool-Specific Errors

#### Sequential Thinking Errors
- `"Missing arguments for sequential_thinking"` - Required fields missing
- `"Invalid sequential thinking params: {details}"` - Parameter validation failed

#### Sequential Thinking External Errors
- `"OpenRouter API key not configured"` - Missing API key for OpenRouter models
- `"OpenAI API key not configured"` - Missing API key for OpenAI models
- `"Failed to generate thought with LLM"` - LLM generation failed

## Type Definitions

### Rust Types

```rust
// Core thought data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThoughtData {
    pub thought: String,
    pub thought_number: u32,
    pub total_thoughts: u32,
    pub next_thought_needed: bool,
    pub is_revision: bool,
    pub revises_thought: Option<u32>,
    pub branch_from_thought: Option<u32>,
    pub branch_id: Option<String>,
    pub needs_more_thoughts: bool,
}

// External thought with AI metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalThoughtData {
    // All fields from ThoughtData plus:
    pub model_used: Option<String>,
    pub confidence: Option<f32>,
}

// Session state
#[derive(Debug, Default)]
struct SessionState {
    thought_history: Vec<ThoughtData>,
    branches: HashMap<String, Vec<ThoughtData>>,
}

// External session state
#[derive(Debug, Default)]
struct ExternalSessionState {
    thought_history: Vec<ExternalThoughtData>,
    branches: HashMap<String, Vec<ExternalThoughtData>>,
    original_query: Option<String>,
}
```

### JSON Schema

#### Sequential Thinking Request Schema
```json
{
  "type": "object",
  "properties": {
    "thought": {
      "type": "string",
      "description": "The current thinking step content"
    },
    "thought_number": {
      "type": "integer",
      "minimum": 1
    },
    "total_thoughts": {
      "type": "integer",
      "minimum": 1
    },
    "next_thought_needed": {
      "type": "boolean"
    },
    "is_revision": {
      "type": "boolean"
    },
    "revises_thought": {
      "type": "integer"
    },
    "branch_from_thought": {
      "type": "integer"
    },
    "branch_id": {
      "type": "string"
    },
    "needs_more_thoughts": {
      "type": "boolean"
    },
    "session_id": {
      "type": "string"
    }
  },
  "required": ["thought", "thought_number", "total_thoughts", "next_thought_needed"],
  "additionalProperties": false
}
```

## Rate Limiting

The tools themselves don't implement rate limiting, but when using the external version:

### LLM Provider Limits
- **OpenAI**: Varies by tier and model
- **OpenRouter**: Depends on selected model
- **Rate limit errors**: Returned as internal errors with details

### Recommendations
1. Implement client-side rate limiting
2. Use exponential backoff for retries
3. Consider caching for repeated queries
4. Monitor usage via provider dashboards

## Performance Characteristics

### Sequential Thinking (Simple)
- **Latency**: < 1ms (in-memory operations only)
- **Throughput**: Limited by mutex contention
- **Memory**: O(n) where n = number of thoughts
- **Scalability**: Single-node only

### Sequential Thinking External
- **Latency**: 500ms - 5s (depends on model and prompt size)
- **Throughput**: Limited by LLM API rate limits
- **Memory**: O(n) for history + LLM context window
- **Scalability**: Depends on LLM provider

### Optimization Tips
1. Use smaller models for exploration phases
2. Batch related thoughts when possible
3. Implement caching for common patterns
4. Monitor session memory usage
5. Clear completed sessions promptly

## Versioning

### Current Version
- Tool Version: 0.1.0
- API Version: MCP 1.0
- Rust Implementation: 2024

### Compatibility
- MCP Protocol: 1.0
- Minimum Rust: 1.70
- LLM Providers: OpenAI v1, OpenRouter v1

### Future Enhancements
- Database persistence for sessions
- Batch processing API
- Streaming responses
- WebSocket support
- Cross-session analysis tools