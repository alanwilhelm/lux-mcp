# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Working Directory
- The project root directory where Lux MCP is installed
- /tmp for temporary operations

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

## Quick Configuration Setup

### Using setup_config Tool

The easiest way to configure Lux MCP is using the `setup_config` tool. This tool guides you through creating or updating the .env file:

```json
{
  "tool": "setup_config",
  "arguments": {
    "openai_api_key": "YOUR_KEY_HERE",
    "openrouter_api_key": "YOUR_KEY_HERE",  // Optional
    "use_advanced_models": true  // Use GPT-5 (true) or GPT-4o (false)
  }
}
```

**What the tool does:**
1. Checks if .env file exists
2. Generates complete configuration template
3. Provides step-by-step instructions for creating/updating .env
4. Configures all model preferences automatically
5. User only needs to provide their API keys

**The tool will instruct you to:**
- Use Write tool to create .env if it doesn't exist
- Use Edit tool to update .env if it exists
- Verify the configuration was saved correctly

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

1. **Tools** - Main tools exposed via MCP:
   - `setup_config` - Environment configuration helper (guides .env setup)
   - `confer` - Conversational AI with model selection
   - `traced_reasoning` - Metacognitive reasoning with real-time monitoring
   - `biased_reasoning` - Step-by-step dual-model verification with visible bias detection
     - **Note**: Always uses configured defaults (o3-pro for reasoning, o4-mini for bias checking)
     - Works step-by-step with visible individual interactions
     - Each reasoning step is followed by a visible bias analysis step
     - Tracks context in a session list similar to sequential thinking
     - Returns step_type, step_number, content, model_used, and next_action

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

Model configuration:
- `LUX_MODEL_REASONING` - Main reasoning model (default: gpt-5)
- `LUX_MODEL_NORMAL` - Main normal model (default: gpt-5)
- `LUX_MODEL_MINI` - Mini model for fast tasks (default: gpt-5-mini)

Named model aliases (optional):
- `LUX_MODEL_OPUS` - Maps 'opus' to specific model (default: anthropic/claude-4.1-opus)
- `LUX_MODEL_SONNET` - Maps 'sonnet' to specific model (default: anthropic/claude-4-sonnet)
- `LUX_MODEL_GROK` - Maps 'grok' to specific model (default: x-ai/grok-beta)

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

### 🔴 STRICT MODEL POLICY - DO NOT VIOLATE 🔴

**ONLY TWO MODELS CAN USE OPENAI:**
1. `gpt-5` - Uses OpenAI Responses API with 128K tokens
2. `gpt-5-mini` - Uses OpenAI Responses API with 16K tokens

**ALL OTHER MODELS MUST USE OPENROUTER - NO EXCEPTIONS**

Any attempt to use other models with OpenAI (like gpt-4o, o3, o4) will be REJECTED with an error.
This is a HARD requirement. The code enforces this strictly in `model_aliases.rs`.

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

### Recommended Claude Desktop Configuration

```json
{
  "lux": {
    "command": "/path/to/lux-mcp/target/release/lux-mcp",
    "env": {
      "OPENAI_API_KEY": "your-openai-key",
      "OPENROUTER_API_KEY": "your-openrouter-key",
      "LUX_MODEL_REASONING": "gpt-5",
      "LUX_MODEL_NORMAL": "gpt-5",
      "LUX_MODEL_MINI": "gpt-5-mini",
      "LUX_MODEL_OPUS": "anthropic/claude-4.1-opus",
      "LUX_MODEL_SONNET": "anthropic/claude-4-sonnet",
      "LUX_MODEL_GROK": "x-ai/grok-beta",
      "RUST_LOG": "info"
    }
  }
}
```

### GPT-5 Advanced Features

GPT-5 now uses the **Responses API** with:
- **Maximum Reasoning**: `reasoning.effort: "high"` for all calls
- **High Verbosity**: `text.verbosity: "high"` for detailed responses
- **128K Token Support**: Full 128,000 completion tokens available
- **Temperature Control**: Full temperature support unlike O3/O4 models

### Interactive Planner Tool (Enhanced with File Reading)

The `planner` tool is an AI-powered sequential planning system with **DIRECT FILE ACCESS**:

- **Model**: Uses configured reasoning model (`LUX_MODEL_REASONING`, defaults to gpt-5)
- **File Reading**: Auto-discovers and examines project files to ground plans in reality
- **Mandatory Actions**: Returns MANDATORY actions that MUST be executed by the caller
- **Step 1**: You provide the initial task description
- **Steps 2+**: The LLM generates ACTIONABLE, IMPLEMENTATION-READY content based on:
  - Previous steps in the planning history
  - **Actual project files and code structure** (reads files directly)
  - Your guidance for what the step should focus on
  - Whether it's a branch, revision, or normal step
- **Features**:
  - **Auto-Discovery**: Automatically finds relevant project files (README, config, code)
  - **File Context**: Reads and analyzes actual code/config files (up to 15KB per file)
  - **Mandatory Actions**: Returns actions you MUST take (marked with ⚠️)
  - **Implementation-Ready**: Includes specific file paths, function names, commands
  - **Session File Cache**: Caches files for efficiency across planning steps
  - Stateful: Maintains planning history and file cache across calls
  - Branching: Explore alternative approaches
  - Revisions: Update earlier steps with new insights
  - Deep thinking pauses for complex plans (≥5 steps)

