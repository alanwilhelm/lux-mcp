#!/bin/bash

echo "Testing MCP Tool Discovery"
echo "=========================="

# First send initialize request, then tools/list
(
echo '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"0.1.0","capabilities":{"tools":{}}},"id":1}'
sleep 0.1
echo '{"jsonrpc":"2.0","method":"tools/list","id":2}'
) | ./target/release/lux-mcp 2>&1 | grep -A 1000 '"method":"tools/list"' | tail -n +2 | jq .

echo -e "\n\nExtracting tool names:"
(
echo '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"0.1.0","capabilities":{"tools":{}}},"id":1}'
sleep 0.1
echo '{"jsonrpc":"2.0","method":"tools/list","id":2}'
) | ./target/release/lux-mcp 2>&1 | grep -A 1000 '"method":"tools/list"' | tail -n +2 | jq -r '.result.tools[].name'