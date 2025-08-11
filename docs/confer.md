# Confer Tool Documentation

## Overview

The `confer` tool is a straightforward conversational AI interface that provides simple, direct interactions with various language models. It maintains conversation context through threading and supports file reading for code analysis, making it ideal for quick questions, explanations, and casual AI interactions without the overhead of monitoring or multi-step reasoning.

## Key Features

- **Simple Interface**: Direct question-answer format
- **Model Selection**: Choose from any supported model
- **Conversation Threading**: Maintain context across multiple turns
- **File Reading**: Analyze code and documents directly
- **Token Control**: Configure response length
- **Temperature Tuning**: Adjust creativity level
- **Cost Optimization**: Optional mini model for savings
- **Named Models**: Use shortcuts like "opus", "sonnet", "grok"

## When to Use

### Ideal For:
- **Quick Questions**: Simple Q&A without complexity
- **Code Explanation**: Understanding code snippets
- **Brainstorming**: Creative ideation sessions
- **Documentation**: Generating docs or comments
- **Learning**: Educational queries and tutorials
- **Casual Chat**: General conversation with AI

### Not Recommended For:
- Complex multi-step reasoning (use `traced_reasoning`)
- Tasks requiring bias detection (use `biased_reasoning`)
- Implementation planning (use `planner`)
- Structured thinking processes (use `sequential_thinking`)

## API Reference

### Request Parameters

```typescript
interface ConferRequest {
  // Required
  message: string;                    // The message to send to the AI
  
  // Optional
  continuation_id?: string;           // Thread ID for conversation continuity
  model?: string;                     // Model to use (default: LUX_MODEL_NORMAL)
  temperature?: number;               // Temperature 0.0-1.0 (default: 0.7)
  max_tokens?: number;                // Max tokens for response (default: 10000)
  use_mini?: boolean;                 // Use mini model for cost savings
  
  // Optional file reading
  file_paths?: string[];              // Files to read and include in context
  include_file_contents?: boolean;    // Whether to read files (default: true)
}
```

### Response Format

```typescript
interface ConferResponse {
  response: string;                   // The AI's response
  model_used: string;                 // Actual model that was used
  thread_id: string;                  // Thread ID for continuation
  token_usage?: {
    prompt_tokens: number;
    completion_tokens: number;
    total_tokens: number;
  };
}
```

## Usage Examples

### Basic Question

```json
{
  "tool": "confer",
  "arguments": {
    "message": "Explain the difference between REST and GraphQL"
  }
}

// Response
{
  "response": "REST and GraphQL are different approaches to API design...",
  "model_used": "gpt-5",
  "thread_id": "thread-abc123"
}
```

### With Model Selection

```json
// Using named model
{
  "tool": "confer",
  "arguments": {
    "message": "Write a haiku about programming",
    "model": "opus"  // Uses Claude 4.1 Opus
  }
}

// Using specific model
{
  "tool": "confer",
  "arguments": {
    "message": "Optimize this SQL query for performance",
    "model": "gpt-4o",
    "temperature": 0.3  // Lower temperature for technical accuracy
  }
}
```

### Conversation Threading

```json
// First message
{
  "tool": "confer",
  "arguments": {
    "message": "What are the main principles of functional programming?"
  }
}
// Returns thread_id: "thread-xyz789"

// Continue conversation
{
  "tool": "confer",
  "arguments": {
    "message": "How does immutability help with concurrency?",
    "continuation_id": "thread-xyz789"
  }
}
```

### With File Context

```json
{
  "tool": "confer",
  "arguments": {
    "message": "Explain what this authentication middleware does and suggest improvements",
    "file_paths": [
      "/api/middleware/auth.js",
      "/api/utils/jwt.js"
    ]
  }
}
```

### Cost-Optimized Usage

```json
{
  "tool": "confer",
  "arguments": {
    "message": "What's the capital of France?",
    "use_mini": true,  // Uses LUX_MODEL_MINI
    "max_tokens": 100  // Limit response length
  }
}
```

