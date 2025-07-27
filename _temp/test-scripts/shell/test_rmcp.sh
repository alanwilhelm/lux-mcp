#!/bin/bash

echo "Testing Lux MCP with rmcp..."
echo "============================="
echo

# Test initialization
echo "1. Testing initialization..."
echo '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"0.1.0","capabilities":{}},"id":1}' | ./target/release/lux-mcp 2>&1 | jq .

echo
echo "2. Testing with full handshake..."
{
    echo '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"0.1.0","capabilities":{}},"id":1}'
    echo '{"jsonrpc":"2.0","method":"initialized","params":{},"id":2}'
    echo '{"jsonrpc":"2.0","method":"tools/list","params":{},"id":3}'
} | ./target/release/lux-mcp 2>&1 | jq .

echo
echo "Test complete!"