#!/bin/bash

echo "🧪 Testing planner with synthesis tracking..."
echo "============================================"

# Function to send MCP request
send_planner_request() {
    local step_num=$1
    local step_desc=$2
    local next_required=$3
    
    echo -e "\n📝 Step $step_num: $step_desc"
    
    cat << EOF | ./target/release/lux-mcp 2>&1 | grep -A20 "Synthesis State" || echo "No synthesis output found"
{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "test", "version": "1.0"}}}
{"jsonrpc": "2.0", "method": "notifications/initialized"}
{"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {"name": "planner", "arguments": {"step": "$step_desc", "step_number": $step_num, "total_steps": 5, "next_step_required": $next_required}}}
EOF
}

# Test planner with multiple steps
echo "📋 Testing multi-step planning with synthesis..."

# Step 1: Initial planning
send_planner_request 1 "Design a REST API for a task management system" true

# Step 2: Continue planning
send_planner_request 2 "Define the core endpoints and data models" true

# Step 3: More planning
send_planner_request 3 "Plan authentication and authorization strategy" true

# Step 4: Near completion
send_planner_request 4 "Design database schema and relationships" true

# Step 5: Final step
send_planner_request 5 "Plan deployment and monitoring strategy" false

echo -e "\n✅ Test completed!"