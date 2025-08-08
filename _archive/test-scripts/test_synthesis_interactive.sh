#!/bin/bash

echo "🧪 Testing biased reasoning with synthesis (interactive)..."

# Create a test file with MCP commands
cat > /tmp/mcp_test_input.txt << 'EOF'
{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "test", "version": "1.0"}}}
{"jsonrpc": "2.0", "method": "notifications/initialized"}
{"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {"name": "biased_reasoning", "arguments": {"query": "Should small startups adopt microservices architecture from day one?", "max_analysis_rounds": 2}}}
EOF

echo "📝 Starting server and sending commands..."
RUST_LOG=debug DATABASE_URL="postgres://lux_user:lux_password@localhost/lux_mcp" \
    ./target/release/lux-mcp < /tmp/mcp_test_input.txt > /tmp/mcp_test_output.txt 2>/tmp/mcp_test_error.txt

echo -e "\n📖 Output:"
cat /tmp/mcp_test_output.txt | jq . 2>/dev/null || cat /tmp/mcp_test_output.txt

echo -e "\n📋 Error log (last 50 lines):"
tail -50 /tmp/mcp_test_error.txt

# Check if we got a biased reasoning response
echo -e "\n🔍 Extracting biased reasoning response:"
cat /tmp/mcp_test_output.txt | jq -r 'select(.id == 2) | .result.content[0].text' 2>/dev/null

# Check database for synthesis states
echo -e "\n📊 Checking database for synthesis states..."
psql -U lux_user -d lux_mcp -c "
SELECT 
    s.session_external_id,
    ss.version,
    LEFT(ss.current_understanding, 80) as understanding,
    ss.confidence_score,
    ss.clarity_score,
    ss.step_number
FROM sessions s
JOIN synthesis_states ss ON s.id = ss.session_id
WHERE s.created_at > NOW() - INTERVAL '5 minutes'
ORDER BY s.created_at DESC, ss.version DESC
LIMIT 5;
"

echo -e "\n✅ Test completed!"