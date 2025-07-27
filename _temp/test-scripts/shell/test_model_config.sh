#!/bin/bash

# Test script to verify model configuration

echo "=== Lux MCP Model Configuration Test ==="
echo

# Check if binary exists
if [ ! -f "./target/release/lux-mcp" ]; then
    echo "ERROR: Binary not found. Run 'cargo build --release' first."
    exit 1
fi

# Function to test a specific configuration
test_config() {
    local name=$1
    local openai_key=$2
    local openrouter_key=$3
    local chat_model=$4
    local reasoning_model=$5
    local bias_model=$6
    
    echo "Testing: $name"
    echo "  Chat model: ${chat_model:-default}"
    echo "  Reasoning model: ${reasoning_model:-default}"
    echo "  Bias checker: ${bias_model:-default}"
    
    # Set up environment
    export OPENAI_API_KEY="$openai_key"
    export OPENROUTER_API_KEY="$openrouter_key"
    [ -n "$chat_model" ] && export LUX_DEFAULT_CHAT_MODEL="$chat_model"
    [ -n "$reasoning_model" ] && export LUX_DEFAULT_REASONING_MODEL="$reasoning_model"
    [ -n "$bias_model" ] && export LUX_DEFAULT_BIAS_CHECKER_MODEL="$bias_model"
    
    # Run server and check startup
    timeout 3s ./target/release/lux-mcp 2>&1 | grep -E "(API Configuration|Available|Not found)" | head -5
    
    echo
}

# Test 1: OpenAI only with defaults
test_config "OpenAI with defaults" \
    "test-key" \
    "" \
    "" \
    "" \
    ""

# Test 2: OpenRouter only with Claude
test_config "OpenRouter with Claude models" \
    "" \
    "test-key" \
    "claude" \
    "opus" \
    "sonnet"

# Test 3: Mixed providers
test_config "Mixed providers" \
    "test-key" \
    "test-key" \
    "gpt4.1" \
    "claude" \
    "flash"

# Test 4: Custom models
test_config "Custom model names" \
    "test-key" \
    "" \
    "4" \
    "o3-pro" \
    "mini"

echo "=== Configuration test complete ==="
echo
echo "To test with real API keys:"
echo "  1. Set OPENAI_API_KEY and/or OPENROUTER_API_KEY in .env"
echo "  2. Run: ./test_chat.sh or ./test_traced_reasoning.sh"