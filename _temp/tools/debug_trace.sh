#!/bin/bash

# Source environment
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
fi

echo "Debugging traced_reasoning..."

# Run with initialization and tool call, capture full output
{
    echo '{"jsonrpc":"2.0","method":"initialize","params":{"capabilities":{}},"id":1}'
    sleep 0.5
    echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"traced_reasoning","arguments":{"thought":"What is consciousness?","thought_number":1,"total_thoughts":3,"next_thought_needed":true,"model":"gpt-4o"}},"id":2}'
} | RUST_LOG=info ./target/release/lux-mcp 2>&1 | tee debug_output.txt

echo -e "\n\nSearching for Model in output..."
grep -i "model" debug_output.txt