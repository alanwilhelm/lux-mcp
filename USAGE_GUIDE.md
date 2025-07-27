# Lux MCP Usage Guide

## Important: How to Use Lux MCP

### With Claude Desktop (Recommended)

Lux MCP is designed to be used as an MCP server with Claude Desktop. It's not meant to be run directly from the command line.

#### Setup:
1. Build the server: `cargo build --release`
2. Configure Claude Desktop (`~/Library/Application Support/Claude/claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "lux": {
      "command": "/path/to/lux-mcp/target/release/lux-mcp",
      "env": {
        "OPENAI_API_KEY": "your-key",
        "OPENROUTER_API_KEY": "your-key",
        "LUX_DEFAULT_CHAT_MODEL": "o3-pro",
        "LUX_DEFAULT_REASONING_MODEL": "o3-pro",
        "LUX_DEFAULT_BIAS_CHECKER_MODEL": "o4-mini",
        "RUST_LOG": "info"
      }
    }
  }
}
```

3. Restart Claude Desktop
4. Use the tools in Claude:
   - `⏺ lux - confer (MCP)(message: "your question")`
   - `⏺ lux - traced_reasoning (MCP)(thought: "complex problem")`
   - `⏺ lux - biased_reasoning (MCP)(query: "analyze this")`
   - `⏺ lux - planner (MCP)(step: "plan this")`

### Direct Testing (Advanced)

The MCP server uses a specific protocol and cannot be tested with simple echo commands. The test scripts in this repository demonstrate the protocol but are primarily for debugging.

### Viewing Logs

When using with Claude Desktop:
```bash
# macOS
tail -f ~/Library/Logs/Claude/mcp-*.log
```

### Configuration

With your current setup using o3-pro for everything:
- **Response times**: 30 seconds to 5 minutes per request
- **Max tokens**: 32,768 for o3 models
- **Reasoning effort**: "high" (automatic for o3 models)
- **Timeout**: 5 minutes

### Tips for o3-pro Usage

1. **Be patient**: o3-pro takes time to think deeply
2. **Use for complex tasks**: Best for philosophical questions, complex analysis, deep reasoning
3. **Consider costs**: o3-pro is expensive - use wisely
4. **Override when needed**: Use faster models explicitly:
   ```
   ⏺ lux - confer (MCP)(message: "quick question", model: "gpt-4o")
   ```

### Troubleshooting

If tools fail in Claude:
1. Check logs: `~/Library/Logs/Claude/mcp-*.log`
2. Verify API keys are set correctly
3. Ensure the server binary path is correct
4. Try with a faster model first to test connectivity