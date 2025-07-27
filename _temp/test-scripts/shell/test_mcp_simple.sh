#!/bin/bash

echo "Testing MCP server directly..."
echo "=============================="

# Just send a simple message and see what happens
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"1.0","capabilities":{}}}' | ./target/release/lux-mcp 2>&1 | head -20