#!/bin/bash

# Source environment
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
fi

echo "Minimal test to check model display..."

# Create a simple test that should show the formatted output
cat << 'EOF' | RUST_LOG=debug ./target/release/lux-mcp 2>&1 | grep -A 50 "formatted_response ="
{"jsonrpc":"2.0","method":"initialize","params":{"capabilities":{}},"id":1}
{"jsonrpc":"2.0","method":"tools/call","params":{"name":"traced_reasoning","arguments":{"thought":"Test","thought_number":1,"total_thoughts":2,"next_thought_needed":true,"model":"gpt-4o"}},"id":2}
EOF