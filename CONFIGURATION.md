# Lux MCP Configuration Guide

## Overview

Lux MCP uses environment variables for configuration, allowing flexible deployment across different environments. The server supports multiple LLM providers and configurable model roles.

## Configuration Files

### 1. `mcp.json` - Server Configuration

The main configuration file that defines the MCP server, its tools, and environment variables.

```json
{
  "name": "lux-mcp",
  "command": "./target/release/lux-mcp",
  "env": {
    "OPENAI_API_KEY": "${OPENAI_API_KEY}",
    "OPENROUTER_API_KEY": "${OPENROUTER_API_KEY}",
    "LUX_DEFAULT_CHAT_MODEL": "gpt4.1",
    "LUX_DEFAULT_REASONING_MODEL": "o3",
    "LUX_DEFAULT_BIAS_CHECKER_MODEL": "o4-mini"
  }
}
```

### 2. `.env` - Local Development

Create a `.env` file in the project root for local development:

```bash
# API Keys (at least one required)
OPENAI_API_KEY=sk-your-openai-key-here
OPENROUTER_API_KEY=sk-or-v1-your-openrouter-key-here

# Model Roles
LUX_DEFAULT_CHAT_MODEL=gpt4.1
LUX_DEFAULT_REASONING_MODEL=o3
LUX_DEFAULT_BIAS_CHECKER_MODEL=o4-mini

# Optional Settings
LUX_REQUEST_TIMEOUT_SECS=30
LUX_MAX_RETRIES=3
RUST_LOG=info
```

### 3. `claude_desktop_config.json` - Claude Desktop Integration

For Claude Desktop users, add this to your Claude Desktop configuration:

```json
{
  "mcpServers": {
    "lux": {
      "command": "/path/to/lux-mcp/target/release/lux-mcp",
      "env": {
        "OPENAI_API_KEY": "sk-your-key",
        "LUX_DEFAULT_REASONING_MODEL": "o3",
        "LUX_DEFAULT_BIAS_CHECKER_MODEL": "o4-mini"
      }
    }
  }
}
```

## Model Roles

Lux MCP uses three distinct model roles for different purposes:

### 1. Chat Model (`LUX_DEFAULT_CHAT_MODEL`)
- **Purpose**: General conversation and simple queries
- **Default**: `gpt4.1` (GPT-4 Turbo)
- **Used by**: `lux:chat` tool
- **Recommended**: Fast, cost-effective models

### 2. Reasoning Model (`LUX_DEFAULT_REASONING_MODEL`)
- **Purpose**: Primary reasoning and complex analysis
- **Default**: `o3` (Advanced reasoning model)
- **Used by**: `traced_reasoning` (primary), `biased_reasoning` (primary)
- **Recommended**: High-capability reasoning models

### 3. Bias Checker Model (`LUX_DEFAULT_BIAS_CHECKER_MODEL`)
- **Purpose**: Verify reasoning steps for bias and errors
- **Default**: `o4-mini` (Fast verification model)
- **Used by**: `biased_reasoning` (verifier)
- **Recommended**: Fast, analytical models

## Supported Models

### OpenAI Models (require `OPENAI_API_KEY`)
- `o3` - Advanced reasoning
- `o3-pro` - Professional reasoning
- `o4-mini` / `mini` - Fast, efficient model
- `gpt4.1` / `gpt-4.1` - GPT-4 Turbo

### OpenRouter Models (require `OPENROUTER_API_KEY`)
- `claude` / `opus` - Claude 3 Opus
- `sonnet` - Claude 3 Sonnet
- `haiku` - Claude 3 Haiku
- `gemini` - Gemini 2.5 Pro
- `flash` - Gemini 2.5 Flash
- `llama3` - Llama 3 70B
- `mixtral` - Mixtral 8x7B
- `deepseek` - DeepSeek Coder

## Environment Variables Reference

### Required (at least one)
- `OPENAI_API_KEY` - OpenAI API key
- `OPENROUTER_API_KEY` - OpenRouter API key

### Model Configuration
- `LUX_DEFAULT_CHAT_MODEL` - Default model for chat (default: "gpt4.1")
- `LUX_DEFAULT_REASONING_MODEL` - Default model for reasoning (default: "o3")
- `LUX_DEFAULT_BIAS_CHECKER_MODEL` - Default model for bias checking (default: "o4-mini")

### Optional Settings
- `LUX_REQUEST_TIMEOUT_SECS` - Request timeout in seconds (default: 30)
- `LUX_MAX_RETRIES` - Maximum retry attempts (default: 3)
- `OPENAI_BASE_URL` - Custom OpenAI API endpoint
- `OPENROUTER_BASE_URL` - Custom OpenRouter endpoint (default: "https://openrouter.ai/api/v1")
- `RUST_LOG` - Logging level (info, debug, warn, error)

## Usage Examples

### Override Models per Request

Models can be overridden on a per-request basis:

```json
{
  "tool": "biased_reasoning",
  "arguments": {
    "query": "Is nuclear energy safe?",
    "primary_model": "claude",
    "verifier_model": "gemini"
  }
}
```

### Different Configurations for Different Tasks

1. **High-stakes reasoning**: Use powerful models
   ```bash
   LUX_DEFAULT_REASONING_MODEL=o3-pro
   LUX_DEFAULT_BIAS_CHECKER_MODEL=claude
   ```

2. **Quick analysis**: Use faster models
   ```bash
   LUX_DEFAULT_REASONING_MODEL=gpt4.1
   LUX_DEFAULT_BIAS_CHECKER_MODEL=mini
   ```

3. **Cost-effective setup**: Use efficient models
   ```bash
   LUX_DEFAULT_CHAT_MODEL=mini
   LUX_DEFAULT_REASONING_MODEL=gemini
   LUX_DEFAULT_BIAS_CHECKER_MODEL=flash
   ```

## Best Practices

1. **API Key Security**
   - Never commit API keys to version control
   - Use environment variables or secure key management
   - Rotate keys regularly

2. **Model Selection**
   - Match model capabilities to task requirements
   - Consider cost vs performance tradeoffs
   - Test different model combinations

3. **Performance Tuning**
   - Adjust timeout for complex reasoning tasks
   - Use appropriate retry counts
   - Monitor API usage and costs

## Troubleshooting

### No API Keys Error
```
Error: No API keys configured
```
**Solution**: Set at least one of `OPENAI_API_KEY` or `OPENROUTER_API_KEY`

### Model Not Found
```
Error: Invalid model
```
**Solution**: Check model name and ensure the provider API key is set

### Timeout Errors
```
Error: Request timeout
```
**Solution**: Increase `LUX_REQUEST_TIMEOUT_SECS` for complex tasks

## Integration Patterns

### Development
```bash
# Load from .env file
source .env
cargo run
```

### Production
```bash
# Set environment variables directly
export OPENAI_API_KEY=sk-...
export LUX_DEFAULT_REASONING_MODEL=o3-pro
./target/release/lux-mcp
```

### Docker
```dockerfile
ENV OPENAI_API_KEY=${OPENAI_API_KEY}
ENV LUX_DEFAULT_REASONING_MODEL=o3
ENV LUX_DEFAULT_BIAS_CHECKER_MODEL=o4-mini
```

### Kubernetes
```yaml
env:
  - name: OPENAI_API_KEY
    valueFrom:
      secretKeyRef:
        name: lux-secrets
        key: openai-api-key
  - name: LUX_DEFAULT_REASONING_MODEL
    value: "o3"
```