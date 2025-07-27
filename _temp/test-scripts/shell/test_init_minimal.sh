#!/bin/bash

echo "Testing minimal initialization..."

# Try the minimal format used in test_mcp_protocol.sh
echo '{"jsonrpc":"2.0","method":"initialize","params":{"capabilities":{}},"id":1}' | \
    ./target/release/lux-mcp 2>&1 | head -20