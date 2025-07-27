#!/bin/bash

# Source environment
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
fi

echo "Testing confer with error diagnostics..."
echo "======================================="
echo

# Enable info logging to see what's happening
export RUST_LOG=info

# Test with a simpler request first to ensure the tool works
echo "Test 1: Simple request"
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"confer","arguments":{"message":"What is 2+2?"}}}' | ./target/release/lux-mcp 2>&1 | tail -50

echo -e "\n\n=============================================="
echo "Test 2: Complex request (like user's) with explicit model"
echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"confer","arguments":{"message":"Analyze a simple file. Just tell me what language it is written in.","model":"gpt-4"}}}' | ./target/release/lux-mcp 2>&1 | tail -50