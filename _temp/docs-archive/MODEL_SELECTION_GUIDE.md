# Model Selection Guide for Lux MCP

## Important: Choose the Right Model for Each Tool

### Chat Models (for `confer` tool)
Use fast, responsive models for interactive chat:
- **gpt-4o** or **gpt-4o-mini** - OpenAI's optimized models (recommended)
- **gpt-4** - Standard GPT-4
- **claude** - Claude 3 Opus via OpenRouter
- **gemini** - Google Gemini Pro via OpenRouter

**DO NOT USE**: o3, o3-pro, o3-mini for chat - these are reasoning models that can take minutes to respond!

### Reasoning Models (for `traced_reasoning` and `planner` tools)
Use deep reasoning models for complex analysis:
- **o3-pro** - Best for complex reasoning (30s - several minutes per step)
- **o3** - Good reasoning capability
- **o3-mini** - Faster reasoning model

### Bias Checking Models (for `biased_reasoning` verifier)
Use balanced models for verification:
- **o4-mini** - Fast reasoning with bias detection
- **gpt-4** - Good general-purpose verification
- **claude** - Alternative perspective via OpenRouter

## Configuration Example

```bash
# .env file
OPENAI_API_KEY=your-key
OPENROUTER_API_KEY=your-key

# CORRECT configuration
LUX_DEFAULT_CHAT_MODEL=gpt-4o          # Fast model for chat
LUX_DEFAULT_REASONING_MODEL=o3-pro     # Deep reasoning model
LUX_DEFAULT_BIAS_CHECKER_MODEL=o4-mini # Fast verifier

# WRONG configuration (will cause timeouts)
# LUX_DEFAULT_CHAT_MODEL=o3-pro  # TOO SLOW for chat!
```

## Why This Matters

1. **o3 models** are designed for deep reasoning and can take 30 seconds to several minutes per response
2. **Chat interactions** need fast responses (typically under 5 seconds)
3. Using o3-pro for chat will cause timeouts and "Failed to complete chat request" errors

## Testing Your Configuration

```bash
# Test chat with fast model
./test_confer_with_model.sh gpt-4o

# Test reasoning with o3-pro
./test_traced_reasoning.sh
```