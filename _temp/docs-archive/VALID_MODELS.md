# Valid Model Names for Lux MCP

## ❌ INVALID Models (These don't exist!)
- `o3` - Not a real model
- `o3-pro` - Not a real model  
- `o4-mini` - Not a real model

## ✅ VALID OpenAI Models (use with OPENAI_API_KEY)
- `gpt-4-turbo-preview` - Latest GPT-4 Turbo (aliases: `gpt4.1`, `gpt-4.1`)
- `gpt-4o-mini` - Fast, efficient model (alias: `mini`)
- `gpt-4` - Standard GPT-4
- `gpt-3.5-turbo` - Fast, cheaper model

## ✅ VALID OpenRouter Models (use with OPENROUTER_API_KEY)
- `claude` - Claude 3 Opus (best Claude model)
- `sonnet` - Claude 3.5 Sonnet
- `haiku` - Claude 3 Haiku (fast)
- `gemini` - Google Gemini Pro
- `flash` - Google Gemini Flash (fast)
- `llama3` - Meta Llama 3 70B
- `mixtral` - Mixtral 8x7B
- `deepseek` - DeepSeek Coder

## Recommended Configuration

For Claude Code, update your config to:

```json
"lux": {
  "command": "/Users/alan/Projects/_MCP/nirvana/lux-mcp/target/release/lux-mcp",
  "env": {
    "OPENROUTER_API_KEY": "your-key",
    "OPENAI_API_KEY": "your-key",
    "LUX_DEFAULT_CHAT_MODEL": "gpt-4-turbo-preview",
    "LUX_DEFAULT_REASONING_MODEL": "gpt-4-turbo-preview", 
    "LUX_DEFAULT_BIAS_CHECKER_MODEL": "gpt-4o-mini",
    "RUST_LOG": "info"
  }
}
```

Or for OpenRouter models:

```json
"LUX_DEFAULT_CHAT_MODEL": "claude",
"LUX_DEFAULT_REASONING_MODEL": "claude",
"LUX_DEFAULT_BIAS_CHECKER_MODEL": "gemini"
```

## Quick Test

```bash
# Test with valid OpenAI models
export LUX_DEFAULT_CHAT_MODEL="gpt-4-turbo-preview"
export LUX_DEFAULT_REASONING_MODEL="gpt-4-turbo-preview"
export LUX_DEFAULT_BIAS_CHECKER_MODEL="gpt-4o-mini"

./test_rmcp_correct.py
```