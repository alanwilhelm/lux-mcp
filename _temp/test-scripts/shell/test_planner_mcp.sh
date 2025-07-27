#!/bin/bash

# Test the planner tool via MCP protocol
echo "Testing Interactive Planner Tool via MCP..."

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Build first
echo -e "${YELLOW}Building project...${NC}"
cargo build --release

# Function to send MCP request and get response
test_planner() {
    echo -e "\n${BLUE}=== Testing Interactive Planner Tool ===${NC}\n"
    
    # Create a test script that sends multiple requests
    cat > /tmp/test_planner_requests.json << 'EOF'
{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{"tools":{}}},"id":1}
{"jsonrpc":"2.0","method":"tools/call","params":{"name":"planner","arguments":{"step":"Create a comprehensive plan for building a real-time collaborative document editor with operational transformation, conflict resolution, and multi-user presence awareness","step_number":1,"total_steps":7,"next_step_required":true,"temperature":0.7}},"id":2}
{"jsonrpc":"2.0","method":"tools/call","params":{"name":"planner","arguments":{"step":"Analyze the core technical requirements: operational transformation algorithms, CRDT alternatives, WebSocket vs WebRTC for real-time sync, and database choices for document storage","step_number":2,"total_steps":7,"next_step_required":true}},"id":3}
{"jsonrpc":"2.0","method":"tools/call","params":{"name":"planner","arguments":{"step":"Design the document data model and conflict resolution strategy, considering both text and rich media content","step_number":3,"total_steps":7,"next_step_required":true}},"id":4}
{"jsonrpc":"2.0","method":"tools/call","params":{"name":"planner","arguments":{"step":"Plan the real-time synchronization architecture including presence system, cursor positions, and collaborative features","step_number":4,"total_steps":7,"next_step_required":true}},"id":5}
{"jsonrpc":"2.0","method":"tools/call","params":{"name":"planner","arguments":{"step":"REVISION: Reconsider the data model to better handle offline editing and eventual consistency when users reconnect","step_number":5,"total_steps":7,"next_step_required":true,"is_step_revision":true,"revises_step_number":3}},"id":6}
{"jsonrpc":"2.0","method":"tools/call","params":{"name":"planner","arguments":{"step":"Design the authentication, authorization, and document sharing system with granular permissions","step_number":6,"total_steps":7,"next_step_required":true}},"id":7}
{"jsonrpc":"2.0","method":"tools/call","params":{"name":"planner","arguments":{"step":"Create deployment and scaling strategy including CDN for static assets, WebSocket server scaling, and database replication","step_number":7,"total_steps":7,"next_step_required":false}},"id":8}
EOF

    # Send requests and capture responses
    echo -e "${GREEN}Sending planning requests...${NC}\n"
    ./target/release/lux-mcp < /tmp/test_planner_requests.json 2>/tmp/planner_stderr.log | while IFS= read -r line; do
        # Pretty print JSON responses
        echo "$line" | jq -C '
            if .method == "tools/call" then
                {
                    id: .id,
                    method: .method,
                    tool: .params.name,
                    step: .params.arguments.step_number // "N/A"
                }
            elif .result then
                {
                    id: .id,
                    status: (.result.content[0].text | 
                        if contains("DEEP THINKING REQUIRED") then "⏸️  Deep Thinking Pause"
                        elif contains("PLANNING STEP RECORDED") then "📋 Step Recorded"
                        elif contains("PLANNING COMPLETE") then "✅ Planning Complete"
                        else "📝 Response"
                        end
                    ),
                    preview: (.result.content[0].text | split("\n")[0:3] | join(" "))
                }
            else
                .
            end
        ' 2>/dev/null || echo "$line"
    done
    
    # Show any errors
    if [ -s /tmp/planner_stderr.log ]; then
        echo -e "\n${YELLOW}Server logs:${NC}"
        cat /tmp/planner_stderr.log
    fi
}

# Test branching functionality
test_branching() {
    echo -e "\n${BLUE}=== Testing Branching Functionality ===${NC}\n"
    
    cat > /tmp/test_branching.json << 'EOF'
{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{"tools":{}}},"id":1}
{"jsonrpc":"2.0","method":"tools/call","params":{"name":"planner","arguments":{"step":"Design a fault-tolerant distributed task queue system","step_number":1,"total_steps":5,"next_step_required":true}},"id":2}
{"jsonrpc":"2.0","method":"tools/call","params":{"name":"planner","arguments":{"step":"Choose between Redis-based queue vs Kafka vs RabbitMQ","step_number":2,"total_steps":5,"next_step_required":true}},"id":3}
{"jsonrpc":"2.0","method":"tools/call","params":{"name":"planner","arguments":{"step":"BRANCH: Explore Redis Streams approach with consumer groups","step_number":3,"total_steps":5,"next_step_required":true,"is_branch_point":true,"branch_from_step":2,"branch_id":"redis-approach"}},"id":4}
{"jsonrpc":"2.0","method":"tools/call","params":{"name":"planner","arguments":{"step":"BRANCH: Explore Kafka-based approach with partitioning","step_number":3,"total_steps":5,"next_step_required":true,"is_branch_point":true,"branch_from_step":2,"branch_id":"kafka-approach"}},"id":5}
{"jsonrpc":"2.0","method":"tools/call","params":{"name":"planner","arguments":{"step":"Design retry mechanism and dead letter queues","step_number":4,"total_steps":5,"next_step_required":true}},"id":6}
{"jsonrpc":"2.0","method":"tools/call","params":{"name":"planner","arguments":{"step":"Plan monitoring, alerting, and performance optimization","step_number":5,"total_steps":5,"next_step_required":false}},"id":7}
EOF

    ./target/release/lux-mcp < /tmp/test_branching.json 2>/dev/null | jq -C '.result.content[0].text' 2>/dev/null | grep -E "(BRANCH|branch)" || echo "Branching test complete"
}

# Run tests
test_planner
test_branching

echo -e "\n${GREEN}=== All Tests Complete ===${NC}"

# Cleanup
rm -f /tmp/test_planner_requests.json /tmp/test_branching.json /tmp/planner_stderr.log