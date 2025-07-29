#!/bin/bash

echo "Testing full session in one run..."

# Run all commands in one session
{
    printf '{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "test-client", "version": "1.0.0"}}}\n'
    sleep 0.1
    printf '{"jsonrpc": "2.0", "method": "notifications/initialized"}\n'
    sleep 0.1
    
    # First call
    printf '{"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {"name": "biased_reasoning", "arguments": {"query": "test simple query"}}}\n'
    sleep 0.5
    
    # Continue with deterministic session ID (based on query hash)
    printf '{"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {"name": "biased_reasoning", "arguments": {"query": "test simple query"}}}\n'
    sleep 0.5
    
    # Third call
    printf '{"jsonrpc": "2.0", "id": 4, "method": "tools/call", "params": {"name": "biased_reasoning", "arguments": {"query": "test simple query"}}}\n'
    sleep 0.5
    
    # Test new session flag
    printf '{"jsonrpc": "2.0", "id": 5, "method": "tools/call", "params": {"name": "biased_reasoning", "arguments": {"query": "test simple query", "new_session": true}}}\n'
    sleep 0.5
    
} | ./target/release/lux-mcp 2>/dev/null | tee full_session.json

echo -e "\n\nParsing outputs..."
echo -e "\nFirst call (should be step 1):"
cat full_session.json | jq '.[] | select(.id == 2) | .result.content[0].text' | grep -E "(Step|Session ID)" || echo "No match found"

echo -e "\nSecond call (should be step 2):"
cat full_session.json | jq '.[] | select(.id == 3) | .result.content[0].text' | grep -E "(Step|Session ID)" || echo "No match found"

echo -e "\nThird call (should be step 3):"
cat full_session.json | jq '.[] | select(.id == 4) | .result.content[0].text' | grep -E "(Step|Session ID)" || echo "No match found"

echo -e "\nNew session call (should be step 1 with different ID):"
cat full_session.json | jq '.[] | select(.id == 5) | .result.content[0].text' | grep -E "(Step|Session ID)" || echo "No match found"