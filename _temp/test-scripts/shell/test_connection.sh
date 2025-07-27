#!/bin/bash

echo "=== Testing Lux MCP Connection ==="
echo

# Check environment variables
echo "Checking API keys..."
if [ -n "$OPENAI_API_KEY" ]; then
    echo "✓ OPENAI_API_KEY is set"
else
    echo "✗ OPENAI_API_KEY is not set"
fi

if [ -n "$OPENROUTER_API_KEY" ]; then
    echo "✓ OPENROUTER_API_KEY is set"
else
    echo "✗ OPENROUTER_API_KEY is not set"
fi

echo
echo "Checking default models..."
echo "LUX_DEFAULT_CHAT_MODEL: ${LUX_DEFAULT_CHAT_MODEL:-gpt4.1 (default)}"
echo "LUX_DEFAULT_REASONING_MODEL: ${LUX_DEFAULT_REASONING_MODEL:-o3-pro (default)}"
echo "LUX_DEFAULT_BIAS_CHECKER_MODEL: ${LUX_DEFAULT_BIAS_CHECKER_MODEL:-o4-mini (default)}"

echo
echo "Testing with different models..."
echo

# Test with OpenRouter model
echo "1. Testing with OpenRouter model (claude)..."
echo '{"jsonrpc": "2.0", "method": "initialize", "params": {"protocolVersion": "0.1.0", "capabilities": {"roots": {"listChanged": true}}}, "id": 1}
{"jsonrpc": "2.0", "method": "tools/call", "params": {"name": "confer", "arguments": {"message": "Hello, testing connection", "model": "claude"}}, "id": 2}' | ./target/release/lux-mcp 2>/tmp/lux-debug.log | grep -A 10 "result"

if [ $? -eq 0 ]; then
    echo "✓ OpenRouter model (claude) works!"
else
    echo "✗ OpenRouter model (claude) failed. Check /tmp/lux-debug.log"
fi

echo
echo "2. Testing with OpenAI model (gpt4.1)..."
echo '{"jsonrpc": "2.0", "method": "initialize", "params": {"protocolVersion": "0.1.0", "capabilities": {"roots": {"listChanged": true}}}, "id": 1}
{"jsonrpc": "2.0", "method": "tools/call", "params": {"name": "confer", "arguments": {"message": "Hello, testing connection", "model": "gpt4.1"}}, "id": 2}' | ./target/release/lux-mcp 2>/tmp/lux-debug2.log | grep -A 10 "result"

if [ $? -eq 0 ]; then
    echo "✓ OpenAI model (gpt4.1) works!"
else
    echo "✗ OpenAI model (gpt4.1) failed. Check /tmp/lux-debug2.log"
fi

echo
echo "Debug logs available at:"
echo "- /tmp/lux-debug.log (OpenRouter test)"
echo "- /tmp/lux-debug2.log (OpenAI test)"