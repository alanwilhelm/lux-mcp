#!/bin/bash

echo "🧪 Testing traced_reasoning with synthesis tracking..."
echo "==================================================="

# Enable debug logging to see synthesis operations
export RUST_LOG=info
export DATABASE_URL="postgres://lux_user:lux_password@localhost/lux_mcp"

# Function to send traced reasoning request
send_traced_request() {
    local thought_num=$1
    local thought_desc=$2
    local next_needed=$3
    
    echo -e "\n📝 Thought $thought_num: $thought_desc"
    
    cat << EOF | ./target/release/lux-mcp 2>&1 | grep -E "(Synthesis|synthesis|Understanding|Confidence|Clarity|Insights|REASONING THOUGHT)" | head -40
{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "test", "version": "1.0"}}}
{"jsonrpc": "2.0", "method": "notifications/initialized"}
{"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {"name": "traced_reasoning", "arguments": {"thought": "$thought_desc", "thought_number": $thought_num, "total_thoughts": 3, "next_thought_needed": $next_needed, "temperature": 0.7, "model": "gpt-4o-mini"}}}
EOF
}

# Test multi-step reasoning with synthesis
echo "🔍 Testing multi-step reasoning with synthesis..."

# Thought 1: Initial query
send_traced_request 1 "Should we migrate our monolithic application to microservices?" true

# Thought 2: Continue reasoning
send_traced_request 2 "Consider the team size, complexity, and maintenance overhead" true

# Thought 3: Final thought
send_traced_request 3 "Synthesize the analysis and provide a recommendation" false

echo -e "\n✅ Test completed!"