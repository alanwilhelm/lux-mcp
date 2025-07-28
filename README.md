# Lux MCP

A Model Context Protocol (MCP) server built in Rust that implements metacognitive monitoring for AI reasoning. It detects and prevents overthinking spirals, circular reasoning, and distractor fixation.

**⚠️ Important**: This is an MCP server designed to be used with Claude Desktop or other MCP-compatible clients. It cannot be tested directly via command line. See [USAGE_GUIDE.md](USAGE_GUIDE.md) for details.

## Overview

Lux MCP provides four specialized tools for enhanced AI reasoning:

- **confer**: Simple conversational AI with model selection
- **traced_reasoning**: Step-by-step reasoning with metacognitive monitoring
- **biased_reasoning**: Dual-model reasoning with bias detection and correction
- **plan**: Create structured plans with metacognitive monitoring

## Features

### Session Management
- **Conversation Isolation**: Each conversation gets its own metacognitive monitor
- **Automatic Cleanup**: Sessions are cleaned up every 5 minutes
- **Session Continuity**: Optional `session_id` parameter for maintaining context
- **30-minute TTL**: Sessions expire after 30 minutes of inactivity

### Metacognitive Monitoring
- **Circular Reasoning Detection**: Identifies when thoughts repeat (>85% similarity)
- **Distractor Fixation**: Detects drift from original query (<30% relevance)
- **Quality Tracking**: Monitors reasoning quality trends over time

### Model Support
- **OpenAI Models**: GPT-4, GPT-4 Turbo, O3, O4-mini
- **OpenRouter Models**: Claude, Gemini, and many others
- **Model Aliases**: Convenient shortcuts (e.g., "claude" → "anthropic/claude-4-sonnet")

## Installation

### Prerequisites
- Rust 1.70+ (for stable async support)
- API keys for OpenAI and/or OpenRouter

### Building

```bash
# Clone the repository
git clone https://github.com/yourusername/lux-mcp.git
cd lux-mcp

# Build the project
cargo build --release
```

### Environment Setup

Create a `.env` file or set environment variables:

```bash
# At least one API key is required
OPENAI_API_KEY=your-openai-key
OPENROUTER_API_KEY=your-openrouter-key

# Optional defaults
LUX_DEFAULT_CHAT_MODEL=gpt4.1              # Default: gpt4.1
LUX_DEFAULT_REASONING_MODEL=o3             # Default: o3
LUX_DEFAULT_BIAS_CHECKER_MODEL=o4-mini     # Default: o4-mini
```

## Usage

### Running the Server

```bash
# Run with cargo
cargo run --release

# Or run the binary directly
./target/release/lux-mcp

# With specific log level
RUST_LOG=debug cargo run --release
```

### Claude Desktop Integration

