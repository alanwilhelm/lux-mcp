#!/bin/bash

echo "🧪 Testing biased reasoning with synthesis (with timeout)..."

# Create a named pipe for communication
PIPE_IN=$(mktemp -u)
PIPE_OUT=$(mktemp -u)
mkfifo "$PIPE_IN"
mkfifo "$PIPE_OUT"

# Start the server with input from pipe
RUST_LOG=info DATABASE_URL="postgres://lux_user:lux_password@localhost/lux_mcp" \
    ./target/release/lux-mcp < "$PIPE_IN" > "$PIPE_OUT" 2>/tmp/lux_err.log &
SERVER_PID=$!

# Function to send messages
send_message() {
    echo "$1" > "$PIPE_IN"
}

# Function to read response with timeout
read_response() {
    timeout 10s cat "$PIPE_OUT"
}

# Initialize the server
echo "📝 Initializing server..."
send_message '{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "test", "version": "1.0"}}}'
INIT_RESPONSE=$(read_response | head -1)
echo "Init response: $INIT_RESPONSE"

# Send initialized notification
send_message '{"jsonrpc": "2.0", "method": "notifications/initialized"}'
sleep 0.5

# Test biased reasoning
echo -e "\n🧠 Testing biased reasoning..."
send_message '{"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {"name": "biased_reasoning", "arguments": {"query": "Should we migrate to microservices?"}}}'

# Read response with timeout
echo -e "\n📖 Reading response (with 10s timeout)..."
RESPONSE=$(read_response)
echo "$RESPONSE" | jq -r '.result.content[0].text' 2>/dev/null || echo "$RESPONSE"

# Keep connection open for a bit to see if we get more responses
echo -e "\n⏳ Waiting for additional responses..."
sleep 2

# Clean up
kill $SERVER_PID 2>/dev/null
rm -f "$PIPE_IN" "$PIPE_OUT"

echo -e "\n📋 Server logs:"
cat /tmp/lux_err.log | tail -30

echo -e "\n✅ Test completed!"