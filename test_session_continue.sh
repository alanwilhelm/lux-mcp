#!/bin/bash

echo "Testing session continuation..."

SESSION_ID="bias_660ab916616e564e"

# Test continuing the session
{
    printf '{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "test-client", "version": "1.0.0"}}}\n'
    sleep 0.1
    printf '{"jsonrpc": "2.0", "method": "notifications/initialized"}\n'
    sleep 0.1
    printf '{"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {"name": "biased_reasoning", "arguments": {"query": "test simple query", "session_id": "'"$SESSION_ID"'"}}}\n'
    sleep 5  # Wait for response
} | ./target/release/lux-mcp 2>/dev/null | tee test_continue.json

echo -e "\n\nParsing second call output..."
cat test_continue.json | grep '"id":2' -A20 | jq '.result.content[0].text'