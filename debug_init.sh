#!/bin/bash

echo "Testing MCP initialization sequence..."
echo "======================================"
echo

# Test 1: Basic server response
echo "1. Testing if server responds at all..."
timeout 2 ./target/release/lux-mcp < /dev/null 2>&1
if [ $? -eq 124 ]; then
    echo "✓ Server is waiting for input (good)"
else
    echo "✗ Server exited unexpectedly"
fi
echo

# Test 2: Single initialization
echo "2. Testing single initialization..."
echo '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"0.1.0","capabilities":{"roots":{"listChanged":true},"sampling":{}}},"id":1}' | timeout 2 ./target/release/lux-mcp 2>/dev/null
echo

# Test 3: Full MCP handshake
echo "3. Testing full MCP handshake..."
{
    echo '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"0.1.0","capabilities":{"roots":{"listChanged":true},"sampling":{}}},"id":1}'
    sleep 0.1
    echo '{"jsonrpc":"2.0","method":"initialized","params":{},"id":2}'
    sleep 0.1
    echo '{"jsonrpc":"2.0","method":"tools/list","params":{},"id":3}'
} | timeout 3 ./target/release/lux-mcp 2>&1

echo
echo "4. Checking for hanging processes..."
ps aux | grep lux-mcp | grep -v grep

echo
echo "5. Testing with stdbuf to disable buffering..."
echo '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"0.1.0","capabilities":{}},"id":1}' | stdbuf -o0 -e0 ./target/release/lux-mcp 2>/dev/null