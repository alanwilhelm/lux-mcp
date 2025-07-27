#!/bin/bash

# Source environment
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
fi

echo "Running test with full logs captured..."
echo "======================================"
echo

# Enable debug logging
export RUST_LOG=debug

# Create a test file that captures both stdout and stderr
cat > /tmp/test_planner.json << 'EOF'
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"planner","arguments":{"step":"Test planning","step_number":1,"total_steps":3,"next_step_required":true}}}
EOF

# Run and capture everything
echo "Running planner test..."
./target/release/lux-mcp < /tmp/test_planner.json > /tmp/lux_stdout.log 2> /tmp/lux_stderr.log

echo "STDOUT (MCP responses):"
echo "======================="
cat /tmp/lux_stdout.log | tail -50

echo -e "\n\nSTDERR (Server logs):"
echo "===================="
cat /tmp/lux_stderr.log | tail -100

echo -e "\n\nChecking for errors in logs:"
grep -i "error\|failed\|timeout" /tmp/lux_stderr.log | tail -20

# Clean up
rm -f /tmp/test_planner.json /tmp/lux_stdout.log /tmp/lux_stderr.log