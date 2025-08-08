#!/bin/bash

# Test biased reasoning with synthesis tracking
echo "🧪 Testing Biased Reasoning with Synthesis Evolution..."
echo "=================================================="

# Ensure database is running and migrated
echo "📊 Checking database..."
if ! pg_isready -h localhost -U lux_user -d lux_mcp; then
    echo "❌ Database is not running. Please start PostgreSQL first."
    exit 1
fi

# Build the project
echo "🔨 Building project..."
cargo build --release

# Start the server in the background
echo "🚀 Starting LUX MCP server..."
RUST_LOG=info DATABASE_URL="postgres://lux_user:lux_password@localhost/lux_mcp" \
    ./target/release/lux-mcp &
SERVER_PID=$!

# Give server time to start
sleep 2

# Function to make MCP call
make_mcp_call() {
    local method=$1
    local params=$2
    
    echo "$params" | jq -c "{
        jsonrpc: \"2.0\",
        method: \"$method\",
        params: .,
        id: 1
    }"
}

# Test 1: Initial query
echo -e "\n📝 Test 1: Starting biased reasoning session..."
RESPONSE1=$(make_mcp_call "tools/call" '{
    "name": "biased_reasoning",
    "arguments": {
        "query": "Should we implement a microservices architecture for our e-commerce platform that currently serves 10,000 daily users?"
    }
}' | nc -N localhost 3000)

echo "Response 1:"
echo "$RESPONSE1" | jq -r '.result.content[0].text' | head -20

# Extract session ID
SESSION_ID=$(echo "$RESPONSE1" | jq -r '.result.content[0].text' | grep -oP 'Session ID: \K[^\s]+' | head -1)
echo -e "\n🔑 Session ID: $SESSION_ID"

# Test 2: Continue reasoning
echo -e "\n🧠 Test 2: Continue reasoning..."
RESPONSE2=$(make_mcp_call "tools/call" '{
    "name": "biased_reasoning",
    "arguments": {
        "session_id": "'$SESSION_ID'"
    }
}' | nc -N localhost 3000)

echo "Response 2:"
echo "$RESPONSE2" | jq -r '.result.content[0].text' | head -20

# Test 3: Continue for bias check
echo -e "\n🔍 Test 3: Bias check..."
RESPONSE3=$(make_mcp_call "tools/call" '{
    "name": "biased_reasoning",
    "arguments": {
        "session_id": "'$SESSION_ID'"
    }
}' | nc -N localhost 3000)

echo "Response 3:"
echo "$RESPONSE3" | jq -r '.result.content[0].text' | head -20

# Test 4: Continue reasoning after bias check
echo -e "\n🧠 Test 4: Continue reasoning after bias check..."
RESPONSE4=$(make_mcp_call "tools/call" '{
    "name": "biased_reasoning",
    "arguments": {
        "session_id": "'$SESSION_ID'"
    }
}' | nc -N localhost 3000)

echo "Response 4:"
echo "$RESPONSE4" | jq -r '.result.content[0].text' | head -20

# Test 5: Final synthesis
echo -e "\n🎯 Test 5: Final synthesis..."
RESPONSE5=$(make_mcp_call "tools/call" '{
    "name": "biased_reasoning",
    "arguments": {
        "session_id": "'$SESSION_ID'"
    }
}' | nc -N localhost 3000)

echo "Response 5:"
echo "$RESPONSE5" | jq -r '.result.content[0].text' | head -30

# Check database for synthesis states
echo -e "\n📊 Checking database for synthesis states..."
psql -U lux_user -d lux_mcp -c "
SELECT 
    s.session_external_id,
    ss.version,
    ss.current_understanding,
    ss.confidence_score,
    ss.clarity_score,
    ss.last_updated_step,
    COUNT(DISTINCT i.id) as insight_count,
    COUNT(DISTINCT a.id) as action_count
FROM sessions s
LEFT JOIN synthesis_states ss ON s.id = ss.session_id
LEFT JOIN insights i ON ss.id = i.synthesis_state_id
LEFT JOIN action_items a ON ss.id = a.synthesis_state_id
WHERE s.session_external_id = '$SESSION_ID'
GROUP BY s.session_external_id, ss.id, ss.version, ss.current_understanding, 
         ss.confidence_score, ss.clarity_score, ss.last_updated_step
ORDER BY ss.version DESC
LIMIT 5;
"

# Check insights
echo -e "\n💡 Top insights for session:"
psql -U lux_user -d lux_mcp -c "
SELECT 
    i.insight,
    i.confidence,
    i.source_step,
    i.supported_by_evidence
FROM sessions s
JOIN synthesis_states ss ON s.id = ss.session_id
JOIN insights i ON ss.id = i.synthesis_state_id
WHERE s.session_external_id = '$SESSION_ID'
ORDER BY i.confidence DESC
LIMIT 5;
"

# Check action items
echo -e "\n📋 Action items for session:"
psql -U lux_user -d lux_mcp -c "
SELECT 
    a.action,
    a.priority,
    a.rationale
FROM sessions s
JOIN synthesis_states ss ON s.id = ss.session_id
JOIN action_items a ON ss.id = a.synthesis_state_id
WHERE s.session_external_id = '$SESSION_ID'
ORDER BY 
    CASE a.priority 
        WHEN 'high' THEN 1
        WHEN 'medium' THEN 2
        WHEN 'low' THEN 3
    END
LIMIT 5;
"

# Kill the server
echo -e "\n🛑 Stopping server..."
kill $SERVER_PID

echo -e "\n✅ Test completed!"