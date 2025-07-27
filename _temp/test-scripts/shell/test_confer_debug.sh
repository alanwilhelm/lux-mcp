#!/bin/bash

# Source environment
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
fi

echo "Testing confer (chat) tool with debug output..."
echo "=============================================="
echo

# Enable debug logging
export RUST_LOG=debug

# Show current configuration
echo "Current environment:"
echo "LUX_DEFAULT_CHAT_MODEL: ${LUX_DEFAULT_CHAT_MODEL:-not set}"
echo "OPENAI_API_KEY: ${OPENAI_API_KEY:+[SET]}"
echo "OPENROUTER_API_KEY: ${OPENROUTER_API_KEY:+[SET]}"
echo

# Test with a simple message first
echo "Test 1: Simple message with default model"
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"confer","arguments":{"message":"Hello, can you hear me?"}}}' | cargo run --release 2>&1 | grep -E "(error|Error|failed|Failed|model|Model|key|Key)" | head -20

echo -e "\n\nTest 2: Simple message with explicit OpenAI model"
echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"confer","arguments":{"message":"Hello, can you hear me?","model":"gpt-4"}}}' | cargo run --release 2>&1 | grep -E "(error|Error|failed|Failed|model|Model|key|Key)" | head -20

echo -e "\n\nTest 3: Simple message with o3-pro model"
echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"confer","arguments":{"message":"What is 2+2?","model":"o3-pro"}}}' | cargo run --release 2>&1 | grep -E "(error|Error|failed|Failed|model|Model|Reasoning effort|max_tokens)" | head -20

echo -e "\n\nCheck logs above for error details."