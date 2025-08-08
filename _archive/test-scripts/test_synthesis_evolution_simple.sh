#!/bin/bash

echo "🧪 Testing synthesis evolution through multiple steps..."
echo "=================================================="

# Function to send MCP request and capture response
send_request() {
    local id=$1
    local session_id=$2
    local step_desc=$3
    
    echo -e "\n📝 Step $id: $step_desc"
    
    if [ -z "$session_id" ]; then
        # Initial query
        local args='{"query": "Should a 5-person startup use microservices architecture?", "max_analysis_rounds": 3}'
    else
        # Continue with session
        local args='{"query": "Should a 5-person startup use microservices architecture?", "session_id": "'"$session_id"'", "max_analysis_rounds": 3}'
    fi
    
    # Create temp files
    local temp_out=$(mktemp)
    local temp_err=$(mktemp)
    
    # Send request
    echo '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {
        "protocolVersion": "2024-11-05",
        "capabilities": {},
        "clientInfo": {"name": "test", "version": "1.0"}
    }
}
{"jsonrpc": "2.0", "method": "notifications/initialized"}
{"jsonrpc": "2.0", "id": '"$id"', "method": "tools/call", "params": {"name": "biased_reasoning", "arguments": '"$args"'}}' | \
    RUST_LOG=info DATABASE_URL="postgres://lux_user:lux_password@localhost/lux_mcp" \
    ./target/release/lux-mcp > "$temp_out" 2> "$temp_err"
    
    # Extract response
    local response=$(cat "$temp_out" | grep -E '"id":\s*'"$id" | jq -r '.result.content[0].text' 2>/dev/null)
    
    if [ -n "$response" ]; then
        # Show response type
        if echo "$response" | grep -q "Query Received"; then
            echo "✅ Got Query response"
        elif echo "$response" | grep -q "Reasoning Step"; then
            echo "✅ Got Reasoning response"
        elif echo "$response" | grep -q "Bias Analysis"; then
            echo "✅ Got Bias Analysis response"
        elif echo "$response" | grep -q "Final Synthesis"; then
            echo "✅ Got Final Synthesis response"
        fi
        
        # Extract session ID if not already set
        if [ -z "$session_id" ]; then
            SESSION_ID=$(echo "$response" | grep -oE 'Session ID: bias_[a-f0-9]+' | cut -d' ' -f3 | head -1)
            echo "🔑 Session ID: $SESSION_ID"
        fi
        
        # Show synthesis summary if present
        if echo "$response" | grep -q "EVOLVING SYNTHESIS"; then
            echo -e "\n📊 Synthesis snapshot:"
            echo "$response" | grep -A 20 "EVOLVING SYNTHESIS" | grep -E "(Current Understanding:|Confidence:|Clarity:|Insights:|Actions:)" | head -5
        fi
    else
        echo "❌ No response received"
        cat "$temp_err" | tail -10
    fi
    
    rm -f "$temp_out" "$temp_err"
}

# Run the test sequence
SESSION_ID=""

# Step 1: Initial query
send_request 2 "" "Initial Query"

# Check database after initial query
if [ -n "$SESSION_ID" ]; then
    echo -e "\n📊 Database check after Step 1:"
    psql -U lux_user -d lux_mcp -c "
    SELECT version, step_number, confidence_score, clarity_score, 
           LEFT(current_understanding, 60) as understanding
    FROM synthesis_states ss
    JOIN sessions s ON ss.session_id = s.id
    WHERE s.session_external_id = '$SESSION_ID'
    ORDER BY version;" 2>/dev/null
fi

# Step 2: First reasoning
send_request 3 "$SESSION_ID" "First Reasoning"

# Check database
if [ -n "$SESSION_ID" ]; then
    echo -e "\n📊 Database check after Step 2:"
    psql -U lux_user -d lux_mcp -c "
    SELECT version, step_number, confidence_score, clarity_score
    FROM synthesis_states ss
    JOIN sessions s ON ss.session_id = s.id
    WHERE s.session_external_id = '$SESSION_ID'
    ORDER BY version;" 2>/dev/null
fi

# Step 3: Bias analysis
send_request 4 "$SESSION_ID" "Bias Analysis"

# Step 4: Continue or synthesize
send_request 5 "$SESSION_ID" "Continue/Synthesize"

# Final database check
if [ -n "$SESSION_ID" ]; then
    echo -e "\n🎯 Final database state:"
    echo "========================"
    
    echo -e "\n📊 All synthesis versions:"
    psql -U lux_user -d lux_mcp -c "
    SELECT version, step_number, confidence_score, clarity_score,
           ready_for_decision, LEFT(current_understanding, 80) as understanding
    FROM synthesis_states ss
    JOIN sessions s ON ss.session_id = s.id
    WHERE s.session_external_id = '$SESSION_ID'
    ORDER BY version;" 2>/dev/null
    
    echo -e "\n💡 Insights collected:"
    psql -U lux_user -d lux_mcp -c "
    SELECT source_step, confidence, LEFT(insight_text, 80) as insight
    FROM insights i
    JOIN synthesis_states ss ON i.synthesis_state_id = ss.id
    JOIN sessions s ON ss.session_id = s.id
    WHERE s.session_external_id = '$SESSION_ID'
    ORDER BY i.created_at
    LIMIT 5;" 2>/dev/null
    
    echo -e "\n📋 Action items:"
    psql -U lux_user -d lux_mcp -c "
    SELECT source_step, priority, LEFT(action_text, 80) as action
    FROM action_items a
    JOIN synthesis_states ss ON a.synthesis_state_id = ss.id
    JOIN sessions s ON ss.session_id = s.id
    WHERE s.session_external_id = '$SESSION_ID'
    ORDER BY a.created_at
    LIMIT 5;" 2>/dev/null
fi

echo -e "\n✅ Test completed!"