## Model Selection Guide

### Available Models

#### OpenAI Models
- `gpt-5` - Most capable, 128K-200K tokens
- `gpt-5-mini` - Faster, cheaper GPT-5
- `gpt-4o` - Balanced performance
- `gpt-4o-mini` - Fast and economical
- `o3`, `o3-pro` - Deep reasoning (slower)
- `o4-mini` - Fast reasoning

#### Named Models (via OpenRouter)
- `opus` - Claude 4.1 Opus (sophisticated)
- `sonnet` - Claude 4 Sonnet (balanced)
- `grok` - Latest Grok model
- `claude` - Default Claude model
- `gemini` - Google Gemini
- `llama3` - Meta Llama 3

### Model Selection Strategy

| Use Case | Recommended Model | Temperature | Rationale |
|----------|------------------|-------------|-----------|
| Code Review | `gpt-5` | 0.3 | Accuracy and depth |
| Creative Writing | `opus` | 0.8 | Creativity and style |
| Quick Facts | `use_mini: true` | 0.5 | Speed and cost |
| Technical Docs | `gpt-4o` | 0.5 | Balance of quality/speed |
| Brainstorming | `claude` | 0.9 | Creative exploration |
| Data Analysis | `gpt-5` | 0.2 | Precision required |

## Conversation Threading

### How Threading Works

1. **Automatic Creation**: First message creates a thread
2. **Context Preservation**: Previous messages are included
3. **Token Management**: Automatically truncates old messages
4. **3-Hour TTL**: Threads expire after inactivity
5. **Seamless Continuation**: Just pass the thread_id

### Threading Example

```python
class ConversationManager:
    def __init__(self):
        self.thread_id = None
    
    def ask(self, message, **kwargs):
        params = {"message": message}
        
        if self.thread_id:
            params["continuation_id"] = self.thread_id
        
        params.update(kwargs)
        
        response = call_tool("confer", params)
        self.thread_id = response["thread_id"]
        
        return response["response"]

# Usage
conv = ConversationManager()
conv.ask("What is Python?")
conv.ask("What are its main use cases?")  # Continues context
conv.ask("Show me a code example")  # Still has context
```

## File Reading Capabilities

### Supported File Types

- **Code**: `.js`, `.py`, `.java`, `.go`, `.rs`, etc.
- **Config**: `.json`, `.yaml`, `.toml`, `.env`
- **Docs**: `.md`, `.txt`, `.rst`
- **Web**: `.html`, `.css`, `.xml`

### File Reading Examples

#### Code Review

```json
{
  "tool": "confer",
  "arguments": {
    "message": "Review this code for security vulnerabilities and performance issues",
    "file_paths": [
      "/api/auth/login.js",
      "/api/auth/session.js"
    ],
    "model": "gpt-5",
    "temperature": 0.3
  }
}
```

#### Documentation Generation

```json
{
  "tool": "confer",
  "arguments": {
    "message": "Generate comprehensive JSDoc comments for all functions",
    "file_paths": ["/src/utils/helpers.js"],
    "model": "gpt-4o"
  }
}
```

#### Bug Analysis

```json
{
  "tool": "confer",
  "arguments": {
    "message": "This test is failing. What's wrong with the implementation?",
    "file_paths": [
      "/src/calculator.js",
      "/tests/calculator.test.js"
    ]
  }
}
```

## Best Practices

### 1. Message Clarity

```json
// Good: Specific and contextual
{
  "message": "Explain the time complexity of this sorting algorithm and suggest optimizations"
}

// Poor: Vague
{
  "message": "Tell me about this"
}
```

### 2. Model Selection

