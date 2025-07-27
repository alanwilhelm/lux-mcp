#!/bin/bash

echo "Testing direct MCP communication..."

# Start server with Claude as default model
export LUX_DEFAULT_CHAT_MODEL=claude
export LUX_DEFAULT_REASONING_MODEL=gemini
export LUX_DEFAULT_BIAS_CHECKER_MODEL=flash

echo "Starting server with OpenRouter defaults..."
echo

# Create a test input file with proper MCP messages
cat > /tmp/mcp_test_input.json << 'EOF'
{"jsonrpc": "2.0", "method": "initialize", "params": {"protocolVersion": "1.0.0", "capabilities": {"tools": {}}}, "id": 1}
{"jsonrpc": "2.0", "method": "initialized", "params": {}, "id": 2}
{"jsonrpc": "2.0", "method": "tools/list", "params": {}, "id": 3}
{"jsonrpc": "2.0", "method": "tools/call", "params": {"name": "confer", "arguments": {"message": "Hello, can you hear me?"}}, "id": 4}
EOF

# Run the server with input
./target/release/lux-mcp < /tmp/mcp_test_input.json 2>/tmp/lux_test.log

echo
echo "Check /tmp/lux_test.log for debug output"