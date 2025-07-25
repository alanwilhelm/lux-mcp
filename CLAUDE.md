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