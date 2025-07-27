# Quick Fix for Lux MCP Connection Issues

## The Problem
The current Lux MCP implementation manually handles the JSON-RPC protocol, which is missing proper MCP protocol compliance. The Nirvana MCP uses the `rmcp` crate (Rust MCP) which properly implements the MCP protocol.

## Immediate Solutions

### Option 1: Use the Wrapper Script (Quick Fix)
Update your config to use the wrapper script which handles buffering:

```json
"lux": {
  "command": "/Users/alan/Projects/_MCP/nirvana/lux-mcp/lux-mcp-wrapper.sh",
  "env": {
    "OPENROUTER_API_KEY": "your-key",
    "OPENAI_API_KEY": "your-key",
    "LUX_DEFAULT_CHAT_MODEL": "gpt4.1",
    "LUX_DEFAULT_REASONING_MODEL": "o3",
    "LUX_DEFAULT_BIAS_CHECKER_MODEL": "o4-mini"
  }
}
```

### Option 2: Test with Direct Protocol
Try connecting with this test command to verify the server works:

```bash
# Test the server directly
echo '{"jsonrpc":"2.0","method":"initialize","params":{"capabilities":{}},"id":1}' | ./target/release/lux-mcp

# Test with wrapper
echo '{"jsonrpc":"2.0","method":"initialize","params":{"capabilities":{}},"id":1}' | ./lux-mcp-wrapper.sh
```

## The Real Solution
The proper fix would be to refactor Lux MCP to use the `rmcp` crate like Nirvana MCP does:

1. Add `rmcp` to dependencies in Cargo.toml
2. Refactor the server to use `rmcp::ServiceExt` 
3. Implement proper MCP handlers using rmcp's macros

This is what makes Nirvana MCP work reliably - it uses a proper MCP implementation library instead of manually handling the protocol.

## Current Status
- ✅ Server responds to JSON-RPC
- ✅ Tools are implemented
- ✅ Added "initialized" handler
- ❌ Missing proper MCP protocol compliance
- ❌ May have buffering issues

## About rmcp
The `rmcp` crate appears to be a Rust implementation of the Model Context Protocol. While not officially from Anthropic, it provides proper MCP protocol handling that our manual implementation lacks.