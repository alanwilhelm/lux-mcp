#!/bin/bash

# Simple test for the planner tool
echo "Testing Planner Tool with LLM generation..."

# Build first
echo "Building project..."
cargo build --release

# Create test requests
cat > /tmp/test_planner_llm.json << 'EOF'
{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{"tools":{}}},"id":1}
{"jsonrpc":"2.0","method":"tools/call","params":{"name":"planner","arguments":{"step":"Build a real-time collaborative document editor","step_number":1,"total_steps":5,"next_step_required":true,"model":"gpt-4o","temperature":0.7}},"id":2}
{"jsonrpc":"2.0","method":"tools/call","params":{"name":"planner","arguments":{"step":"Focus on the data synchronization architecture","step_number":2,"total_steps":5,"next_step_required":true}},"id":3}
{"jsonrpc":"2.0","method":"tools/call","params":{"name":"planner","arguments":{"step":"Design the conflict resolution strategy","step_number":3,"total_steps":5,"next_step_required":true}},"id":4}
{"jsonrpc":"2.0","method":"tools/call","params":{"name":"planner","arguments":{"step":"Plan the user presence and cursor tracking system","step_number":4,"total_steps":5,"next_step_required":true}},"id":5}
{"jsonrpc":"2.0","method":"tools/call","params":{"name":"planner","arguments":{"step":"Define deployment and scaling approach","step_number":5,"total_steps":5,"next_step_required":false}},"id":6}
EOF

echo -e "\nRunning planner test..."
./target/release/lux-mcp < /tmp/test_planner_llm.json 2>/tmp/planner_err.log | jq -C '.result.content[0].text // .error // .' 2>/dev/null

echo -e "\nServer logs:"
cat /tmp/planner_err.log | grep -E "(Planning step|model|Warning)" || echo "No relevant logs found"

# Cleanup
rm -f /tmp/test_planner_llm.json /tmp/planner_err.log