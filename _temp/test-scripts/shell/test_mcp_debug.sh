#!/bin/bash

# Test MCP server with debug logging
echo "Testing MCP server with debug logging..."
echo "======================================="

# Export environment variables
export OPENAI_API_KEY="${OPENAI_API_KEY}"
export LUX_DEFAULT_CHAT_MODEL="gpt-4-turbo-preview"
export LUX_DEFAULT_REASONING_MODEL="o3-pro"
export LUX_DEFAULT_BIAS_CHECKER_MODEL="o4-mini"
export RUST_LOG=debug

echo "Configuration:"
echo "  OPENAI_API_KEY: ${OPENAI_API_KEY:0:10}..."
echo "  Default chat model: $LUX_DEFAULT_CHAT_MODEL"
echo "  Default reasoning model: $LUX_DEFAULT_REASONING_MODEL"
echo "  Default bias checker model: $LUX_DEFAULT_BIAS_CHECKER_MODEL"
echo "  Log level: $RUST_LOG"
echo

# Test the confer tool
echo "Testing confer tool..."
echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"confer","arguments":{"message":"Hello, can you hear me?"}},"id":1}' | \
    RUST_LOG=debug ./target/release/lux-mcp 2>&1 | \
    tee mcp_debug.log

echo
echo "Debug output saved to mcp_debug.log"
echo "Look for error messages in the log to diagnose the issue."