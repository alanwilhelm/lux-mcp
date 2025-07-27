#!/bin/bash

echo "=== Debugging Lux MCP in Claude Code ==="
echo
echo "1. Checking if server starts properly..."

# Test basic server startup
timeout 5 ./target/release/lux-mcp <<EOF 2>&1 | tee debug_startup.log
{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{}},"id":1}
EOF

echo
echo "2. Checking server response..."
if grep -q "result" debug_startup.log; then
    echo "✓ Server responds to initialize"
else
    echo "✗ Server failed to initialize"
    echo "Check debug_startup.log for errors"
fi

echo
echo "3. Testing with environment from your config..."
export OPENAI_API_KEY="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"
export OPENROUTER_API_KEY="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"
export LUX_DEFAULT_CHAT_MODEL="o3-pro"
export LUX_DEFAULT_REASONING_MODEL="o3-pro"
export LUX_DEFAULT_BIAS_CHECKER_MODEL="o4-mini"
export RUST_LOG="debug"

echo "Testing with your exact configuration..."
./test_rmcp_correct.py 2>&1 | tee debug_full_test.log

echo
echo "4. Analyzing results..."
echo

# Check for specific errors
if grep -q "invalid_api_key" debug_full_test.log; then
    echo "❌ API Key Issue: The OpenAI API key is invalid or expired"
fi

if grep -q "model_not_found" debug_full_test.log; then
    echo "❌ Model Issue: The requested model doesn't exist"
    echo "   o3-pro and o4-mini are not real OpenAI models!"
fi

if grep -q "Chat error: Failed to complete chat request" debug_full_test.log; then
    echo "❌ Chat Tool Failed - checking why..."
    grep -A5 -B5 "Chat error" debug_full_test.log
fi

echo
echo "5. Testing with VALID OpenAI models..."
export LUX_DEFAULT_CHAT_MODEL="gpt-4-turbo-preview"
export LUX_DEFAULT_REASONING_MODEL="gpt-4-turbo-preview"
export LUX_DEFAULT_BIAS_CHECKER_MODEL="gpt-4o-mini"

echo "Testing with real models (gpt-4-turbo-preview, gpt-4o-mini)..."
./test_rmcp_correct.py 2>&1 | tee debug_valid_models.log

if grep -q '"text":' debug_valid_models.log; then
    echo "✅ SUCCESS with valid models!"
    echo "   The issue is that o3-pro and o4-mini don't exist"
else
    echo "❌ Still failing with valid models"
    echo "   There's a deeper issue"
fi

echo
echo "=== SUMMARY ==="
echo "The most likely issues are:"
echo "1. o3-pro and o4-mini are not real OpenAI models"
echo "2. You need to use actual model names like:"
echo "   - gpt-4-turbo-preview"
echo "   - gpt-4o-mini"
echo "   - Or OpenRouter models like 'claude' or 'gemini'"
echo
echo "Update your Claude Code config to use real models!"