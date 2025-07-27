#!/bin/bash

echo "Testing lux-mcp tools list..."
echo

# Initialize and list tools
(echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"1.0.0","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'; \
 sleep 0.1; \
 echo '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}') | \
./target/release/lux-mcp 2>&1 | grep -A200 '"method":"tools/list"' | grep -E '"name":|plan_iterative'

echo
echo "Checking prompts..."
(echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"1.0.0","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'; \
 sleep 0.1; \
 echo '{"jsonrpc":"2.0","id":3,"method":"prompts/list","params":{}}') | \
./target/release/lux-mcp 2>&1 | grep -A100 '"method":"prompts/list"' | grep '"name":'