# Lux MCP Setup Guide

## 1. Environment Setup

First, create your `.env` file from the example:

```bash
cp .env.example .env
```

Edit `.env` and add your API keys:

```bash
# Required: At least one API key
OPENAI_API_KEY=sk-your-openai-key-here
OPENROUTER_API_KEY=sk-or-v1-your-openrouter-key-here

# Model Configuration
LUX_DEFAULT_CHAT_MODEL=gpt4.1
LUX_DEFAULT_REASONING_MODEL=o3
LUX_DEFAULT_BIAS_CHECKER_MODEL=o4-mini
```

## 2. Testing the Server

### Direct Testing
```bash
# Test the server directly
./target/release/lux-mcp
```

This should output the MCP capabilities in JSON format.

### Test Scripts
```bash
# Test chat tool
./test_chat.sh

# Test traced reasoning
./test_traced_reasoning.sh

# Test biased reasoning
./test_biased_reasoning.sh
```

## 3. Claude Desktop Configuration

Add to your Claude Desktop config file:

**macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
**Windows**: `%APPDATA%\Claude\claude_desktop_config.json`
**Linux**: `~/.config/Claude/claude_desktop_config.json`

```json
{
  "mcpServers": {
    "lux": {
      "command": "/Users/alan/Projects/_MCP/nirvana/lux-mcp/target/release/lux-mcp",
      "env": {
        "OPENAI_API_KEY": "sk-your-openai-key",
        "OPENROUTER_API_KEY": "sk-or-v1-your-openrouter-key",
        "LUX_DEFAULT_CHAT_MODEL": "gpt4.1",
        "LUX_DEFAULT_REASONING_MODEL": "o3",
        "LUX_DEFAULT_BIAS_CHECKER_MODEL": "o4-mini",
        "RUST_LOG": "info"
      }
    }
  }
}
```

## 4. VS Code / Cline Configuration

For VS Code with Cline extension, add to your workspace settings:

`.vscode/settings.json`:
```json
{
  "mcpServers": {
    "lux": {
      "command": "cargo",
      "args": ["run", "--release"],
      "cwd": "/Users/alan/Projects/_MCP/nirvana/lux-mcp",
      "env": {
        "OPENAI_API_KEY": "${env:OPENAI_API_KEY}",
        "OPENROUTER_API_KEY": "${env:OPENROUTER_API_KEY}",
        "LUX_DEFAULT_CHAT_MODEL": "gpt4.1",
        "LUX_DEFAULT_REASONING_MODEL": "o3",
        "LUX_DEFAULT_BIAS_CHECKER_MODEL": "o4-mini"
      }
    }
  }
}
```

## 5. Available Tools

### lux:chat
Simple conversational AI with model selection:
```typescript
{
  "tool": "lux:chat",
  "arguments": {
    "message": "Explain quantum computing",
    "model": "claude", // optional, defaults to LUX_DEFAULT_CHAT_MODEL
    "temperature": 0.7, // optional
    "max_tokens": 1000 // optional
  }
}
```

### traced_reasoning
Step-by-step reasoning with transparency:
```typescript
{
  "tool": "traced_reasoning",
  "arguments": {
    "query": "Design a distributed cache system",
    "model": "o3", // optional
    "max_thinking_steps": 10 // optional
  }
}
```

### biased_reasoning
Dual-model verification for bias detection:
```typescript
{
  "tool": "biased_reasoning",  
  "arguments": {
    "query": "Is nuclear energy safe?",
    "primary_model": "o3", // optional
    "verifier_model": "claude", // optional
    "max_analysis_rounds": 3 // optional
  }
}
```

## 6. Model Aliases

### OpenAI Models
- `o3`, `o3-pro` - Advanced reasoning
- `o4-mini`, `mini` - Fast, efficient
- `gpt4.1`, `gpt-4.1` - GPT-4 Turbo

### Claude Models (via OpenRouter)
- `claude` - Maps to Claude 4 Sonnet
- `opus-4`, `sonnet-4` - Claude 4 variants
- `opus`, `sonnet`, `haiku` - Claude 3 variants
- `claude-3.5` - Claude 3.5 Sonnet

### Google Models (via OpenRouter)
- `gemini` - Gemini 2.5 Pro
- `flash` - Gemini 2.5 Flash

### Other Models (via OpenRouter)
- `llama3` - Llama 3 70B
- `mixtral` - Mixtral 8x7B
- `deepseek` - DeepSeek Coder

## 7. Verifying Setup

1. **Check server capabilities:**
   ```bash
   echo '{"jsonrpc":"2.0","method":"initialize","params":{"capabilities":{}},"id":1}' | ./target/release/lux-mcp
   ```

2. **Test a tool:**
   ```bash
   echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"lux:chat","arguments":{"message":"Hello"}},"id":2}' | ./target/release/lux-mcp
   ```

3. **Check logs:**
   ```bash
   RUST_LOG=debug ./target/release/lux-mcp
   ```

## 8. Troubleshooting

### No API Keys Error
- Ensure at least one of `OPENAI_API_KEY` or `OPENROUTER_API_KEY` is set
- Check that keys are valid and have credits

### Model Not Found
- Verify the model alias exists in `model_aliases.rs`
- Ensure the appropriate API key is set for the provider

### Connection Issues
- Check that the binary path is absolute in config files
- Verify file permissions: `chmod +x target/release/lux-mcp`
- Test the binary directly before MCP integration

### Claude Desktop Not Finding Tools
- Restart Claude Desktop after config changes
- Check logs: View → Developer → Developer Tools → Console
- Verify JSON syntax in config file

## 9. Development Tips

### Running in Development Mode
```bash
# With debug logging
RUST_LOG=debug cargo run

# With specific model overrides
LUX_DEFAULT_REASONING_MODEL=claude cargo run
```

### Testing Model Combinations
```bash
# High-quality reasoning
LUX_DEFAULT_REASONING_MODEL=o3-pro LUX_DEFAULT_BIAS_CHECKER_MODEL=claude cargo run

# Fast iteration
LUX_DEFAULT_REASONING_MODEL=mini LUX_DEFAULT_BIAS_CHECKER_MODEL=flash cargo run
```

## 10. Best Practices

1. **Model Selection**
   - Use `o3`/`o3-pro` for complex reasoning tasks
   - Use `mini`/`flash` for quick responses
   - Use `claude`/`gemini` for balanced performance

2. **API Key Management**
   - Never commit `.env` files
   - Use environment variables in production
   - Rotate keys regularly

3. **Performance Tuning**
   - Adjust `LUX_REQUEST_TIMEOUT_SECS` for long tasks
   - Monitor API usage and costs
   - Use appropriate models for each role