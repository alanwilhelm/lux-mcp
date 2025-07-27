#!/bin/bash

# Proper MCP protocol test with initialization
echo "Testing MCP server with proper protocol flow..."
echo "============================================="

# Export environment variables
export OPENAI_API_KEY="${OPENAI_API_KEY}"
export LUX_DEFAULT_CHAT_MODEL="gpt-4-turbo-preview"
export LUX_DEFAULT_REASONING_MODEL="o3-pro"
export LUX_DEFAULT_BIAS_CHECKER_MODEL="o4-mini"
export RUST_LOG=debug

# Create a test script that sends proper MCP messages
cat > test_messages.txt << 'EOF'
{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"1.0","clientInfo":{"name":"test-client","version":"1.0"}},"id":1}
{"jsonrpc":"2.0","method":"tools/call","params":{"name":"confer","arguments":{"message":"Hello, can you hear me?"}},"id":2}
EOF

echo "Sending initialization and tool call..."
cat test_messages.txt | ./target/release/lux-mcp 2>mcp_proper.log

echo
echo "Logs saved to mcp_proper.log"
echo "Check the log for detailed debug output."