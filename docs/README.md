# Lux MCP Documentation

## Overview

Lux MCP (Model Context Protocol) is a metacognitive monitoring server that provides sophisticated reasoning tools with quality assurance capabilities. This documentation covers all available tools, with a focus on the sequential thinking capabilities.

## Quick Start

### Installation

```bash
# Build the project
cargo build --release

# Run the server
./target/release/lux-mcp

# Or with logging
RUST_LOG=info ./target/release/lux-mcp
```

### Configuration

Set environment variables for LLM access:

```bash
export OPENAI_API_KEY="your-openai-key"
export OPENROUTER_API_KEY="your-openrouter-key"
export LUX_MODEL_REASONING="gpt-5"      # Main reasoning model
export LUX_MODEL_NORMAL="gpt-5"         # Main normal model
export LUX_MODEL_MINI="gpt-5-mini"      # Mini model for fast tasks
```

## Documentation Index

### Core Documentation

1. **[Sequential Thinking Guide](./sequential-thinking.md)**
   - Comprehensive overview of sequential thinking tools
   - Architecture and design patterns
   - Best practices and migration guides

2. **[API Reference](./sequential-thinking-api.md)**
   - Complete API specifications
   - Request/response formats
   - Error codes and type definitions

3. **[Examples](./sequential-thinking-examples.md)**
   - Real-world usage scenarios
   - Integration patterns
   - Testing strategies

4. **[Tool Comparison](./sequential-thinking-comparison.md)**
   - Detailed feature comparisons
   - Decision trees for tool selection
   - Cost and performance analysis

5. **[Hybrid Biased Reasoning](./hybrid-biased-reasoning.md)**
   - Claude + external LLM bias detection
   - Integration patterns and best practices
   - Performance optimization strategies

### Tool Categories

#### 1. Sequential Reasoning Tools

Tools for step-by-step problem solving and analysis:

- **`sequential_thinking`** - Manual thought organization without LLM
- **`sequential_thinking_external`** - AI-powered sequential reasoning
- **`traced_reasoning`** - Deep analysis with metacognitive monitoring

#### 2. Planning Tools

High-level planning and structure:

- **`planner`** - Interactive sequential planning with AI

#### 3. Analysis Tools

Specialized analysis capabilities:

- **`biased_reasoning`** - Dual-model bias detection
- **`hybrid_biased_reasoning`** - Claude reasoning with external bias checking
- **`confer`** - Simple conversational AI

## Tool Selection Guide

### Quick Decision Matrix

| Need | Recommended Tool |
|------|-----------------|
| Full control, no AI | `sequential_thinking` |
| AI assistance with control | `sequential_thinking_external` |
| Critical analysis with quality checks | `traced_reasoning` |
| High-level planning | `planner` |
| Bias detection | `biased_reasoning` |
| Simple conversation | `confer` |

### By Use Case

#### Software Development
- **Architecture Design**: `planner` → `traced_reasoning`
- **Code Review**: `sequential_thinking_external`
- **Debugging**: `sequential_thinking` (full control)
- **Documentation**: `sequential_thinking_external`

#### Research & Analysis
- **Literature Review**: `traced_reasoning`
- **Data Analysis Planning**: `planner`
- **Hypothesis Testing**: `biased_reasoning`
- **Report Writing**: `sequential_thinking_external`

#### Decision Making
- **Strategic Planning**: `planner`
- **Risk Assessment**: `biased_reasoning`
- **Option Evaluation**: `traced_reasoning`
- **Decision Documentation**: `sequential_thinking`

## Key Features

### 1. Session Management
All tools support session-based state management for maintaining context across multiple interactions.

### 2. Branching & Revisions
Explore alternative paths and revise previous thoughts without losing context.

### 3. Model Flexibility
Choose from various LLM models (GPT-4, O3, Claude, etc.) based on your needs.

### 4. Quality Monitoring
Advanced tools provide metacognitive monitoring, semantic drift detection, and intervention systems.

### 5. Cost Control
From free (manual) to low-cost (simple AI) to premium (full monitoring) options.

## Integration Examples

### With Claude Desktop

Add to your `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "lux": {
      "command": "/path/to/lux-mcp",
      "env": {
        "OPENAI_API_KEY": "your-key",
        "RUST_LOG": "info"
      }
    }
  }
}
```

### Programmatic Usage

```python
import requests
import json

def call_lux_tool(tool_name, arguments):
    response = requests.post("http://localhost:8080/tools/call", json={
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": tool_name,
            "arguments": arguments
        }
    })
    return response.json()

# Example: Sequential thinking
result = call_lux_tool("sequential_thinking", {
    "thought": "Analyze the problem",
    "thought_number": 1,
    "total_thoughts": 5,
    "next_thought_needed": True
})
```

