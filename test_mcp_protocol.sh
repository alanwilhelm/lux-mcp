#!/bin/bash

echo "Testing MCP Protocol Compliance"
echo "==============================="
echo

# Test 1: Initialize
echo "1. Testing initialize..."
INIT_RESPONSE=$(echo '{"jsonrpc":"2.0","method":"initialize","params":{"capabilities":{}},"id":1}' | OPENAI_API_KEY="test" OPENROUTER_API_KEY="test" ./target/release/lux-mcp 2>/dev/null)
echo "Response: $INIT_RESPONSE"
echo

# Test 2: List tools
echo "2. Testing tools/list..."
TOOLS_RESPONSE=$(echo '{"jsonrpc":"2.0","method":"tools/list","params":{},"id":2}' | OPENAI_API_KEY="test" OPENROUTER_API_KEY="test" ./target/release/lux-mcp 2>/dev/null)
echo "Response: $TOOLS_RESPONSE"
echo

# Test 3: Error handling
echo "3. Testing error handling..."
ERROR_RESPONSE=$(echo '{"jsonrpc":"2.0","method":"invalid/method","params":{},"id":3}' | OPENAI_API_KEY="test" OPENROUTER_API_KEY="test" ./target/release/lux-mcp 2>/dev/null)
echo "Response: $ERROR_RESPONSE"
echo

# Test 4: Multiple requests
echo "4. Testing multiple requests..."
{
    echo '{"jsonrpc":"2.0","method":"initialize","params":{"capabilities":{}},"id":1}'
    echo '{"jsonrpc":"2.0","method":"tools/list","params":{},"id":2}'
} | OPENAI_API_KEY="test" OPENROUTER_API_KEY="test" ./target/release/lux-mcp 2>/dev/null
echo

echo "5. Testing with real API keys from config..."
{
    echo '{"jsonrpc":"2.0","method":"initialize","params":{"capabilities":{}},"id":1}'
} | OPENAI_API_KEY="$OPENAI_API_KEY" OPENROUTER_API_KEY="$OPENROUTER_API_KEY" ./target/release/lux-mcp 2>&1 | head -20