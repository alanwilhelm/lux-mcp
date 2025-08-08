#!/bin/bash

# Test GPT-5 availability
echo "Testing GPT-5 availability..."

# Start the server in background
OPENAI_API_KEY="${OPENAI_API_KEY}" RUST_LOG=info ./target/release/lux-mcp &
SERVER_PID=$!

# Wait for server to start
sleep 2

# Send test request
cat << 'EOF' | nc localhost 3333
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "confer",
    "arguments": {
      "message": "What model are you? Please state your exact model name.",
      "model": "gpt-5"
    }
  },
  "id": 1
}
EOF

# Kill the server
kill $SERVER_PID 2>/dev/null

echo "Test complete"