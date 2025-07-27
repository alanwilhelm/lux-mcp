#!/bin/bash

echo "MCP Server Log Analysis"
echo "======================"
echo

# Check current configuration
echo "1. Current Model Configuration:"
echo "------------------------------"
if [ -f .env ]; then
    echo "LUX_DEFAULT_CHAT_MODEL: $(grep "^LUX_DEFAULT_CHAT_MODEL=" .env | cut -d= -f2)"
    echo "LUX_DEFAULT_REASONING_MODEL: $(grep "^LUX_DEFAULT_REASONING_MODEL=" .env | cut -d= -f2)"
    echo "LUX_DEFAULT_BIAS_CHECKER_MODEL: $(grep "^LUX_DEFAULT_BIAS_CHECKER_MODEL=" .env | cut -d= -f2)"
else
    echo "No .env file found!"
fi

echo -e "\n2. Model Usage by Tool:"
echo "-----------------------"
echo "- confer: Uses LUX_DEFAULT_CHAT_MODEL (should be fast like gpt-4o)"
echo "- planner: Uses LUX_DEFAULT_REASONING_MODEL (can be o3-pro)"
echo "- traced_reasoning: Uses LUX_DEFAULT_REASONING_MODEL (can be o3-pro)"
echo "- biased_reasoning: Primary uses LUX_DEFAULT_REASONING_MODEL, verifier uses LUX_DEFAULT_BIAS_CHECKER_MODEL"

echo -e "\n3. Running MCP Server Logs:"
echo "---------------------------"
echo "The MCP server logs to stderr. To see real-time logs while using Claude:"
echo
echo "Option A - Run server manually with logging:"
echo "  RUST_LOG=info ./target/release/lux-mcp 2> lux-mcp.log"
echo "  Then tail the log in another terminal: tail -f lux-mcp.log"
echo
echo "Option B - Test specific tool with logs:"
echo "  ./test_with_logs.sh"
echo
echo "Option C - For Claude Desktop, check system logs:"
echo "  On macOS: ~/Library/Logs/Claude/mcp-*.log"

echo -e "\n4. Common Issues:"
echo "-----------------"
echo "❌ 'Failed to complete chat request' - Using slow model (o3) for chat"
echo "❌ 'Failed to generate planning step' - Model timeout (o3-pro takes minutes)"
echo "❌ Empty responses - Insufficient tokens for reasoning models"
echo
echo "✅ Solution: Use appropriate models for each tool type"

echo -e "\n5. Test with Proper Models:"
echo "---------------------------"
echo "# Test confer with fast model:"
echo 'echo '"'"'{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"confer","arguments":{"message":"Hello","model":"gpt-4o"}}}'"'"' | ./target/release/lux-mcp'
echo
echo "# Test planner with explicit model:"
echo 'echo '"'"'{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"planner","arguments":{"step":"Test","step_number":1,"total_steps":3,"next_step_required":true,"model":"gpt-4"}}}'"'"' | ./target/release/lux-mcp'