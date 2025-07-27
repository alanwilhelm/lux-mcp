#!/bin/bash

echo "Testing confer tool directly"
echo "============================"

# Set environment variables
export OPENROUTER_API_KEY="${OPENROUTER_API_KEY:-}"

if [ -z "$OPENROUTER_API_KEY" ]; then
    echo "Error: OPENROUTER_API_KEY not set"
    exit 1
fi

# Build if needed
if [ ! -f target/release/lux-mcp ]; then
    cargo build --release || exit 1
fi

# Start the server and send commands
echo "Starting server and sending test commands..."

# Create a temporary file for commands
cat > /tmp/mcp_test_commands.jsonl << 'EOF'
{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"1.0","capabilities":{"tools":{}}},"id":1}
{"jsonrpc":"2.0","method":"initialized","id":2}
{"jsonrpc":"2.0","method":"tools/list","id":3}
EOF

echo "Sending commands to server..."
cat /tmp/mcp_test_commands.jsonl | RUST_LOG=info ./target/release/lux-mcp 2>/tmp/mcp_server.log

echo -e "\nServer log:"
cat /tmp/mcp_server.log

# Clean up
rm -f /tmp/mcp_test_commands.jsonl /tmp/mcp_server.log