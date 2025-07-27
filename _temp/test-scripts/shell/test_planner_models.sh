#!/bin/bash

# Source environment
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
fi

echo "Testing planner with different models..."
echo "========================================"
echo

# Enable info logging
export RUST_LOG=info

# Test 1: Fast model (gpt-4)
echo "Test 1: GPT-4 (should be fast - under 5 seconds)"
cat > /tmp/planner_test1.jsonl << 'EOF'
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"0.1.0","capabilities":{"tools":{}}}}
{"jsonrpc":"2.0","method":"notifications/initialized"}
{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"planner","arguments":{"step":"Plan a simple task","step_number":1,"total_steps":3,"next_step_required":true,"model":"gpt-4"}}}
EOF

echo "Starting GPT-4 test..."
time ./target/release/lux-mcp < /tmp/planner_test1.jsonl 2>&1 | grep -E "(result|error|Model:|Planning)" | tail -20

echo -e "\n\n========================================"
echo "Test 2: Default model (o3-pro - will be slow)"
echo "This will take 30 seconds to 5 minutes..."
echo "Press Ctrl+C if you don't want to wait"
cat > /tmp/planner_test2.jsonl << 'EOF'
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"0.1.0","capabilities":{"tools":{}}}}
{"jsonrpc":"2.0","method":"notifications/initialized"}
{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"planner","arguments":{"step":"Plan a complex task","step_number":1,"total_steps":3,"next_step_required":true}}}
EOF

echo "Starting o3-pro test (be patient)..."
./target/release/lux-mcp < /tmp/planner_test2.jsonl 2>&1 | grep -E "(result|error|Model:|Planning)" | tail -20 &
PID=$!

# Give it 30 seconds before showing a message
sleep 30
if kill -0 $PID 2>/dev/null; then
    echo "Still waiting for o3-pro response... (normal behavior)"
fi

wait $PID

# Clean up
rm -f /tmp/planner_test1.jsonl /tmp/planner_test2.jsonl

echo -e "\n\nTests complete!"
echo "If GPT-4 worked but o3-pro failed/timed out, use model: \"gpt-4\" in your planner calls."