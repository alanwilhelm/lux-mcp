#!/bin/bash

# Source environment
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
fi

echo "Testing model display in traced_reasoning..."
echo "==========================================="

# Run with initialization and tool call
{
    echo '{"jsonrpc":"2.0","method":"initialize","params":{"capabilities":{}},"id":1}'
    echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"traced_reasoning","arguments":{"thought":"What is consciousness?","thought_number":1,"total_thoughts":3,"next_thought_needed":true,"model":"gpt-4o"}},"id":2}'
} | RUST_LOG=info ./target/release/lux-mcp 2>&1 | grep -A 30 "REASONING THOUGHT"