```python
def smart_model_selection(query_type, priority="balanced"):
    models = {
        "speed": {"model": "gpt-4o-mini", "temperature": 0.5},
        "quality": {"model": "gpt-5", "temperature": 0.3},
        "balanced": {"model": "gpt-4o", "temperature": 0.5},
        "creative": {"model": "opus", "temperature": 0.8},
        "budget": {"use_mini": True, "temperature": 0.5}
    }
    
    return models.get(priority, models["balanced"])
```

### 3. Temperature Guidelines

| Temperature | Use Case | Example |
|------------|----------|---------|
| 0.0-0.3 | Factual, deterministic | Code generation, math |
| 0.4-0.6 | Balanced | General Q&A, explanations |
| 0.7-0.8 | Creative, varied | Brainstorming, writing |
| 0.9-1.0 | Maximum creativity | Poetry, fiction |

### 4. Token Management

```python
def managed_conversation(message, context_size="medium"):
    token_limits = {
        "small": 1000,
        "medium": 5000,
        "large": 10000,
        "max": 50000
    }
    
    return call_tool("confer", {
        "message": message,
        "max_tokens": token_limits[context_size]
    })
```

### 5. Error Handling

```python
def safe_confer(message, retries=3):
    for attempt in range(retries):
        try:
            response = call_tool("confer", {
                "message": message,
                "model": "gpt-4o"
            })
            return response
        except Exception as e:
            if "rate_limit" in str(e):
                time.sleep(2 ** attempt)  # Exponential backoff
            elif "model_not_available" in str(e):
                # Fallback to different model
                response = call_tool("confer", {
                    "message": message,
                    "use_mini": True
                })
                return response
            else:
                raise
```

## Performance Characteristics

### Response Times

| Model | Simple Query | With Files | Complex Query |
|-------|-------------|------------|---------------|
| Mini | 0.5-2s | 1-3s | 2-5s |
| GPT-4o | 1-3s | 2-5s | 3-8s |
| GPT-5 | 2-5s | 3-8s | 5-15s |
| Opus | 2-4s | 3-6s | 4-10s |
| O3-Pro | 10-30s | 15-40s | 30-60s |

### Token Usage

- **Simple Query**: 500-1500 tokens
- **With Context**: 2000-5000 tokens
- **With Files**: 5000-15000 tokens
- **Long Conversation**: 10000-30000 tokens

### Cost Optimization

```python
class CostAwareConfer:
    def __init__(self, budget_mode=False):
        self.budget_mode = budget_mode
        self.token_count = 0
    
    def ask(self, message, force_quality=False):
        if force_quality:
            params = {"model": "gpt-5"}
        elif self.budget_mode or self.token_count > 50000:
            params = {"use_mini": True, "max_tokens": 2000}
        else:
            params = {"model": "gpt-4o"}
        
        response = call_tool("confer", {
            "message": message,
            **params
        })
        
        self.token_count += response.get("token_usage", {}).get("total_tokens", 0)
        return response
```

## Common Use Cases

### Code Explanation

```json
{
  "message": "Explain this React hook implementation line by line",
  "file_paths": ["/src/hooks/useAuth.js"],
  "model": "gpt-4o",
  "temperature": 0.3
}
```

### API Design

```json
{
  "message": "Design a RESTful API for a task management system with user authentication",
  "model": "gpt-5",
  "temperature": 0.5,
  "max_tokens": 15000
}
```

### Debugging Help

```json
{
  "message": "This function returns undefined sometimes. Help me debug it",
  "file_paths": [
    "/src/utils/dataProcessor.js",
    "/tests/dataProcessor.test.js"
  ],
  "model": "gpt-5",
  "temperature": 0.2
}
```

### Learning Assistant

```json
{
  "message": "Teach me about recursion with practical JavaScript examples",
  "model": "claude",
  "temperature": 0.6
}
```

### Code Generation

```json
{
  "message": "Generate a Python class for managing a connection pool with retry logic",
  "model": "gpt-4o",
  "temperature": 0.3,
  "max_tokens": 5000
}
```

## Integration Patterns

### With Other Tools

