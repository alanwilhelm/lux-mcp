#!/bin/bash

# Source environment
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
fi

echo "Testing biased_reasoning with proper MCP protocol..."
echo "==================================================="

# Create a temporary file with proper MCP messages
cat > /tmp/mcp_test.jsonl << 'EOF'
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"0.1.0","capabilities":{"tools":{}}}}
{"jsonrpc":"2.0","method":"notifications/initialized"}
{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"biased_reasoning","arguments":{"query":"Should we use Redis?","primary_model":"gpt-4","verifier_model":"gpt-4","max_steps":1}}}
EOF

# Run the test
./target/release/lux-mcp < /tmp/mcp_test.jsonl 2>&1 | grep -v "^warning:" | head -200

# Clean up
rm -f /tmp/mcp_test.jsonl

echo -e "\n\nTest complete!"