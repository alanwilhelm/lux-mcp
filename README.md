# Lux MCP - Metacognitive Model Context Protocol Server

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![MCP](https://img.shields.io/badge/MCP-1.0-blue?style=for-the-badge)](https://modelcontextprotocol.io/)
[![License](https://img.shields.io/badge/license-MIT-green?style=for-the-badge)](LICENSE)

Lux MCP is a Model Context Protocol server that "illuminates" AI reasoning by providing metacognitive monitoring, conversation threading, and quality tracking for LLM interactions.

## 🎯 Revolutionary Direct File Access

### 📁 **Third-Party LLMs Read Files Directly - Bypassing the Orchestrator**
Unlike traditional MCP servers where the host (Claude/ChatGPT) must read files and pass contents, **Lux MCP enables third-party LLMs to read files directly on the server side**. This means:

- ✅ **Token Savings**: The orchestrator doesn't waste tokens on file contents
- ✅ **Privacy**: File contents never pass through the main model
- ✅ **Speed**: Direct server-side reading is faster
- ✅ **Scale**: Can process large files without context limits
- ✅ **Security**: Files stay within your MCP server boundary

Example: When you provide `file_paths: ["./src/main.rs"]`, the external LLM (GPT-5, O3, etc.) reads the file directly on the server, NOT through Claude/ChatGPT!

## 🌟 Other Key Features

- **🧠 Metacognitive Monitoring** - Detects and prevents circular reasoning, distractor fixation, and quality degradation
- **🧵 Conversation Threading** - Maintains context across tool calls with session management
- **🔍 Bias Detection** - Dual-model reasoning with step-by-step bias analysis
- **📊 Quality Metrics** - Tracks confidence, clarity, and coherence with trend analysis
- **💾 Stateless Design** - Fast in-memory operation, no database required
- **🚀 O3/O4/GPT-5 Support** - Full support for OpenAI's latest reasoning models
- **🎯 Optimal Token Allocation** - Automatically maximizes token limits per model (GPT-5: 128K tokens)

## 📦 Installation

### Prerequisites
- Rust 1.70+ 
- OpenAI API key and/or OpenRouter API key
- PostgreSQL (optional, for persistence)

### Build from Source
```bash
git clone https://github.com/yourusername/lux-mcp.git
cd lux-mcp
cargo build --release
```

### Environment Setup
```bash
# Required (at least one)
export OPENAI_API_KEY="sk-..."
export OPENROUTER_API_KEY="sk-..."

# Model configuration
export LUX_MODEL_REASONING="gpt-5"      # Main reasoning model
export LUX_MODEL_NORMAL="gpt-5"         # Main normal model
export LUX_MODEL_MINI="gpt-5-mini"      # Mini model for fast tasks

# Named OpenRouter models (optional)
export LUX_MODEL_OPUS="anthropic/claude-4.1-opus"      # Maps 'opus' to Claude 4.1
export LUX_MODEL_SONNET="anthropic/claude-4-sonnet"    # Maps 'sonnet' to Claude 4
export LUX_MODEL_GROK="x-ai/grok-beta"                 # Maps 'grok' to latest Grok

# Logging
export RUST_LOG="info"
```

## 🚀 Quick Start

### Automated Configuration Setup

**NEW!** Use the `setup_config` tool to automatically configure your environment:

```json
{
  "tool": "setup_config",
  "arguments": {
    "openai_api_key": "sk-...",         // Your OpenAI API key
    "openrouter_api_key": "sk-...",     // Your OpenRouter API key (optional)
    "use_advanced_models": true         // Use GPT-5/O3 (true) or GPT-4o (false)
  }
}
```

The tool will:
1. Guide the host LLM through creating/updating your .env file
2. Configure all necessary environment variables
3. Set up model preferences
4. Provide step-by-step instructions

All you need to provide are your API keys!

### Manual Claude Desktop Configuration
Add to your Claude Desktop config (`~/Library/Application Support/Claude/claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "lux": {
      "command": "/path/to/lux-mcp/target/release/lux-mcp",
      "env": {
        "OPENAI_API_KEY": "your-key",
        "LUX_MODEL_REASONING": "gpt-5",
        "LUX_MODEL_NORMAL": "gpt-5",
        "LUX_MODEL_MINI": "gpt-5-mini",
        "RUST_LOG": "info"
      }
    }
  }
}
```

### Direct Usage
```bash
# Start the server
./target/release/lux-mcp

# In another terminal, send MCP requests
echo '{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "confer",
    "arguments": {
      "message": "Hello! What is metacognition?"
    }
  },
  "id": 1
}' | nc localhost 3333
```

## 📁 Direct File Access

All Lux MCP tools support optional read-only file access. When you provide file paths, tools read them directly rather than requiring the host LLM to pass contents.

### Benefits
- **Token Savings**: Tools read files directly, saving tokens
- **Security**: Read-only access - cannot modify files
- **Performance**: Faster processing without data transfer overhead

### Usage
All tools accept optional `file_paths` parameter:
```json
{
  "tool": "confer",
  "arguments": {
    "message": "Analyze this code",
    "file_paths": ["/app/main.rs", "/app/lib.rs"]
  }
}
```

### Agent Configuration
**⚠️ IMPORTANT**: Add [`context_helper.txt`](context_helper.txt) to your agent's memory file (Claude.md, AGENTS.md, etc.) for complete tool documentation and usage patterns.

```bash
# Copy context helper to your agent configuration
cat context_helper.txt >> ~/.config/claude/CLAUDE.md
# or
cat context_helper.txt >> your-project/AGENTS.md
```

### Example with Files
```json
{
  "tool": "confer",
  "arguments": {
    "message": "Review this authentication system for security issues",
    "file_paths": ["/app/auth.py", "/app/config.py"],
    "model": "gpt-5"
  }
}
```

## 🛠️ Available Tools (9 Total)

All tools with external LLM integration support **direct file reading** from your filesystem.

### `setup_config` - Environment Configuration Helper
Automatically configure Lux MCP environment settings. Guides the host LLM through creating or updating the .env file.

**Features:**
- Creates or updates .env configuration file
- Configures API keys (OpenAI, OpenRouter)
- Sets up model preferences (GPT-5, GPT-4o, etc.)
- Provides step-by-step instructions for the host LLM
- User only needs to provide API keys

```json
{
  "tool": "setup_config",
  "arguments": {
    "openai_api_key": "sk-...",
    "use_advanced_models": true
  }
}
```

### `confer` - Conversational AI
Simple chat with model selection, threading support, and **file reading capability**.

**Features:**
- Named model shortcuts: `"opus"`, `"sonnet"`, `"grok"` (configurable via env)
- Cost-saving mode: Set `"use_mini": true` to use mini model
- File reading: Include `"file_paths"` to provide context

⚠️ **IMPORTANT**: The `max_tokens` parameter is **NOT supported** and will be **IGNORED** if provided.
- GPT-5: Always uses 128,000 tokens
- O3: Always uses 100,000 tokens  
- O4: Always uses 50,000 tokens
- Other models: 20,000 tokens

```json
{
  "tool": "confer",
  "arguments": {
    "message": "Your message here",
    "model": "opus",  // Use named model alias
    "use_mini": false,  // Or true for cost savings
    "continuation_id": "thread-123"
  }
}
```

### `traced_reasoning` - Step-by-Step Reasoning
Metacognitive reasoning with monitoring and synthesis.

⚠️ **IMPORTANT**: Uses **MAXIMUM** token allocation for deep reasoning:
- GPT-5: 200,000 tokens
- O3: 100,000 tokens
- Other models: 20,000 tokens

```json
{
  "tool": "traced_reasoning",
  "arguments": {
    "thought": "How can we optimize database queries?",
    "thought_number": 1,
    "total_thoughts": 5,
    "next_thought_needed": true,
    "model": "gpt5"
  }
}
```

### `biased_reasoning` - Bias Detection
Dual-model reasoning with bias analysis.
```json
{
  "tool": "biased_reasoning",
  "arguments": {
    "query": "What programming language should I learn?",
    "max_analysis_rounds": 3
  }
}
```

### `planner` - Interactive Planning
LLM-powered sequential planning.
```json
{
  "tool": "planner",
  "arguments": {
    "step": "Design a microservices architecture",
    "step_number": 1,
    "total_steps": 7,
    "next_step_required": true
  }
}
```

### `hybrid_biased_reasoning` - Claude + External Bias Check
Claude provides reasoning, external LLM checks for bias with **file context support**.
```json
{
  "tool": "hybrid_biased_reasoning",
  "arguments": {
    "reasoning_step": "This authentication system is secure",
    "file_paths": ["/app/auth.py", "/app/security.py"],
    "context": "Security audit",
    "session_id": "audit-001"
  }
}
```

### `illumination_status` - System Status
Check metacognitive monitoring status.
```json
{
  "tool": "illumination_status",
  "arguments": {}
}
```

## ⚡ Token Allocation Policy

**Lux MCP uses OPTIMAL TOKEN ALLOCATION for maximum intelligence.**

### Why No `max_tokens` Parameter?
- The `max_tokens` parameter is **deliberately removed** from all tools
- Any `max_tokens` value passed will be **silently ignored**
- Each model gets its optimal token allocation automatically
- This ensures models have maximum space for deep reasoning

### Token Limits by Model

| Model | Chat (confer) | Reasoning | Planner |
|-------|--------------|-----------|---------|
| GPT-5 | 128,000 | 200,000 | 200,000 |
| O3/O3-Pro | 100,000 | 100,000 | 100,000 |
| O4 | 50,000 | 50,000 | 50,000 |
| Mini models* | 16,000 | 16,000 | 16,000 |
| Standard | 20,000 | 20,000 | 20,000 |

*Mini models include: gpt-4o-mini, o4-mini, gpt-5-mini (if/when available)

**Note**: O4 models and GPT-5-mini only support default temperature (1.0). Custom temperature values will be ignored.

### Example: Ignored Parameters
```json
{
  "tool": "confer",
  "arguments": {
    "message": "Hello",
    "model": "gpt5",
    "max_tokens": 100  // ← This is IGNORED! GPT-5 will use 128,000
  }
}
```

## 🏗️ Architecture

```
lux-mcp/
├── src/
│   ├── main.rs              # Entry point
│   ├── server/              # MCP server implementation
│   │   ├── mod.rs          # Server struct
│   │   └── handler.rs      # Request handlers
│   ├── tools/              # Tool implementations
│   │   ├── chat.rs         # Confer tool
│   │   ├── traced_reasoning.rs
│   │   ├── biased_reasoning.rs
│   │   └── planner.rs
│   ├── threading/          # Conversation threading
│   │   ├── manager.rs      # Thread management
│   │   ├── context.rs      # Thread context
│   │   ├── synthesis.rs    # Synthesis integration
│   │   ├── quality.rs      # Quality metrics
│   │   └── persistence.rs  # Database checkpoints
│   ├── monitoring/         # Metacognitive monitoring
│   │   ├── circular_reasoning.rs
│   │   ├── distractor_fixation.rs
│   │   └── quality_degradation.rs
│   ├── llm/               # LLM integrations
│   │   ├── client.rs      # Unified interface
│   │   ├── openai.rs      # OpenAI/O3/O4 support
│   │   └── openrouter.rs  # OpenRouter support
│   └── db/                # Database layer
│       ├── connection.rs
│       └── service.rs
├── crates/
│   ├── lux_synthesis/     # Synthesis engine
│   └── lux_synthesis_db/  # Database bindings
└── migrations/            # Database schema
```

## 🧠 How It Works

### Metacognitive Monitoring
Lux monitors reasoning in real-time to detect:
- **Circular Reasoning**: Semantic similarity > 85% for 3+ consecutive thoughts
- **Distractor Fixation**: Relevance < 30% to original query
- **Quality Degradation**: Quality drop > 40% from baseline

### Threading System
- Conversations persist across tool calls via `continuation_id`
- Threads expire after 3 hours (configurable)
- Context intelligently reconstructed within token limits
- Quality metrics tracked per thread

### Synthesis Integration
- Tracks insights and actions across reasoning sessions
- Links synthesis states to conversation threads
- Builds knowledge graph of related concepts

## 📊 Supported Models

### GPT-5 Support (NEW!)
**GPT-5 uses the advanced Responses API with:**
- **Maximum Reasoning**: Always uses `reasoning.effort: "high"` for deepest analysis
- **High Verbosity**: Uses `text.verbosity: "high"` for detailed responses
- **128K Tokens**: Supports up to 128,000 completion tokens
- **Temperature Control**: Full temperature support for creativity control

### OpenAI Models
- **GPT-5 Series**: `gpt-5`, `gpt-5-mini` (uses Responses API with max reasoning)
- **O3 Series**: `o3`, `o3-pro`, `o3-mini` (deep reasoning, 30s-5min response)
- **O4 Series**: `o4-mini` (fast reasoning with special handling)
- **GPT-4**: `gpt-4o`, `gpt-4o-mini`, `gpt-4-turbo-preview`
- **GPT-3.5**: `gpt-3.5-turbo`

### OpenRouter Models
- **Claude**: `anthropic/claude-3-opus`
- **Gemini**: `google/gemini-2.5-pro`
- **Llama**: `meta-llama/llama-3-70b`

### Model Aliases
```
"gpt5" → "gpt-5" (available August 2025)
"gpt4.1" → "gpt-4-turbo-preview"
"claude" → "anthropic/claude-3-opus"
"gemini" → "google/gemini-2.5-pro"
"o3-pro" → "o3-pro-2025-06-10"
```

## 🔧 Configuration

### Environment Variables

#### Model Configuration
| Variable | Description | Default |
|----------|-------------|---------|
| `LUX_MODEL_REASONING` | Main reasoning model for complex tasks | `gpt-5` |
| `LUX_MODEL_NORMAL` | Main normal model for standard tasks | `gpt-5` |
| `LUX_MODEL_MINI` | Mini model for fast/simple tasks | `gpt-5-mini` |
| `LUX_MODEL_OPUS` | Custom mapping for 'opus' alias | `anthropic/claude-4.1-opus` |
| `LUX_MODEL_SONNET` | Custom mapping for 'sonnet' alias | `anthropic/claude-4-sonnet` |
| `LUX_MODEL_GROK` | Custom mapping for 'grok' alias | `x-ai/grok-beta` |

#### API Keys & Settings
| Variable | Description | Default |
|----------|-------------|---------|
| `OPENAI_API_KEY` | OpenAI API key | Required* |
| `OPENROUTER_API_KEY` | OpenRouter API key | Required* |
| `RUST_LOG` | Log level | `info` |

*At least one API key required

## 📖 Documentation

### Essential Files
- **[`context_helper.txt`](context_helper.txt)** - Complete tool documentation for AI agents (add to Claude.md/AGENTS.md)
- **[`CLAUDE.md`](CLAUDE.md)** - Claude Code specific instructions and patterns

### Tool Documentation
- **[Hybrid Biased Reasoning](docs/hybrid-biased-reasoning.md)** - Detailed guide for bias detection with file context
- **[AGENTS.md](AGENTS.md)** - Comprehensive agent configuration guide including ConPort integration

### Technical Documentation
- [API Reference](API_REFERENCE.md) - Complete tool documentation
- [Configuration Guide](CONFIGURATION.md) - Detailed setup instructions
- [Design Document](DESIGN.md) - Architecture and design decisions
- [Troubleshooting](TROUBLESHOOTING.md) - Common issues and solutions

## 🧪 Development Tools

### Quick Commands
```bash
# Run all quality checks
make check
# or
./check.sh

# Auto-fix issues
make fix

# Run CI checks
make ci

# Show configuration
make config
```

### Available Make Commands
```bash
make build    # Build debug version
make release  # Build release version
make check    # Run quality checks (fmt, clippy, test)
make fmt      # Format code
make clippy   # Run clippy lints
make test     # Run tests
make clean    # Clean build artifacts
make run      # Build and run server
make install  # Install to ~/.cargo/bin
```

### Testing
```bash
# Run unit tests
cargo test

# Run integration tests
./test_threading_complete.sh

# Test specific tool
echo '{"jsonrpc":"2.0","method":"tools/list","id":1}' | nc localhost 3333
```

## 🤝 Contributing

Contributions are welcome! Please read our [Contributing Guide](CONTRIBUTING.md) for details.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [Model Context Protocol](https://modelcontextprotocol.io/) by Anthropic
- [zen-mcp](https://github.com/theluk/zen-mcp) for threading inspiration
- OpenAI for O3/O4 reasoning models
- The Rust community for excellent libraries

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/yourusername/lux-mcp/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/lux-mcp/discussions)
- **Email**: support@example.com

---

**Lux MCP** - Illuminating the path to better AI reasoning 🔦