#!/bin/bash

# Test script for conversation threading in Lux MCP

echo "=== Testing Lux MCP Conversation Threading ==="
echo

# Start the server in background
echo "Starting Lux MCP server..."
./target/release/lux-mcp 2>/dev/null &
SERVER_PID=$!
sleep 2

# Function to send MCP request
send_request() {
    echo "$1" | nc localhost 3000 2>/dev/null
}

# Test 1: Create a new thread with confer
echo "Test 1: Creating new conversation thread with confer..."
RESPONSE1=$(cat <<EOF | nc localhost 3000 2>/dev/null
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "confer",
    "arguments": {
      "message": "What is the capital of France?"
    }
  },
  "id": 1
}
EOF
)

echo "Response 1:"
echo "$RESPONSE1" | jq -r '.result.content[0].text' 2>/dev/null || echo "$RESPONSE1"
echo

# Extract continuation_id from response (if present)
THREAD_ID=$(echo "$RESPONSE1" | grep -oE 'Continuation ID: [a-f0-9-]{36}' | cut -d' ' -f3)

if [ -n "$THREAD_ID" ]; then
    echo "Thread ID extracted: $THREAD_ID"
    echo
    
    # Test 2: Continue conversation with the same thread
    echo "Test 2: Continuing conversation with thread ID..."
    RESPONSE2=$(cat <<EOF | nc localhost 3000 2>/dev/null
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "confer",
    "arguments": {
      "message": "What is the population of that city?",
      "continuation_id": "$THREAD_ID"
    }
  },
  "id": 2
}
EOF
)
    
    echo "Response 2:"
    echo "$RESPONSE2" | jq -r '.result.content[0].text' 2>/dev/null || echo "$RESPONSE2"
    echo
    
    # Check if context was preserved
    if echo "$RESPONSE2" | grep -qi "paris\|france"; then
        echo "✅ SUCCESS: Context was preserved! The model remembered we were talking about Paris/France."
    else
        echo "⚠️  WARNING: Context might not have been preserved. Check the response manually."
    fi
else
    echo "❌ ERROR: Could not extract thread ID from first response"
fi

# Clean up
echo
echo "Stopping server..."
kill $SERVER_PID 2>/dev/null
wait $SERVER_PID 2>/dev/null

echo "=== Test Complete ==="