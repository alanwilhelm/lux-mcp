# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Important Notes

- Tool names must match pattern `^[a-zA-Z0-9_-]{1,128}$` (no colons allowed)
- Tools use simple names: `confer`, `traced_reasoning`, `biased_reasoning`
- Slash commands in Claude can still use colon syntax like `/lux:confer`

## Project Overview

Lux MCP is a focused Model Context Protocol (MCP) server built in Rust that implements metacognitive monitoring for AI reasoning. It detects and prevents overthinking spirals, circular reasoning, and distractor fixation by "illuminating" thought processes.

## Key Commands

### Build and Development
```bash
# Build the project (required before running)
cargo build --release

# Run the server
./target/release/lux-mcp

# Run with specific log level
RUST_LOG=debug cargo run --release

# Check compilation without building
cargo check

# Format code
cargo fmt

# Run linter
cargo clippy

# Run tests
cargo test

# Build documentation
cargo doc --open
```

### Testing Tools
```bash
# Test MCP server initialization and basic functionality
./test_server.sh

# Test specific tools
./test_chat.sh              # Test lux:chat tool
./test_traced_reasoning.sh  # Test traced_reasoning tool  
./test_biased_reasoning.sh  # Test biased_reasoning tool
./test_mcp_protocol.sh      # Test MCP protocol compliance
```

## Architecture Overview

### Module Structure
- `src/main.rs` - Entry point, MCP server setup on stdio transport
- `src/server/` - MCP server implementation
  - `handler.rs` - Tool and prompt handlers
  - `mod.rs` - Server struct and initialization
- `src/tools/` - Tool implementations
  - `chat.rs` - Simple chat with model selection
  - `traced_reasoning.rs` - Step-by-step reasoning with monitoring
  - `biased_reasoning.rs` - Dual-model reasoning with bias detection
- `src/llm/` - LLM client abstraction
  - `client.rs` - Unified LLM interface
  - `openai.rs` - OpenAI API implementation
  - `openrouter.rs` - OpenRouter API implementation
  - `model_aliases.rs` - Model name resolution (gpt4.1 → gpt-4-turbo-preview)
- `src/monitoring/` - Metacognitive monitoring algorithms
- `src/models/` - Data models

### Core Concepts

1. **Tools** - Three main tools exposed via MCP:
   - `confer` - Conversational AI with model selection
   - `traced_reasoning` - Metacognitive reasoning with real-time monitoring
   - `biased_reasoning` - Dual-model verification with bias detection

2. **LLM Integration** - Supports both OpenAI and OpenRouter APIs with:
   - Model aliasing for convenience (e.g., "gpt4.1", "claude", "gemini")
   - Automatic provider detection based on model name
   - Configurable defaults via environment variables

3. **Monitoring System** - Detects:
   - Circular reasoning (>85% semantic similarity)
   - Distractor fixation (<30% relevance to original query)
   - Quality degradation over time

## Environment Configuration

Required environment variables (at least one):
- `OPENAI_API_KEY` - For OpenAI models
- `OPENROUTER_API_KEY` - For OpenRouter models

Optional defaults:
- `LUX_DEFAULT_CHAT_MODEL` - Default for confer (default: gpt4.1)
- `LUX_DEFAULT_REASONING_MODEL` - Default for traced_reasoning (default: o3)
- `LUX_DEFAULT_BIAS_CHECKER_MODEL` - Default for biased_reasoning verifier (default: o4-mini)

## MCP Protocol Details

The server implements MCP 1.0 with:
- Transport: stdio (stdin/stdout)
- Tools: `confer`, `traced_reasoning`, `biased_reasoning`, `illumination_status`
- Prompts: `illuminate_thinking`, `analyze_illumination`
- Capabilities: tools, prompts (no resources, no sampling)

## Testing Approach

1. **Unit Tests**: In Rust modules with `#[cfg(test)]`
2. **Integration Tests**: Shell scripts testing MCP protocol
3. **Manual Testing**: Use test scripts or Claude Desktop integration