Add to your Claude Desktop configuration (`claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "lux": {
      "command": "/path/to/lux-mcp/target/release/lux-mcp",
      "env": {
        "OPENAI_API_KEY": "your-key",
        "OPENROUTER_API_KEY": "your-key",
        "RUST_LOG": "info"
      }
    }
  }
}
```

### Troubleshooting

If you encounter issues, especially "MCP error -32603: Chat error: Failed to complete chat request":

1. **Enable debug logging** by setting `RUST_LOG=debug` in your environment
2. **Check the TROUBLESHOOTING.md** file for detailed debugging steps
3. **Test your API keys** using the provided test scripts:
   ```bash
   ./test_openai_direct.sh  # Test OpenAI API directly
   ./test_mcp_debug.sh     # Test MCP server with debug logging
   ```

The server now includes comprehensive logging in all critical components to help diagnose connection issues.

## Tools

### confer

Simple conversational AI with model selection.

```json
{
  "tool": "confer",
  "arguments": {
    "message": "Explain quantum computing",
    "model": "claude",  // optional, defaults to LUX_DEFAULT_CHAT_MODEL
    "session_id": "optional-session-id"
  }
}
```

### traced_reasoning

Step-by-step reasoning with real-time metacognitive monitoring.

```json
{
  "tool": "traced_reasoning",
  "arguments": {
    "query": "Design a distributed cache system",
    "model": "o3",  // optional
    "max_steps": 10,  // optional, default: 10
    "temperature": 0.7,  // optional, default: 0.7
    "session_id": "optional-session-id",
    "guardrails": {  // optional
      "semantic_drift_check": true,
      "circular_reasoning_detection": true,
      "perplexity_monitoring": true
    }
  }
}
```

Response includes:
- Final answer
- Step-by-step reasoning trace
- Metrics (confidence, coherence, quality)
- Interventions (if any issues detected)

### biased_reasoning

Dual-model reasoning where a second model checks for biases.

**Note**: This tool ALWAYS uses the configured default models regardless of parameters:
- Primary reasoner: `LUX_DEFAULT_REASONING_MODEL` (default: o3-pro)
- Bias checker: `LUX_DEFAULT_BIAS_CHECKER_MODEL` (default: o4-mini)

```json
{
  "tool": "biased_reasoning",
  "arguments": {
    "query": "Should we adopt this new technology?",
    "max_steps": 10,  // optional
    "session_id": "optional-session-id",
    "bias_config": {  // optional
      "check_confirmation_bias": true,
      "check_anchoring_bias": true,
      "bias_threshold": 0.7
    }
  }
}
```

Response includes:
- Final answer
- Verified reasoning steps
- Bias analysis for each step
- Overall quality assessment

### plan

Create structured plans with metacognitive monitoring.

```json
{
  "tool": "plan",
  "arguments": {
    "goal": "Build a scalable e-commerce platform",
    "model": "o3-pro",  // optional, defaults to reasoning model
    "max_steps": 5,  // optional, default: 5
    "temperature": 0.7,  // optional, default: 0.7
    "session_id": "optional-session-id"
  }
}
```

Response includes:
- Complete plan text
- Structured steps with dependencies
- Monitoring feedback (if issues detected)
- Model used for planning

## Model Aliases

Convenient shortcuts for common models:

### OpenAI/Direct Models
- `gpt4`, `4` → `gpt-4`
- `gpt4.1`, `4.1` → `gpt-4-turbo-preview`
- `mini` → `gpt-4o-mini`
- `o3` → `o3`
- `o3-pro` → `o3-pro`
- `o4-mini`, `o4mini` → `o4-mini`

### OpenRouter Models
- `claude` → `anthropic/claude-4-sonnet`
- `opus` → `anthropic/claude-4-opus`
- `sonnet` → `anthropic/claude-3-sonnet`
- `gemini` → `google/gemini-2.5-pro`
- `flash` → `google/gemini-2.5-flash`
- `llama3` → `meta-llama/llama-3-70b-instruct`

## Current Limitations

### Placeholder Implementations
Some monitoring features are currently placeholders:
- **Perplexity monitoring**: Returns mock values
- **Attention entropy**: Not yet implemented
- **Semantic similarity**: Uses basic word overlap

### Production Readiness
- Basic monitoring is functional
- Session management is fully implemented
- Tool interfaces are stable
- Advanced monitoring algorithms are still in development

## Testing

```bash
# Run all tests
cargo test

# Test MCP server functionality
./test_server.sh

# Test individual tools
./test_chat.sh
./test_traced_reasoning.sh
./test_biased_reasoning.sh
./test_plan.sh
```

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                       MCP Server                             │
├─────────────────────────────────────────────────────────────┤
│                    Session Manager                           │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐            │
│  │  Session 1  │  │  Session 2  │  │  Session N  │ ...      │
│  │  Monitor 1  │  │  Monitor 2  │  │  Monitor N  │          │
│  └────────────┘  └────────────┘  └────────────┘            │
├─────────────────────────────────────────────────────────────┤
│                        Tools                                 │
│  ┌────────────┐  ┌──────────────┐  ┌──────────────┐        │
│  │   confer    │  │    traced     │  │    biased     │      │
│  │            │  │  reasoning    │  │  reasoning    │      │
│  └────────────┘  └──────────────┘  └──────────────┘        │
├─────────────────────────────────────────────────────────────┤
│                     LLM Clients                              │
│  ┌────────────┐  ┌──────────────┐                          │
│  │   OpenAI   │  │  OpenRouter   │                          │
│  └────────────┘  └──────────────┘                          │
└─────────────────────────────────────────────────────────────┘
```

## Contributing

We welcome contributions! Areas of interest:
- Implementing real semantic similarity algorithms
- Adding perplexity and attention entropy monitoring
- Improving circular reasoning detection
- Performance optimizations

## License

MIT License - see LICENSE file for details

## Acknowledgments

- NIRVANA project for metacognitive insights
- MCP specification by Anthropic
- Rust async ecosystem