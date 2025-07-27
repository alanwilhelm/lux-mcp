#!/bin/bash

# Test script to verify action directives in tool responses

echo "Testing lux-mcp action directives..."
echo "==================================="

# Build first
echo "Building project..."
cargo build --release

if [ $? -ne 0 ]; then
    echo "Build failed!"
    exit 1
fi

echo ""
echo "Starting MCP server and testing confer tool..."
echo "---------------------------------------------"

# Create a test interaction
cat > test_confer_request.json << 'EOF'
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {
    "protocolVersion": "2024-11-05",
    "capabilities": {},
    "clientInfo": {
      "name": "test-client",
      "version": "1.0.0"
    }
  }
}
EOF

cat > test_confer_call.json << 'EOF'
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "confer",
    "arguments": {
      "message": "What are the key principles of clean code architecture?",
      "model": "gpt-4o",
      "max_tokens": 150
    }
  }
}
EOF

# Start server and send requests
echo "Sending test requests..."
(
  cat test_confer_request.json
  echo
  sleep 0.5
  echo '{"jsonrpc": "2.0", "method": "notifications/initialized"}'
  echo
  sleep 0.5
  cat test_confer_call.json
  echo
  sleep 3
) | RUST_LOG=info ./target/release/lux-mcp 2>&1 | grep -A 20 "LUX ANALYSIS COMPLETE"

# Clean up
rm -f test_confer_request.json test_confer_call.json

echo ""
echo "Test complete! Check above for action directives in the response."