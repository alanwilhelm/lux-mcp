#!/bin/bash

# Test the plan tool

echo "=== Testing Lux MCP Plan Tool ==="
echo

# Build if needed
if [ ! -f "./target/release/lux-mcp" ]; then
    echo "Building..."
    cargo build --release
fi

# Test plan tool
echo "Testing plan tool..."
echo '{"jsonrpc": "2.0", "method": "initialize", "params": {"protocolVersion": "0.1.0", "capabilities": {"roots": {"listChanged": true}}}, "id": 1}
{"jsonrpc": "2.0", "method": "tools/call", "params": {"name": "plan", "arguments": {"goal": "Build a web application with user authentication"}}, "id": 2}' | ./target/release/lux-mcp 2>&1 | grep -A 20 '"result"'

echo
echo "Test complete!"