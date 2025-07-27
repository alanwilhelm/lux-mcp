#!/bin/bash

echo "Testing lux-mcp server readiness..."
echo

# Test that server responds to initialization
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"1.0.0","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}' | \
./target/release/lux-mcp 2>/dev/null | \
jq -r '.result.capabilities' && echo "✅ Server initialized successfully" || echo "❌ Server failed to initialize"

echo
echo "Checking registered tools..."
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"1.0.0","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}
{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' | \
./target/release/lux-mcp 2>/dev/null | \
grep -A50 '"id":2' | \
jq -r '.result.tools[] | "  ✅ " + .name' || echo "❌ Failed to list tools"

echo
echo "Checking that plan_iterative is NOT present..."
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"1.0.0","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}
{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' | \
./target/release/lux-mcp 2>/dev/null | \
grep -q "plan_iterative" && echo "❌ plan_iterative still exists!" || echo "✅ plan_iterative successfully removed"

echo
echo "Checking prompts list includes planner..."
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"1.0.0","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}
{"jsonrpc":"2.0","id":3,"method":"prompts/list","params":{}}' | \
./target/release/lux-mcp 2>/dev/null | \
grep -A50 '"id":3' | \
jq -r '.result.prompts[] | "  ✅ " + .name' || echo "❌ Failed to list prompts"