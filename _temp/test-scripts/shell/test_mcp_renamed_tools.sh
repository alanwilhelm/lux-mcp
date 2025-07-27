#!/bin/bash

echo "Testing Lux MCP with Renamed Tools"
echo "==================================="

# Build if needed
if [ ! -f target/release/lux-mcp ]; then
    echo "Building Lux MCP server..."
    cargo build --release || exit 1
fi

# Create a named pipe for bidirectional communication
PIPE_IN=$(mktemp -u)
PIPE_OUT=$(mktemp -u)
mkfifo "$PIPE_IN" "$PIPE_OUT"

# Start the server in background
./target/release/lux-mcp < "$PIPE_IN" > "$PIPE_OUT" 2>server.log &
SERVER_PID=$!

# Function to send request and get response
send_request() {
    echo "$1" > "$PIPE_IN"
    timeout 2 head -n 1 "$PIPE_OUT"
}

# Clean up on exit
cleanup() {
    kill $SERVER_PID 2>/dev/null
    rm -f "$PIPE_IN" "$PIPE_OUT"
}
trap cleanup EXIT

echo "1. Testing initialize..."
INIT_RESPONSE=$(send_request '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"0.1.0","capabilities":{"tools":{}}},"id":1}')
echo "$INIT_RESPONSE" | jq .

echo -e "\n2. Testing tools/list..."
TOOLS_RESPONSE=$(send_request '{"jsonrpc":"2.0","method":"tools/list","id":2}')
echo "$TOOLS_RESPONSE" | jq .

echo -e "\nTool names found:"
echo "$TOOLS_RESPONSE" | jq -r '.result.tools[].name'

echo -e "\n3. Testing confer tool..."
CONFER_RESPONSE=$(send_request '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"confer","arguments":{"message":"Hello, test"}},"id":3}')
echo "$CONFER_RESPONSE" | jq .

echo -e "\n4. Testing traced_reasoning tool..."
TRACED_RESPONSE=$(send_request '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"traced_reasoning","arguments":{"query":"What is 2+2?","max_steps":3}},"id":4}')
echo "$TRACED_RESPONSE" | jq .

echo -e "\n5. Testing biased_reasoning tool..."
BIASED_RESPONSE=$(send_request '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"biased_reasoning","arguments":{"query":"Is the sky blue?","max_steps":2}},"id":5}')
echo "$BIASED_RESPONSE" | jq .

echo -e "\nServer log:"
cat server.log

cleanup