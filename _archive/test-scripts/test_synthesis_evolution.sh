#!/bin/bash

# Test biased reasoning synthesis evolution through MCP
echo "🧪 Testing Synthesis Evolution..."
echo "================================="

# Start the server
echo "🚀 Starting LUX MCP server..."
RUST_LOG=info DATABASE_URL="postgres://lux_user:lux_password@localhost/lux_mcp" \
    ./target/release/lux-mcp &
SERVER_PID=$!

# Give server time to start
sleep 3

# Test using proper MCP protocol
echo -e "\n📝 Test 1: Initial Query"
echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"biased_reasoning","arguments":{"query":"Should we migrate from monolith to microservices for our e-commerce platform?"}},"id":1}' | nc localhost 3000 | jq -r '.result.content[0].text' | head -50

echo -e "\n⏸️  Waiting before next step..."
sleep 2

# Continue the session (the server should have created a session)
echo -e "\n🧠 Test 2: Continue Reasoning"
echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"biased_reasoning","arguments":{"query":"Should we migrate from monolith to microservices for our e-commerce platform?"}},"id":2}' | nc localhost 3000 | jq -r '.result.content[0].text' | head -50

sleep 2

echo -e "\n🔍 Test 3: Bias Check"
echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"biased_reasoning","arguments":{"query":"Should we migrate from monolith to microservices for our e-commerce platform?"}},"id":3}' | nc localhost 3000 | jq -r '.result.content[0].text' | head -50

sleep 2

echo -e "\n🎯 Test 4: Final Synthesis"  
echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"biased_reasoning","arguments":{"query":"Should we migrate from monolith to microservices for our e-commerce platform?"}},"id":4}' | nc localhost 3000 | jq -r '.result.content[0].text' | head -50

# Check database
echo -e "\n📊 Checking synthesis states in database..."
psql -U lux_user -d lux_mcp -c "
SELECT 
    s.session_external_id,
    ss.version,
    LEFT(ss.current_understanding, 100) as understanding_preview,
    ss.confidence_score,
    ss.clarity_score,
    ss.step_number
FROM sessions s
JOIN synthesis_states ss ON s.id = ss.session_id
ORDER BY s.created_at DESC, ss.version DESC
LIMIT 5;
"

echo -e "\n💡 Recent insights..."
psql -U lux_user -d lux_mcp -c "
SELECT 
    LEFT(i.insight_text, 80) as insight,
    i.confidence,
    i.source_step
FROM insights i
JOIN synthesis_states ss ON i.synthesis_state_id = ss.id
JOIN sessions s ON ss.session_id = s.id
ORDER BY s.created_at DESC, i.created_at DESC
LIMIT 5;
"

echo -e "\n📋 Recent action items..."
psql -U lux_user -d lux_mcp -c "
SELECT 
    LEFT(a.action_text, 80) as action,
    a.priority,
    LEFT(a.rationale, 60) as rationale
FROM action_items a
JOIN synthesis_states ss ON a.synthesis_state_id = ss.id  
JOIN sessions s ON ss.session_id = s.id
ORDER BY s.created_at DESC, a.created_at DESC
LIMIT 5;
"

# Kill server
kill $SERVER_PID 2>/dev/null

echo -e "\n✅ Test completed!"