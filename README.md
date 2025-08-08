# Lux MCP - Metacognitive Model Context Protocol Server

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![MCP](https://img.shields.io/badge/MCP-1.0-blue?style=for-the-badge)](https://modelcontextprotocol.io/)
[![License](https://img.shields.io/badge/license-MIT-green?style=for-the-badge)](LICENSE)

Lux MCP is a Model Context Protocol server that "illuminates" AI reasoning by providing metacognitive monitoring, conversation threading, and quality tracking for LLM interactions.

## 🌟 Key Features

- **🧠 Metacognitive Monitoring** - Detects and prevents circular reasoning, distractor fixation, and quality degradation
- **🧵 Conversation Threading** - Maintains context across tool calls with Zen-style threading
- **🔍 Bias Detection** - Dual-model reasoning with step-by-step bias analysis
- **📊 Quality Metrics** - Tracks confidence, clarity, and coherence with trend analysis
- **💾 Hybrid Storage** - In-memory performance with optional database persistence
- **🚀 O3/O4 Support** - Full support for OpenAI's latest reasoning models

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

# Optional defaults
export LUX_DEFAULT_CHAT_MODEL="gpt-4o"
export LUX_DEFAULT_REASONING_MODEL="o3-pro"
export LUX_DEFAULT_BIAS_CHECKER_MODEL="o4-mini"

# Optional database
export DATABASE_URL="postgresql://user:pass@localhost/lux_mcp"

# Logging
export RUST_LOG="info"
```

## 🚀 Quick Start

### Claude Desktop Configuration
Add to your Claude Desktop config (`~/Library/Application Support/Claude/claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "lux": {
      "command": "/path/to/lux-mcp/target/release/lux-mcp",
      "env": {
        "OPENAI_API_KEY": "your-key",
        "LUX_DEFAULT_REASONING_MODEL": "o3-pro",
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

## 🛠️ Available Tools

### `confer` - Conversational AI
Simple chat with model selection and threading support.
```json
{
  "tool": "confer",
  "arguments": {
    "message": "Your message here",
    "model": "gpt-4o",
    "continuation_id": "thread-123"
  }
}
```

### `traced_reasoning` - Step-by-Step Reasoning
Metacognitive reasoning with monitoring and synthesis.
```json
{
  "tool": "traced_reasoning",
  "arguments": {
    "thought": "How can we optimize database queries?",
    "thought_number": 1,
    "total_thoughts": 5,
    "next_thought_needed": true
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

### `illumination_status` - System Status
Check metacognitive monitoring status.
```json
{
  "tool": "illumination_status",
  "arguments": {}
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

### OpenAI Models
- **GPT-5**: `gpt-5` (coming August 2025 - pre-configured support)
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
| Variable | Description | Default |
|----------|-------------|---------|
| `OPENAI_API_KEY` | OpenAI API key | Required* |
| `OPENROUTER_API_KEY` | OpenRouter API key | Required* |
| `LUX_DEFAULT_CHAT_MODEL` | Default for confer | `gpt-4o` |
| `LUX_DEFAULT_REASONING_MODEL` | Default for traced_reasoning | `o3-pro` |
| `LUX_DEFAULT_BIAS_CHECKER_MODEL` | Default for biased_reasoning | `o4-mini` |
| `DATABASE_URL` | PostgreSQL connection | Optional |
| `RUST_LOG` | Log level | `info` |

*At least one API key required

### Database Setup (Optional)
```bash
# Install SeaORM CLI
cargo install sea-orm-cli

# Run migrations
DATABASE_URL="postgresql://localhost/lux_mcp" sea-orm-cli migrate up

# Or use the setup script
./setup_database.sh
```

## 📖 Documentation

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