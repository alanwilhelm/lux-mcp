#!/bin/bash

# Source environment
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
fi

MODEL="${1:-gpt-4o}"

echo "Testing confer with explicit model: $MODEL"
echo "=========================================="
echo

# Enable info logging
export RUST_LOG=info

# Test with explicit model
echo "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/call\",\"params\":{\"name\":\"confer\",\"arguments\":{\"message\":\"What is 2+2? Please respond quickly.\",\"model\":\"$MODEL\"}}}" | ./target/release/lux-mcp 2>&1 | grep -v "^warning:" | tail -30

echo -e "\n\nIf this works quickly, the issue is that o3-pro is too slow for chat."
echo "Update your .env file to use a faster model for LUX_DEFAULT_CHAT_MODEL."