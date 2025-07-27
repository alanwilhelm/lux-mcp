#!/bin/bash

# Test the interactive planner tool
echo "Testing Interactive Planner Tool..."

PORT=${1:-3000}
URL="http://localhost:$PORT"

# Function to send JSON-RPC request
send_request() {
    local method=$1
    local params=$2
    local id=${3:-1}
    
    curl -s -X POST $URL \
        -H "Content-Type: application/json" \
        -d "{
            \"jsonrpc\": \"2.0\",
            \"method\": \"$method\",
            \"params\": $params,
            \"id\": $id
        }" | jq '.'
}

# Step 1: Initialize the planner with the task description
echo -e "\n\n=== Step 1: Initialize Planning ==="
send_request "tools/call" '{
    "name": "planner",
    "arguments": {
        "step": "Create a comprehensive microservices architecture plan for migrating a monolithic e-commerce application to microservices, considering database decomposition, API gateway design, service discovery, and deployment strategies",
        "step_number": 1,
        "total_steps": 8,
        "next_step_required": true,
        "model": "gpt-4o"
    }
}' 1

# Wait a bit
sleep 2

# Step 2: Continue planning
echo -e "\n\n=== Step 2: Continue Planning ==="
send_request "tools/call" '{
    "name": "planner",
    "arguments": {
        "step": "Analyze the current monolithic architecture and identify service boundaries based on business domains, data ownership, and team structure",
        "step_number": 2,
        "total_steps": 8,
        "next_step_required": true
    }
}' 2

# Wait a bit
sleep 2

# Step 3: More planning
echo -e "\n\n=== Step 3: More Planning ==="
send_request "tools/call" '{
    "name": "planner",
    "arguments": {
        "step": "Design the database decomposition strategy including handling of shared data, distributed transactions, and data consistency patterns",
        "step_number": 3,
        "total_steps": 8,
        "next_step_required": true
    }
}' 3

# Wait a bit
sleep 2

# Step 4: Branching example
echo -e "\n\n=== Step 4: Branch to explore alternative ==="
send_request "tools/call" '{
    "name": "planner",
    "arguments": {
        "step": "BRANCH: Explore event-driven architecture with event sourcing as an alternative to traditional REST-based microservices",
        "step_number": 4,
        "total_steps": 8,
        "next_step_required": true,
        "is_branch_point": true,
        "branch_from_step": 3,
        "branch_id": "event-driven-approach"
    }
}' 4

# Wait a bit
sleep 2

# Step 5: Continue main path
echo -e "\n\n=== Step 5: Continue Main Path ==="
send_request "tools/call" '{
    "name": "planner",
    "arguments": {
        "step": "Define API gateway architecture with authentication, rate limiting, request routing, and protocol translation",
        "step_number": 5,
        "total_steps": 8,
        "next_step_required": true
    }
}' 5

# Wait a bit
sleep 2

# Step 6: Revise earlier step
echo -e "\n\n=== Step 6: Revise Step 2 ==="
send_request "tools/call" '{
    "name": "planner",
    "arguments": {
        "step": "REVISION: Re-analyze service boundaries with focus on minimizing inter-service communication and considering team cognitive load",
        "step_number": 6,
        "total_steps": 8,
        "next_step_required": true,
        "is_step_revision": true,
        "revises_step_number": 2
    }
}' 6

# Wait a bit
sleep 2

# Step 7: Almost done
echo -e "\n\n=== Step 7: Service Discovery and Deployment ==="
send_request "tools/call" '{
    "name": "planner",
    "arguments": {
        "step": "Design service discovery mechanism (Consul vs Kubernetes native) and deployment strategy including CI/CD pipelines, container orchestration, and monitoring",
        "step_number": 7,
        "total_steps": 8,
        "next_step_required": true
    }
}' 7

# Wait a bit
sleep 2

# Step 8: Final step - complete the plan
echo -e "\n\n=== Step 8: Complete Planning ==="
send_request "tools/call" '{
    "name": "planner",
    "arguments": {
        "step": "Create migration roadmap with phases, rollback strategies, and success metrics for tracking the monolith-to-microservices transformation",
        "step_number": 8,
        "total_steps": 8,
        "next_step_required": false
    }
}' 8

echo -e "\n\n=== Test Complete ==="