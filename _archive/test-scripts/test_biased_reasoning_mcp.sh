#!/bin/bash

echo "🧪 Testing biased reasoning through MCP server..."

# Start server in background
echo "🚀 Starting server..."
RUST_LOG=info DATABASE_URL="postgres://lux_user:lux_password@localhost/lux_mcp" \
    ./target/release/lux-mcp > /tmp/mcp_output.txt 2>/tmp/mcp_error.txt &
SERVER_PID=$!

# Give server time to start
sleep 2

# Send MCP messages using expect or python
echo "📝 Sending MCP messages..."
python3 << 'EOF'
import json
import subprocess
import time

# MCP messages
messages = [
    {"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "test", "version": "1.0"}}},
    {"jsonrpc": "2.0", "method": "notifications/initialized"},
    {"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {"name": "biased_reasoning", "arguments": {"query": "Should we adopt microservices?", "max_analysis_rounds": 2}}}
]

# Send messages
proc = subprocess.Popen(
    ['./target/release/lux-mcp'],
    stdin=subprocess.PIPE,
    stdout=subprocess.PIPE,
    stderr=subprocess.PIPE,
    env={"RUST_LOG": "info", "DATABASE_URL": "postgres://lux_user:lux_password@localhost/lux_mcp"}
)

# Send all messages
for msg in messages:
    proc.stdin.write((json.dumps(msg) + '\n').encode())
    proc.stdin.flush()
    time.sleep(0.5)

# Wait a bit for processing
time.sleep(3)

# Read output
try:
    stdout, stderr = proc.communicate(timeout=5)
    
    # Parse responses
    for line in stdout.decode().split('\n'):
        if line.strip():
            try:
                response = json.loads(line)
                if response.get('id') == 2:
                    print("✅ Got response for biased reasoning:")
                    content = response.get('result', {}).get('content', [{}])[0].get('text', '')
                    print(content[:500])
                    
                    # Extract session ID
                    import re
                    match = re.search(r'Session ID: (bias_[a-f0-9]+)', content)
                    if match:
                        print(f"\n🔑 Session ID: {match.group(1)}")
                        with open('/tmp/session_id.txt', 'w') as f:
                            f.write(match.group(1))
                    
            except json.JSONDecodeError:
                pass
except subprocess.TimeoutExpired:
    proc.kill()
    print("⏰ Process timed out")

EOF

# Check if we got a session ID
if [ -f /tmp/session_id.txt ]; then
    SESSION_ID=$(cat /tmp/session_id.txt)
    echo -e "\n📊 Checking database for session: $SESSION_ID"
    
    psql -U lux_user -d lux_mcp -c "
    SELECT 
        s.session_external_id,
        ss.version,
        LEFT(ss.current_understanding, 60) as understanding,
        ss.confidence_score,
        ss.clarity_score
    FROM sessions s
    JOIN synthesis_states ss ON s.id = ss.session_id
    WHERE s.session_external_id = '$SESSION_ID'
    ORDER BY ss.version;
    "
    
    echo -e "\n💡 Checking for insights..."
    psql -U lux_user -d lux_mcp -c "
    SELECT 
        LEFT(i.insight_text, 60) as insight,
        i.confidence
    FROM insights i
    JOIN synthesis_states ss ON i.synthesis_state_id = ss.id
    JOIN sessions s ON ss.session_id = s.id
    WHERE s.session_external_id = '$SESSION_ID';
    "
else
    echo "❌ No session ID found"
fi

# Clean up
rm -f /tmp/session_id.txt /tmp/mcp_output.txt /tmp/mcp_error.txt

echo -e "\n✅ Test completed!"