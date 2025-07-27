#!/bin/bash

# Source environment
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
fi

# Enable debug logging
export RUST_LOG=info

echo "Testing biased_reasoning tool directly..."
echo "========================================"

# Test with the exact parameters the user reported
cat << 'EOF' | ./target/release/lux-mcp 2>&1
{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"0.1.0","capabilities":{"tools":{}}},"id":1}
{"jsonrpc":"2.0","method":"tools/call","params":{"name":"biased_reasoning","arguments":{"query":"plan a similar system for our stripe payments","primary_model":"gpt-4","verifier_model":"o4-mini","max_steps":2}},"id":2}
EOF