## Performance Considerations

### Response Times
- **Instant** (<1ms): `sequential_thinking`
- **Fast** (0.5-5s): `sequential_thinking_external`, `confer`
- **Medium** (2-30s): `traced_reasoning`, `planner`
- **Slow** (10-60s): `biased_reasoning`

### Token Usage
- **None**: `sequential_thinking`
- **Low** (~2K/step): `sequential_thinking_external`
- **Medium** (~10K/step): `planner`
- **High** (~20K+/step): `traced_reasoning`, `biased_reasoning`

## Best Practices

### 1. Start Simple
Begin with `sequential_thinking` or `sequential_thinking_external`, then upgrade to more sophisticated tools as needed.

### 2. Use Sessions
Always provide session IDs for multi-step reasoning to maintain context.

### 3. Monitor Costs
Track token usage, especially with premium models and monitoring features.

### 4. Mix and Match
Combine tools for optimal results - use `planner` for structure, `sequential_thinking` for sensitive data, and `traced_reasoning` for critical validation.

### 5. Handle Errors Gracefully
Implement retry logic and fallback strategies for API failures.

## Troubleshooting

### Common Issues

1. **"API key not configured"**
   - Ensure environment variables are set
   - Check key validity

2. **"Session not found"**
   - Use consistent session IDs
   - Check for server restarts

3. **High latency**
   - Consider using smaller models
   - Disable unnecessary monitoring features
   - Check network connectivity

4. **Rate limits**
   - Implement client-side rate limiting
   - Use exponential backoff
   - Consider caching responses

### Debug Mode

Enable detailed logging:

```bash
RUST_LOG=debug ./target/release/lux-mcp
```

## Architecture Overview

```
┌─────────────────────────────────────────┐
│            Lux MCP Server              │
├─────────────────────────────────────────┤
│                                         │
│  ┌─────────────────────────────────┐   │
│  │     Sequential Thinking         │   │
│  │  ┌──────────┐  ┌─────────────┐  │   │
│  │  │  Simple  │  │  External   │  │   │
│  │  │  (Manual)│  │(AI-Powered) │  │   │
│  │  └──────────┘  └─────────────┘  │   │
│  └─────────────────────────────────┘   │
│                                         │
│  ┌─────────────────────────────────┐   │
│  │      Advanced Reasoning         │   │
│  │  ┌──────────┐  ┌─────────────┐  │   │
│  │  │  Traced  │  │   Biased    │  │   │
│  │  │Reasoning │  │  Reasoning  │  │   │
│  │  └──────────┘  └─────────────┘  │   │
│  └─────────────────────────────────┘   │
│                                         │
│  ┌─────────────────────────────────┐   │
│  │      Planning & Chat            │   │
│  │  ┌──────────┐  ┌─────────────┐  │   │
│  │  │ Planner  │  │   Confer    │  │   │
│  │  └──────────┘  └─────────────┘  │   │
│  └─────────────────────────────────┘   │
│                                         │
│  ┌─────────────────────────────────┐   │
│  │        Core Services            │   │
│  │  - Session Management           │   │
│  │  - LLM Integration              │   │
│  │  - Monitoring & Synthesis       │   │
│  │  - Database Service (optional)  │   │
│  └─────────────────────────────────┘   │
└─────────────────────────────────────────┘
```

## Contributing

### Development Setup

```bash
# Clone the repository
git clone https://github.com/your-org/lux-mcp.git
cd lux-mcp

# Install dependencies
cargo build

# Run tests
cargo test

# Format code
cargo fmt

# Run linter
cargo clippy
```

### Adding New Tools

1. Create tool implementation in `src/tools/`
2. Add to `src/tools/mod.rs`
3. Register in `src/server/handler.rs`
4. Update documentation
5. Add tests

## License

[License information here]

## Support

- **Issues**: [GitHub Issues](https://github.com/your-org/lux-mcp/issues)
- **Documentation**: This directory
- **Examples**: See [sequential-thinking-examples.md](./sequential-thinking-examples.md)

## Version History

### v0.1.0 (Current)
- Initial release with sequential thinking tools
- Core reasoning capabilities
- MCP 1.0 support

### Roadmap
- Database persistence
- Streaming responses
- WebSocket support
- Cross-session analysis
- Enhanced monitoring capabilities

---

For detailed information about specific tools, please refer to the individual documentation files listed above.