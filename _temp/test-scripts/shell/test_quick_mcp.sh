#!/bin/bash

# Source environment
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
fi

echo "Quick MCP test with fast model..."
echo "================================="
echo

# Enable info logging
export RUST_LOG=info

# Test with gpt-4o first (fast)
cat > /tmp/mcp_quick_test.jsonl << 'EOF'
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"0.1.0","capabilities":{"tools":{}}}}
{"jsonrpc":"2.0","method":"notifications/initialized"}
{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"confer","arguments":{"message":"What is 2+2?","model":"gpt-4o"}}}
EOF

echo "Testing with gpt-4o (should be fast)..."
time ./target/release/lux-mcp < /tmp/mcp_quick_test.jsonl 2>&1 | grep -E "(result|error)" | tail -20

echo -e "\n\nNow testing with o3-pro (will be slow)..."
cat > /tmp/mcp_o3_test.jsonl << 'EOF'
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"0.1.0","capabilities":{"tools":{}}}}
{"jsonrpc":"2.0","method":"notifications/initialized"}
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"confer","arguments":{"message":"What is 2+2?"}}}
EOF

time ./target/release/lux-mcp < /tmp/mcp_o3_test.jsonl 2>&1 | grep -E "(result|error)" | tail -20

# Clean up
rm -f /tmp/mcp_quick_test.jsonl /tmp/mcp_o3_test.jsonl