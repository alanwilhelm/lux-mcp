# Lux MCP Model Configuration Guide

## Quick Start

Set your API keys in `.env`:
```bash
# Choose one or both:
OPENAI_API_KEY=sk-...
OPENROUTER_API_KEY=sk-or-...

# Optional: Override default models
LUX_DEFAULT_CHAT_MODEL=gpt4.1       # default: gpt4.1
LUX_DEFAULT_REASONING_MODEL=o3-pro  # default: o3-pro
LUX_DEFAULT_BIAS_CHECKER_MODEL=o4-mini  # default: o4-mini
```

## Available Models

### OpenAI Direct (requires OPENAI_API_KEY)
| Alias | Actual Model | Best For |
|-------|--------------|----------|
| `gpt4`, `4` | gpt-4 | General use |
| `gpt4.1`, `4.1` | gpt-4-turbo-preview | Faster, cheaper |
| `mini` | gpt-4o-mini | Quick responses |
| `o3` | o3 | Deep reasoning |
| `o3-pro` | o3-pro | Advanced reasoning |
| `o4-mini` | o4-mini | Fast verification |

### OpenRouter Models (requires OPENROUTER_API_KEY)
| Alias | Actual Model | Best For |
|-------|--------------|----------|
| `claude` | anthropic/claude-4-sonnet | Balanced performance |
| `opus` | anthropic/claude-4-opus | Best quality |
| `sonnet` | anthropic/claude-3-sonnet | Good balance |
| `gemini` | google/gemini-2.5-pro | Large context |
| `flash` | google/gemini-2.5-flash | Fast responses |
| `llama3` | meta-llama/llama-3-70b-instruct | Open source |

## Tool-Specific Defaults

### confer (chat tool)
- Default: `gpt4.1` (OpenAI)
- Override: Pass `model` parameter

### traced_reasoning
- Default: `o3-pro` (OpenAI) 
- Override: Pass `model` parameter
- Best models: `o3-pro`, `o3`, `claude`, `opus`

### biased_reasoning
- Primary model default: `o3-pro` (OpenAI)
- Verifier model default: `o4-mini` (OpenAI)
- Override: Pass `primary_model` and `verifier_model`

## Testing Examples

### Test with OpenAI models only:
```bash
OPENAI_API_KEY=sk-... ./target/release/lux-mcp
```

### Test with OpenRouter models:
```bash
OPENROUTER_API_KEY=sk-or-... \
LUX_DEFAULT_CHAT_MODEL=claude \
LUX_DEFAULT_REASONING_MODEL=opus \
./target/release/lux-mcp
```

### Mixed configuration:
```bash
OPENAI_API_KEY=sk-... \
OPENROUTER_API_KEY=sk-or-... \
LUX_DEFAULT_CHAT_MODEL=gpt4.1 \
LUX_DEFAULT_REASONING_MODEL=claude \
LUX_DEFAULT_BIAS_CHECKER_MODEL=gemini \
./target/release/lux-mcp
```

## Model Selection Logic

1. If model specified in request → use that model
2. If not specified → use tool-specific default from env
3. Model alias resolved via `model_aliases.rs`
4. Provider determined by model format:
   - Contains `/` → OpenRouter
   - Otherwise → OpenAI

## Troubleshooting

### "No API keys configured"
- Set at least one: `OPENAI_API_KEY` or `OPENROUTER_API_KEY`

### "Model not available"
- Check if model requires specific provider
- Verify API key for that provider is set
- Use alias from tables above

### Performance issues
- `o3` models are slower but more thoughtful
- Use `mini`, `flash`, or `o4-mini` for faster responses
- `gpt4.1` is good balance of speed and quality