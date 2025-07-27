#!/bin/bash

# Source environment
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
fi

echo "Testing o3-pro with proper MCP protocol..."
echo "=========================================="
echo "- Model: o3-pro"
echo "- Max tokens: 32,768"
echo "- Reasoning effort: high"
echo "- Timeout: 5 minutes"
echo

# Enable info logging
export RUST_LOG=info

# Create proper MCP messages
cat > /tmp/mcp_o3_test.jsonl << 'EOF'
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"0.1.0","capabilities":{"tools":{}}}}
{"jsonrpc":"2.0","method":"notifications/initialized"}
{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"confer","arguments":{"message":"What is the meaning of consciousness?"}}}
EOF

echo "Starting test (this may take 30 seconds to 5 minutes)..."
echo "Press Ctrl+C if it takes too long."
echo

# Run the test and save output
./target/release/lux-mcp < /tmp/mcp_o3_test.jsonl > /tmp/o3_output.json 2> /tmp/o3_logs.txt

echo "Response:"
echo "========="
cat /tmp/o3_output.json | python3 -m json.tool 2>/dev/null || cat /tmp/o3_output.json

echo -e "\n\nLog excerpts:"
echo "============="
grep -E "(max_tokens: 32768|reasoning_effort|Model: o3|Sending chat request)" /tmp/o3_logs.txt | head -10

# Clean up
rm -f /tmp/mcp_o3_test.jsonl /tmp/o3_output.json /tmp/o3_logs.txt