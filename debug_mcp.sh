#!/bin/bash

echo "🔍 Debugging Lux MCP Connection"
echo "==============================="
echo

# Test 1: Binary exists and is executable
echo "1. Checking binary..."
if [ -x "./target/release/lux-mcp" ]; then
    echo "✓ Binary exists and is executable"
else
    echo "✗ Binary not found or not executable"
    exit 1
fi

# Test 2: Environment variables
echo
echo "2. Checking environment variables..."
if [ ! -z "$OPENAI_API_KEY" ] || [ ! -z "$OPENROUTER_API_KEY" ]; then
    echo "✓ API keys are set"
else
    echo "⚠️  No API keys in environment"
fi

# Test 3: MCP Protocol test
echo
echo "3. Testing MCP protocol..."
RESPONSE=$(echo '{"jsonrpc":"2.0","method":"initialize","params":{"capabilities":{}},"id":1}' | ./target/release/lux-mcp 2>&1)
if echo "$RESPONSE" | grep -q '"capabilities"'; then
    echo "✓ Server responds to MCP protocol"
    echo "   Available tools:"
    echo "$RESPONSE" | jq -r '.result.capabilities.tools[]' 2>/dev/null | sed 's/^/   - /'
else
    echo "✗ Server did not respond properly"
    echo "Response: $RESPONSE"
fi

# Test 4: Simple tool call
echo
echo "4. Testing tool call..."
TEST_REQUEST='{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "lux:chat",
    "arguments": {
      "message": "Hello, testing connection"
    }
  },
  "id": "test"
}'

RESPONSE=$(echo "$TEST_REQUEST" | OPENAI_API_KEY="${OPENAI_API_KEY:-test}" OPENROUTER_API_KEY="${OPENROUTER_API_KEY:-test}" ./target/release/lux-mcp 2>&1 | head -100)
if echo "$RESPONSE" | grep -q '"result"'; then
    echo "✓ Tool call successful"
else
    echo "⚠️  Tool call may have failed (this is normal if API keys are invalid)"
fi

# Test 5: Check for common issues
echo
echo "5. Common issues check..."

# Check if another process is using the binary
if lsof ./target/release/lux-mcp >/dev/null 2>&1; then
    echo "⚠️  Another process may be using the binary"
else
    echo "✓ No other process using the binary"
fi

# Check file permissions in detail
echo
echo "6. File permissions:"
ls -la ./target/release/lux-mcp

echo
echo "Debug complete!"
echo
echo "If the MCP client still shows 'connecting...', try:"
echo "1. Restart the MCP client/Claude Desktop"
echo "2. Check the client's console/logs for errors"
echo "3. Ensure the full path in config is correct"
echo "4. Make sure no firewall is blocking local connections"