```python
def comprehensive_analysis(code_path):
    # Step 1: Quick understanding with confer
    understanding = call_tool("confer", {
        "message": "What does this code do?",
        "file_paths": [code_path]
    })
    
    # Step 2: Deep analysis if needed
    if "complex" in understanding["response"].lower():
        analysis = call_tool("traced_reasoning", {
            "thought": f"Analyze the architecture of {code_path}",
            "thought_number": 1,
            "total_thoughts": 3,
            "next_thought_needed": True
        })
    
    # Step 3: Check for biases if decision logic
    if "decision" in understanding["response"].lower():
        bias_check = call_tool("biased_reasoning", {
            "query": f"Check for biases in: {understanding['response']}"
        })
    
    return understanding
```

### Streaming Responses (Future)

```python
# Planned feature
async def stream_confer(message):
    async for chunk in call_tool_stream("confer", {
        "message": message,
        "stream": True
    }):
        print(chunk, end="")
```

## Comparison with Other Tools

| Feature | confer | traced_reasoning | planner | biased_reasoning |
|---------|--------|------------------|---------|-------------------|
| Simplicity | ✅ Best | ❌ Complex | ❌ Complex | ❌ Complex |
| Speed | ✅ Fast | Medium | Medium | Slow |
| Threading | ✅ | ✅ | ✅ | ✅ |
| File Reading | ✅ | ✅ | ✅ Auto | ✅ |
| Monitoring | ❌ | ✅ Full | ❌ | Bias only |
| Multi-step | ❌ | ✅ | ✅ | ✅ |
| Cost | Low | High | Medium | Very High |

## Troubleshooting

### Common Issues

1. **"Model not found"**
   ```json
   // Solution: Use a valid model or default
   {
     "model": "gpt-4o"  // Valid model
     // Or just omit model to use default
   }
   ```

2. **"Thread expired"**
   - Threads expire after 3 hours
   - Start a new conversation
   - Or omit continuation_id

3. **Response truncated**
   ```json
   // Solution: Increase max_tokens
   {
     "max_tokens": 20000  // Increase limit
   }
   ```

4. **High latency**
   - Use faster models (gpt-4o-mini, use_mini)
   - Reduce file_paths count
   - Lower max_tokens

### Debug Mode

```bash
# Enable logging
RUST_LOG=debug ./target/release/lux-mcp

# Trace specific tool
RUST_LOG=lux_mcp::tools::chat=trace ./target/release/lux-mcp
```

## Tips and Tricks

### 1. Model Fallbacks

```python
models_priority = ["gpt-5", "gpt-4o", "claude", "use_mini"]

for model in models_priority:
    try:
        if model == "use_mini":
            response = call_tool("confer", {
                "message": message,
                "use_mini": True
            })
        else:
            response = call_tool("confer", {
                "message": message,
                "model": model
            })
        break
    except:
        continue
```

### 2. Context Injection

```python
def contextualized_query(question, context_files):
    context = f"""
    Based on the following code context, {question}
    
    Please consider:
    - Code style and conventions
    - Existing patterns
    - Dependencies and constraints
    """
    
    return call_tool("confer", {
        "message": context,
        "file_paths": context_files
    })
```

### 3. Response Formatting

```json
{
  "message": "Explain the code structure. Format your response as: 1) Overview 2) Main Components 3) Data Flow 4) Key Functions",
  "file_paths": ["/src/app.js"]
}
```

## Summary

The `confer` tool is your go-to for simple, fast AI interactions. It excels at straightforward Q&A, code explanation, and creative tasks without the complexity of multi-step reasoning or monitoring systems.

Key takeaways:
- Simplest tool for basic AI conversations
- Supports all models with easy selection
- Maintains conversation context via threading
- Can read files for code/document analysis
- Low latency and token usage
- Perfect for learning, brainstorming, and quick questions

For complex reasoning, use `traced_reasoning`. For planning, use `planner`. For bias detection, use `biased_reasoning`.