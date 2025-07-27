#!/bin/bash

# Source environment
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
fi

echo "Testing biased_reasoning with error logging..."
echo "=============================================="

# Enable debug logging to see error messages
export RUST_LOG=debug

# Create the exact test that the user reported fails
cat << 'EOF' | ./target/release/lux-mcp 2>&1 | tee biased_reasoning_debug.log
{"jsonrpc":"2.0","method":"initialize","params":{"capabilities":{}},"id":1}
{"jsonrpc":"2.0","method":"tools/call","params":{"name":"biased_reasoning","arguments":{"query":"plan a similar system for our stripe payments","primary_model":"gpt-4","verifier_model":"o4-mini","max_steps":3}},"id":2}
EOF

echo -e "\n\nSearching for error messages in logs..."
grep -i "verifier model" biased_reasoning_debug.log || echo "No verifier model error found"
grep -i "failed" biased_reasoning_debug.log | grep -i "bias" || echo "No bias check failure found"