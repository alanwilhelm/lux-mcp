#!/bin/bash

# Source environment
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
fi

echo "Testing o3-pro with maximum reasoning configuration..."
echo "===================================================="
echo "- Model: o3-pro"
echo "- Max tokens: 32,768"
echo "- Reasoning effort: high"
echo "- Timeout: 5 minutes"
echo
echo "This may take 30 seconds to 5 minutes..."
echo

# Enable info logging to see configuration
export RUST_LOG=info

# Test with a complex reasoning task
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"confer","arguments":{"message":"Analyze the philosophical implications of emergent consciousness in artificial neural networks, considering both materialist and dualist perspectives"}}}' | ./target/release/lux-mcp 2>&1 | tee o3_test.log

echo -e "\n\nChecking logs for configuration..."
grep -E "(max_tokens: 32768|reasoning_effort|Model: o3)" o3_test.log | head -10

echo -e "\n\nTest complete. Check o3_test.log for full output."