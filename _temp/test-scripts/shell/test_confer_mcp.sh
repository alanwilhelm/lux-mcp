#!/bin/bash

# Source environment
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
fi

echo "Testing confer with proper MCP protocol..."
echo "=========================================="
echo

# Test 1: With explicit fast model
echo "Test 1: gpt-4o (fast model)"
cat > /tmp/mcp_test1.jsonl << 'EOF'
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"0.1.0","capabilities":{"tools":{}}}}
{"jsonrpc":"2.0","method":"notifications/initialized"}
{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"confer","arguments":{"message":"What is 2+2?","model":"gpt-4o"}}}
EOF

./target/release/lux-mcp < /tmp/mcp_test1.jsonl 2>&1 | grep -v "^warning:" | head -50

echo -e "\n\n=================================="
echo "Test 2: Default model (o3-pro)"
echo "This will take 30 seconds to 5 minutes..."
cat > /tmp/mcp_test2.jsonl << 'EOF'
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"0.1.0","capabilities":{"tools":{}}}}
{"jsonrpc":"2.0","method":"notifications/initialized"}
{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"confer","arguments":{"message":"What is 2+2?"}}}
EOF

# Run with timeout in case it takes too long
timeout 60 ./target/release/lux-mcp < /tmp/mcp_test2.jsonl 2>&1 | grep -v "^warning:" | head -50

# Clean up
rm -f /tmp/mcp_test1.jsonl /tmp/mcp_test2.jsonl

echo -e "\n\nTest complete!"