#!/bin/bash

# Test Monitor Integration - Simple test

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

# Create a test file with proper JSON-RPC messages
cat > /tmp/test_monitor.json <<'EOF'
{"jsonrpc": "2.0", "method": "initialize", "params": {"capabilities": {}}, "id": 1}
{"jsonrpc": "2.0", "method": "tools/call", "params": {"name": "traced_reasoning", "arguments": {"query": "Understanding recursion requires understanding recursion. To understand recursion, you must understand recursion.", "max_steps": 3, "guardrails": {"circular_reasoning_detection": true}}}, "id": 2}
EOF

# Run the test
./target/release/lux-mcp < /tmp/test_monitor.json

# Clean up
rm -f /tmp/test_monitor.json

echo -e "\n=== Test Complete ==="
echo "Look for intervention messages in the output above."