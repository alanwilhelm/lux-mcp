#!/bin/bash

echo "=== Testing MetacognitiveMonitor Integration ==="
echo

# Build if needed
if [ ! -f target/release/lux-mcp ]; then
    echo "Building Lux MCP..."
    cargo build --release || exit 1
fi

# Source environment
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
fi

echo "1. Testing circular reasoning detection..."
echo "First initializing, then calling traced_reasoning with circular query..."

# Test with multiple requests in sequence
{
    echo '{"jsonrpc":"2.0","method":"initialize","params":{"capabilities":{}},"id":1}'
    echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"traced_reasoning","arguments":{"query":"Understanding recursion requires understanding recursion. To understand recursion, you must understand recursion. Recursion is when you understand recursion by understanding recursion.","max_steps":5,"guardrails":{"circular_reasoning_detection":true}}},"id":2}'
} | ./target/release/lux-mcp 2>&1 | grep -E "(intervention|CircularReasoning|Intervention|circular_reasoning)" || echo "No circular reasoning detected in output"

echo
echo "2. Testing quality degradation..."
{
    echo '{"jsonrpc":"2.0","method":"initialize","params":{"capabilities":{}},"id":1}'
    echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"traced_reasoning","arguments":{"query":"Start with detailed explanation then degrade","max_steps":5}},"id":2}'
} | ./target/release/lux-mcp 2>&1 | grep -E "(quality|degrading|fatigue)" || echo "No quality metrics in output"

echo -e "\n=== Test Complete ==="
echo "If monitoring is working, you should see intervention messages above."