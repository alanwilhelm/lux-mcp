#!/bin/bash

echo "Simple MCP Protocol Test"
echo "========================"

# Test with rmcp (if available)
if command -v rmcp &> /dev/null; then
    echo "Using rmcp to test the server..."
    
    # Start server in background
    ./target/release/lux-mcp &
    SERVER_PID=$!
    sleep 1
    
    # Test with rmcp
    echo "Listing tools:"
    echo '{"method": "tools/list"}' | rmcp --server stdio -- ./target/release/lux-mcp
    
    # Kill server
    kill $SERVER_PID 2>/dev/null
else
    echo "rmcp not found, using direct pipe test..."
    
    # Create a test script that sends multiple commands
    cat > test_commands.txt << EOF
{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"0.1.0","capabilities":{"tools":{}}},"id":1}
{"jsonrpc":"2.0","method":"tools/list","id":2}
EOF
    
    echo "Running server with test commands..."
    cat test_commands.txt | ./target/release/lux-mcp 2>&1 | while IFS= read -r line; do
        echo "$line"
        # Try to parse as JSON
        echo "$line" | jq . 2>/dev/null || true
    done
    
    rm -f test_commands.txt
fi