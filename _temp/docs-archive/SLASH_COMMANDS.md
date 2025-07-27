# Lux MCP Slash Commands

## Overview

In Claude Desktop or other MCP clients, slash commands follow this pattern:
```
/[server-name]:[tool-name]
```

For Lux MCP, the server name is `lux`, so all commands start with `/lux:`.

## Available Slash Commands

### /lux:confer
Simple conversational AI with model selection.

**Example usage:**
```
/lux:confer What is quantum computing?
```

**With options:**
```
/lux:confer {"message": "Explain React hooks", "model": "claude"}
```

### /lux:traced_reasoning
Step-by-step reasoning with metacognitive monitoring.

**Example usage:**
```
/lux:traced_reasoning How do I design a distributed cache system?
```

**With options:**
```
/lux:traced_reasoning {"query": "Design a REST API", "model": "o3", "max_steps": 15}
```

### /lux:biased_reasoning
Dual-model reasoning with bias detection.

**Example usage:**
```
/lux:biased_reasoning Should we migrate to microservices?
```

**With options:**
```
/lux:biased_reasoning {"query": "Evaluate this technology choice", "primary_model": "o3", "verifier_model": "claude"}
```

### /lux:plan
Create structured plans with metacognitive monitoring.

**Example usage:**
```
/lux:plan Build a mobile app with offline sync
```

**With options:**
```
/lux:plan {"goal": "Launch a SaaS product", "model": "o3-pro", "max_steps": 8}
```

### /lux:illumination_status
Check the current metacognitive state.

**Example usage:**
```
/lux:illumination_status
```

## Prompts vs Tools

**Important distinction:**
- **Tools** execute code and perform work
- **Prompts** are text templates that help format requests

In Lux MCP, prompts and tools have the same names, so slash commands map directly to both the tool and its associated prompt template.

## Model Selection

Each tool supports an optional `model` parameter. Available models include:

**OpenAI Models:**
- `gpt4`, `gpt4.1` - GPT-4 variants
- `o3`, `o3-pro` - Reasoning models
- `o4-mini` - Fast verification model
- `mini` - GPT-4o-mini

**OpenRouter Models:**
- `claude`, `opus`, `sonnet` - Anthropic models
- `gemini`, `flash` - Google models
- `llama3` - Meta's Llama

## Default Models

- **confer**: `gpt4.1`
- **traced_reasoning**: `o3-pro`
- **biased_reasoning**: `o3-pro` (primary), `o4-mini` (verifier)
- **plan**: `o3-pro`

## Session Management

All tools support an optional `session_id` parameter to maintain conversation context:

```
/lux:confer {"message": "Continue our discussion", "session_id": "my-session-123"}
```

Sessions automatically expire after 30 minutes of inactivity.