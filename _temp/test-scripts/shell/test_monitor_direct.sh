#!/bin/bash

# Test Monitor Integration - Direct test

echo "=== Testing MetacognitiveMonitor Integration ==="
echo

# Build if needed
if [ ! -f target/release/lux-mcp ]; then
    echo "Building Lux MCP..."
    cargo build --release || exit 1
fi

# Source environment
export $(cat .env | grep -v '^#' | xargs)

echo "Testing circular reasoning detection..."
echo

# Send the JSON-RPC request directly
echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"traced_reasoning","arguments":{"query":"Understanding recursion requires understanding recursion. To understand recursion, you must understand recursion.","max_steps":3,"guardrails":{"circular_reasoning_detection":true}}},"id":1}' | ./target/release/lux-mcp

echo -e "\n=== Test Complete ==="
echo "Look for 'CircularReasoning' intervention in the output above."