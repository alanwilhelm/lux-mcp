#!/bin/bash

# Source environment
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
fi

echo "Testing o3-pro with 5 minute timeout..."
echo "======================================="
echo "This may take up to 5 minutes to complete or timeout."
echo

# Enable info logging
export RUST_LOG=info

# Test with o3-pro
time echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"confer","arguments":{"message":"What is 2+2?","model":"o3-pro"}}}' | ./target/release/lux-mcp 2>&1 | grep -E "(result|error|Error|timeout|Timeout)" | tail -20

echo -e "\n\nTest complete. If it worked, you should see a result."
echo "If it timed out after 5 minutes, you'll see a timeout error."