To add tests, follow Rust conventions:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_something() {
        // Test implementation
    }
}
```

## Code Style Guidelines

- Follow Rust idioms and conventions
- Use `cargo fmt` before committing
- Address `cargo clippy` warnings
- Prefer explicit error handling with `Result<T, E>`
- Use `tracing` for logging, not `println!`
- Keep functions focused and under 50 lines when possible

## Common Tasks

### Adding a New Tool
1. Create new file in `src/tools/`
2. Define request/response structs with `serde::Deserialize/Serialize`
3. Implement tool logic
4. Add to `src/tools/mod.rs`
5. Register in `src/server/handler.rs` `list_tools()` and `call_tool()`

### Adding a Model Alias
1. Edit `src/llm/model_aliases.rs`
2. Add mapping in `resolve_model_name()` function
3. Update documentation in QUICKSTART.md

### Debugging MCP Issues
1. Enable debug logging: `RUST_LOG=debug`
2. Use `debug_mcp.sh` for protocol testing
3. Check stderr for server logs (stdout is for MCP protocol)

## ⚠️ CRITICAL: Model Configuration for Claude Code

### OpenAI Reasoning Models (o3/o4) Support ✅

Lux MCP now fully supports OpenAI's reasoning models with automatic API detection:

#### O3 Models (Advanced Reasoning)
- **API**: Uses OpenAI Responses API (`/v1/responses`)
- **Models**: `o3`, `o3-pro` (maps to `o3-pro-2025-06-10`), `o3-mini`
- **Parameter**: `max_output_tokens` (not `max_tokens`)
- **Temperature**: Not supported (ignored if provided)
- **Reasoning**: Uses nested `reasoning.effort` parameter set to "high" (API updated Jan 2025)
- **Response Time**: 30 seconds to several minutes due to deep reasoning
- **Token Limit**: 32,768 tokens for o3 models (maximum reasoning capability)

#### O4 Models (Fast Reasoning)
- **API**: Uses Chat Completions API with special parameters
- **Models**: `o4-mini`, `o4-mini-2025-04-16`
- **Parameter**: `max_completion_tokens` (not `max_tokens`)
- **Temperature**: Only supports default (1.0) - custom temperatures not allowed
- **Note**: May return empty responses with low token limits due to reasoning overhead

#### Standard Models
- **API**: Uses standard Chat Completions API
- **Models**: `gpt-4o`, `gpt-4o-mini`, `gpt-4-turbo-preview`, etc.
- **Parameter**: `max_tokens`

### Recommended Claude Code Configuration

```json
{
  "lux": {
    "command": "/Users/alan/Projects/_MCP/nirvana/lux-mcp/target/release/lux-mcp",
    "env": {
      "OPENAI_API_KEY": "your-openai-key",
      "OPENROUTER_API_KEY": "your-openrouter-key",
      "LUX_DEFAULT_CHAT_MODEL": "o3-pro",
      "LUX_DEFAULT_REASONING_MODEL": "o3-pro",
      "LUX_DEFAULT_BIAS_CHECKER_MODEL": "o4-mini",
      "RUST_LOG": "info"
    }
  }
}
```

### Interactive Planner Tool

The `planner` tool is an LLM-powered sequential planning system that generates planning steps using AI:

- **Model**: Uses `LUX_DEFAULT_REASONING_MODEL` (default: o3-pro) to generate each planning step
- **Step 1**: You provide the initial task description
- **Steps 2+**: The LLM generates planning content based on:
  - Previous steps in the planning history
  - Your guidance for what the step should focus on
  - Whether it's a branch, revision, or normal step
- **Features**:
  - Stateful: Maintains planning history across calls
  - Branching: Explore alternative approaches
  - Revisions: Update earlier steps with new insights
  - Deep thinking pauses for complex plans (≥5 steps)
  - Metacognitive monitoring for quality

Example usage:
```json
{
  "tool": "planner",
  "arguments": {
    "step": "Build a distributed task queue system",
    "step_number": 1,
    "total_steps": 7,
    "next_step_required": true,
    "model": "o3-pro",  // Optional, defaults to reasoning model
    "temperature": 0.7  // Optional, default 0.7
  }
}
```

### All Supported Models (Tested & Working)

**OpenAI Reasoning Models:**
- `o3` - Advanced reasoning (uses 8k tokens for primary, 5k for final)
- `o3-pro` - Professional reasoning (alias for o3-pro-2025-06-10)
- `o3-mini` - Smaller reasoning model
- `o4-mini` - Fast reasoning model (uses 10k tokens for bias checking due to reasoning overhead)

**OpenAI Standard Models:**
- `gpt-4o` - GPT-4 Optimized
- `gpt-4o-mini` - Smaller, faster GPT-4
- `gpt-4-turbo-preview` - Latest GPT-4 Turbo
- `gpt-4` - Standard GPT-4
- `gpt-3.5-turbo` - Fast, cheaper

**OpenRouter Models:**
- `claude` - Claude 3 Opus
- `gemini` - Google Gemini Pro
- `llama3` - Meta Llama 3 70B

### Technical Implementation Details

The OpenAI client (`src/llm/openai.rs`) automatically detects and routes requests:
1. O3 models → Responses API with `max_output_tokens`
2. O4 models → Chat Completions API with `max_completion_tokens`
3. Other models → Standard Chat Completions API with `max_tokens`

### Token Limits Configuration

**Unified token limits for all models:**
- All operations now use 10,000 tokens to ensure no truncation
- This applies to primary reasoning, bias checking, and final answers
- O3 models also use `reasoning_effort: "high"` for maximum capability

**Why 10,000 tokens?**
- O4 models use tokens for internal reasoning before generating output
- With insufficient tokens, they return empty responses
- O3 models perform deep reasoning that benefits from high limits
- Standard models work fine with 10,000 tokens (no negative impact)

See `TOKEN_LIMITS.md` for detailed configuration information.

## Best Practices for Using Lux Tools

### Passing File Context to External Models

Since Lux tools delegate reasoning to external models (o3, gpt-4, etc.) that don't have direct file access, Claude should proactively read and pass relevant file contents when using these tools:

1. **For `planner` tool** - When planning code-related tasks:
   ```json
   // DO: Read relevant files first
   {
     "tool": "planner",
     "arguments": {
       "step": "Refactor authentication system\n\nCurrent implementation:\n[FULL CONTENTS OF auth.js HERE]",
       "step_number": 1,
       "total_steps": 5,
       "next_step_required": true
     }
   }
   ```

2. **For `traced_reasoning` tool** - When debugging or analyzing code:
   ```json
   // DO: Include full file contents in the query
   {
     "tool": "traced_reasoning", 
     "arguments": {
       "thought": "Debug why login fails. Here's the current code:\n[FULL CONTENTS OF login.js HERE]\n\nError log:\n[RELEVANT ERROR MESSAGES HERE]",
       "thought_number": 1,
       "total_thoughts": 5,
       "next_thought_needed": true
     }
   }
   ```

3. **For `biased_reasoning` tool** - When reviewing code or documentation:
   ```json
   // DO: Pass complete context
   {
     "tool": "biased_reasoning",
     "arguments": {
       "query": "Review this API design for potential biases:\n[FULL API SPECIFICATION HERE]\n\nCurrent user model:\n[USER MODEL SCHEMA HERE]"
     }
   }
   ```

### Guidelines:
- **Always read full files** before calling lux tools for code-related tasks
- **Include file paths** in the context so models understand structure
- **Pass relevant dependencies** - if analyzing a function, include imported modules
- **Add execution context** - error messages, logs, test results
- **Specify relationships** - explain how files connect to each other

This approach maximizes the reasoning capabilities of external models while maintaining security boundaries.