# Setup Config Tool Guide

## Overview

The `setup_config` tool is designed to make configuring Lux MCP as simple as possible. Instead of manually creating and editing configuration files, this tool guides the host LLM (like Claude) through the entire process automatically.

## How It Works

When you call the `setup_config` tool, it:

1. **Analyzes Current Environment** - Checks if .env exists and what's configured
2. **Generates Configuration** - Creates a complete .env template with your preferences
3. **Provides Instructions** - Gives step-by-step instructions to the host LLM
4. **Guides File Creation** - Tells the LLM exactly how to create/update the .env file

## Usage Example

### Simple Setup (Recommended)

Just provide your API keys:

```json
{
  "tool": "setup_config",
  "arguments": {
    "openai_api_key": "sk-proj-abc123...",
    "openrouter_api_key": "sk-or-v1-xyz789..."
  }
}
```

### Advanced Setup with Custom Models

```json
{
  "tool": "setup_config",
  "arguments": {
    "openai_api_key": "sk-proj-abc123...",
    "openrouter_api_key": "sk-or-v1-xyz789...",
    "use_advanced_models": false,  // Use GPT-4o instead of GPT-5
    "custom_models": {
      "reasoning_model": "o3-pro",
      "normal_model": "gpt-4o",
      "mini_model": "gpt-4o-mini"
    }
  }
}
```

## What Gets Configured

The tool sets up:

### API Keys
- `OPENAI_API_KEY` - For OpenAI models (GPT-4, GPT-5, O3, O4)
- `OPENROUTER_API_KEY` - For OpenRouter models (Claude, Gemini, Llama)

### Model Configuration
- `LUX_MODEL_REASONING` - Model for complex reasoning tasks
- `LUX_MODEL_NORMAL` - Model for standard tasks
- `LUX_MODEL_MINI` - Model for fast/cheap tasks

### Named Model Aliases
- `LUX_MODEL_OPUS` - Maps "opus" to Claude 4.1 Opus
- `LUX_MODEL_SONNET` - Maps "sonnet" to Claude 4 Sonnet
- `LUX_MODEL_GROK` - Maps "grok" to X.AI Grok

### Optional Settings
- `RUST_LOG` - Logging level
- `LUX_REQUEST_TIMEOUT_SECS` - Request timeout
- `LUX_MAX_RETRIES` - Retry attempts

## Response Format

The tool returns a structured response containing:

```json
{
  "status": "creation_required" | "update_required",
  "instructions_for_host_llm": [
    "Step-by-step instructions for the LLM..."
  ],
  "env_template": "Complete .env file contents...",
  "env_file_path": "./.env",
  "current_config": {
    "env_file_exists": false,
    "openai_configured": false,
    "openrouter_configured": false,
    "models_configured": false,
    "current_models": {
      "reasoning": "gpt-5",
      "normal": "gpt-5",
      "mini": "gpt-5-mini",
      "opus": "anthropic/claude-4.1-opus",
      "sonnet": "anthropic/claude-4-sonnet",
      "grok": "x-ai/grok-beta"
    }
  },
  "next_steps": [
    "⚠️ Add your OpenAI API key...",
    "📝 Create the .env file...",
    "🔄 Restart the server...",
    "✅ Test the configuration..."
  ]
}
```

## Host LLM Instructions

The tool provides detailed instructions that tell the host LLM to:

1. Check if .env exists
2. Use Write tool (if creating) or Edit tool (if updating)
3. Copy the exact template provided
4. Save to ./.env
5. Verify the file was saved

## Complete Workflow Example

Here's how a typical setup conversation might look:

**User:** "Help me set up Lux MCP with my OpenAI key sk-proj-123"

**Claude:** "I'll help you configure Lux MCP. Let me use the setup tool."

```json
{
  "tool": "setup_config",
  "arguments": {
    "openai_api_key": "sk-proj-123",
    "use_advanced_models": true
  }
}
```

**Tool Response:** (Instructions and template)

**Claude:** "I'll now create your .env configuration file."

*Uses Write tool to create .env with the provided template*

**Claude:** "✅ Configuration complete! Your .env file has been created with:
- OpenAI API key configured
- GPT-5 models set as defaults
- Named model aliases configured
- All optional settings included

You can now restart the Lux MCP server to apply these settings."

## Minimal Setup

If the user just wants the absolute minimum:

```json
{
  "tool": "setup_config",
  "arguments": {
    "openai_api_key": "sk-..."
  }
}
```

This will:
- Use GPT-5 models by default
- Skip OpenRouter configuration
- Use all default settings

## Troubleshooting

### Issue: .env file not being created
**Solution:** Ensure the host LLM has file write permissions and is using the correct path (./.env)

### Issue: API keys not working
**Solution:** Verify the keys are correct and have proper permissions/credits

### Issue: Models not available
**Solution:** Check that your API keys have access to the requested models

## Best Practices

1. **Always provide at least one API key** - Either OpenAI or OpenRouter
2. **Use advanced models for best results** - GPT-5 and O3 provide superior reasoning
3. **Configure both API keys if possible** - Gives access to more model options
4. **Test after configuration** - Use a simple confer command to verify setup

## Security Notes

- Never commit .env files to version control
- Keep API keys secure and rotate them regularly
- Use environment-specific .env files for different deployments
- Consider using a secrets management service for production