Example usage:
```json
{
  "tool": "planner",
  "arguments": {
    "step": "Build a distributed task queue system",
    "step_number": 1,
    "total_steps": 7,
    "next_step_required": true,
    "auto_discover_files": true,  // Default: true - auto-find relevant files
    "file_paths": ["/src/queue.py", "/config.yaml"],  // Optional: specific files
    "include_file_contents": true,  // Default: true - read file contents
    "model": "gpt-5",  // Optional, defaults to gpt-5 now (not o3-pro)
    "temperature": 0.7  // Optional, default 0.7
  }
}
```

**Response includes:**
- `mandatory_actions`: Array of actions you MUST take (e.g., "⚠️ MANDATORY: You MUST examine API route definitions")
- `files_examined`: Files that were read during planning
- `recommended_files`: Files to examine for implementation
- `step_content`: The actual planning step with specific implementation details

**Auto-Discovery Patterns:**
- Looks for: README.md, package.json, Cargo.toml, requirements.txt, Makefile
- API planning: Searches for *api*, *route* files
- Database planning: Searches for *model*, *schema*, migrations
- Testing: Searches for *test*, *spec* files
- Security: Searches for *auth*, *security* files

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

## Using Named Models and Cost Optimization

### Named Model Aliases
You can use simple names for popular models:
- `"opus"` - Maps to Claude 4.1 Opus (configurable via LUX_MODEL_OPUS)
- `"sonnet"` - Maps to Claude 4 Sonnet (configurable via LUX_MODEL_SONNET)
- `"grok"` - Maps to latest Grok (configurable via LUX_MODEL_GROK)

Example:
```json
{
  "tool": "confer",
  "arguments": {
    "message": "Explain quantum computing",
    "model": "opus"  // Uses Claude 4.1 Opus
  }
}
```

### Cost Savings with Mini Model
All tools support a `use_mini` parameter for cost-effective operations:

```json
{
  "tool": "confer",
  "arguments": {
    "message": "What's 2+2?",
    "use_mini": true  // Uses gpt-5-mini instead of gpt-5
  }
}
```

This works for `confer`, `planner`, `traced_reasoning`, and other tools.

## Direct File Access

Lux MCP tools support optional file reading. When you provide file paths, tools read them directly instead of requiring you to pass the contents.

### How It Works
- Tools accept an optional `file_paths` parameter
- Files are read directly by the tool (read-only)
- Saves tokens since you don't need to pass file contents
- All tools support this feature

### IMPORTANT: How to Use File Reading

**CORRECT** - Pass paths in the `file_paths` array:
```json
{
  "tool": "confer",
  "arguments": {
    "message": "Analyze this code for security issues",
    "file_paths": ["/path/to/file1.js", "/path/to/file2.py"],
    "model": "gpt-5"
  }
}
```

**INCORRECT** - Don't just mention files in the message:
```json
{
  "tool": "confer",
  "arguments": {
    "message": "Read /path/to/file.js and analyze it",  // ❌ Won't work!
    "model": "gpt-5"
  }
}
```

The server reads the files and includes their contents automatically!

## Best Practices for Using Lux Tools

### Using File Access

When tools need file context, you can provide file paths directly:

1. **For `planner` tool**:
   ```json
   {
     "tool": "planner",
     "arguments": {
       "step": "Refactor authentication system",
       "file_paths": ["/app/auth.js", "/app/config.js"],
       "step_number": 1,
       "total_steps": 5,
       "next_step_required": true
     }
   }
   ```

2. **For `confer` tool**:
   ```json
   {
     "tool": "confer", 
     "arguments": {
       "message": "Debug why login fails",
       "file_paths": ["/app/login.js", "/logs/error.log"],
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
       "query": "Review this API design for potential biases:\n[FULL API SPECIFICATION HERE]\n\nCurrent user model:\n[USER MODEL SCHEMA HERE]",
       "max_analysis_rounds": 3  // Optional, defaults to 3
     }
   }
   ```
   
   **Step-by-Step Response Format:**
   - Each call returns a single step with:
     - `step_type`: Query, Reasoning, BiasAnalysis, Correction, Guidance, or Synthesis
     - `step_number`: Current step number
     - `content`: The actual content of this step
     - `model_used`: Which model was used (e.g., "o3-pro" or "o4-mini")
     - `next_action`: What should happen next (BiasCheck, ContinueReasoning, etc.)
     - `session_status`: Overall progress tracking

### Guidelines:
- **Always read full files** before calling lux tools for code-related tasks
- **Include file paths** in the context so models understand structure
- **Pass relevant dependencies** - if analyzing a function, include imported modules
- **Add execution context** - error messages, logs, test results
- **Specify relationships** - explain how files connect to each other

This approach maximizes the reasoning capabilities of external models while maintaining security boundaries.

## Sequential Thinking Tools

Lux MCP now provides two new sequential thinking tools for step-by-step reasoning:

- **`sequential_thinking`** - Simple state tracker for manual thought organization (no LLM)
- **`sequential_thinking_external`** - AI-powered sequential reasoning with LLM integration

For comprehensive documentation, see the [docs/](./docs/) directory:
- [Sequential Thinking Guide](./docs/sequential-thinking.md) - Complete overview and architecture
- [API Reference](./docs/sequential-thinking-api.md) - Detailed API specifications
- [Examples](./docs/sequential-thinking-examples.md) - Real-world usage scenarios
- [Tool Comparison](./docs/sequential-thinking-comparison.md) - Detailed feature comparison with other tools