#!/bin/bash

echo "Simple test of biased_reasoning..."

# Use printf to keep the server alive
{
    printf '{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "test-client", "version": "1.0.0"}}}\n'
    sleep 0.1
    printf '{"jsonrpc": "2.0", "method": "notifications/initialized"}\n'
    sleep 0.1
    printf '{"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {"name": "biased_reasoning", "arguments": {"query": "test simple query"}}}\n'
    sleep 5  # Wait for response
} | ./target/release/lux-mcp 2>/dev/null | tee test_output.json

echo -e "\n\nParsing output..."
cat test_output.json | jq '.'