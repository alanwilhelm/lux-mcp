#!/bin/bash

echo "🧪 Testing biased reasoning with synthesis..."

# Function to make a call and capture both stdout and stderr
make_call() {
    local query="$1"
    local session_id="$2"
    local temp_out=$(mktemp)
    local temp_err=$(mktemp)
    
    if [ -z "$session_id" ]; then
        local args='{"query": "'"$query"'"}'
    else
        local args='{"query": "'"$query"'", "session_id": "'"$session_id"'"}'
    fi
    
    echo '{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "test", "version": "1.0"}}}
{"jsonrpc": "2.0", "method": "notifications/initialized"}
{"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {"name": "biased_reasoning", "arguments": '"$args"'}}' | \
    RUST_LOG=info DATABASE_URL="postgres://lux_user:lux_password@localhost/lux_mcp" \
    ./target/release/lux-mcp > "$temp_out" 2> "$temp_err"
    
    # Extract the result
    local result=$(cat "$temp_out" | jq -s '.[2].result.content[0].text' -r 2>/dev/null)
    
    if [ -n "$result" ] && [ "$result" != "null" ]; then
        echo "$result"
    else
        echo "Error occurred. Check stderr:"
        cat "$temp_err"
    fi
    
    rm -f "$temp_out" "$temp_err"
}

# Test 1: Initial query
echo "📝 Step 1: Initial Query"
echo "========================"
RESPONSE1=$(make_call "Should small startups use microservices architecture?")
echo "$RESPONSE1" | head -40

# Extract session ID
SESSION_ID=$(echo "$RESPONSE1" | grep -oE 'Session ID: bias_[a-f0-9]+' | cut -d' ' -f3 | head -1)
echo -e "\n🔑 Extracted Session ID: $SESSION_ID"

if [ -z "$SESSION_ID" ]; then
    echo "❌ Failed to extract session ID"
    exit 1
fi

# Check database
echo -e "\n📊 Checking database after step 1..."
psql -U lux_user -d lux_mcp -c "
SELECT 
    s.session_external_id,
    ss.version,
    LEFT(ss.current_understanding, 80) as understanding,
    ss.confidence_score,
    ss.clarity_score
FROM sessions s
JOIN synthesis_states ss ON s.id = ss.session_id
WHERE s.session_external_id = '$SESSION_ID'
ORDER BY ss.version;
"

# Test 2: Continue reasoning
echo -e "\n🧠 Step 2: Continue Reasoning"
echo "============================="
RESPONSE2=$(make_call "Should small startups use microservices architecture?" "$SESSION_ID")
echo "$RESPONSE2" | head -40

# Check database again
echo -e "\n📊 Checking database after step 2..."
psql -U lux_user -d lux_mcp -c "
SELECT 
    s.session_external_id,
    ss.version,
    LEFT(ss.current_understanding, 80) as understanding,
    ss.confidence_score,
    ss.clarity_score
FROM sessions s
JOIN synthesis_states ss ON s.id = ss.session_id
WHERE s.session_external_id = '$SESSION_ID'
ORDER BY ss.version;
"

# Check insights
echo -e "\n💡 Insights collected..."
psql -U lux_user -d lux_mcp -c "
SELECT 
    LEFT(i.insight_text, 60) as insight,
    i.confidence,
    i.source_step
FROM insights i
JOIN synthesis_states ss ON i.synthesis_state_id = ss.id
JOIN sessions s ON ss.session_id = s.id
WHERE s.session_external_id = '$SESSION_ID'
ORDER BY i.confidence DESC;
"

echo -e "\n✅ Test completed!"