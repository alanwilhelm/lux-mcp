#!/bin/bash

echo "Testing Database Integration Directly"
echo "===================================="

# Set environment variables
export DATABASE_URL="postgres://lux_user:lux_password@localhost/lux_mcp"
export RUST_LOG=info

# Check if we have API keys
if [ -z "$OPENAI_API_KEY" ] && [ -z "$OPENROUTER_API_KEY" ]; then
    echo "Warning: No API keys found. Using test key."
    export OPENAI_API_KEY="test-key"
fi

echo "Database URL: $DATABASE_URL"

# Check initial session count
INITIAL_COUNT=$(psql -t -A $DATABASE_URL -c "SELECT COUNT(*) FROM sessions;")
echo "Initial sessions: $INITIAL_COUNT"

# Run biased reasoning directly
echo -e "\nRunning biased_reasoning tool..."
RESPONSE=$(echo '{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "biased_reasoning",
    "arguments": {
      "query": "Should we use microservices architecture for our new project?"
    }
  }
}' | ./target/release/lux-mcp 2>server.log | grep -A100 '"result"' | jq -r '.result.content[] | select(.type == "text") | .text' || echo "No response")

echo "Response snippet:"
echo "$RESPONSE" | head -10

# Extract session ID
SESSION_ID=$(echo "$RESPONSE" | grep -oE 'Session ID: [a-zA-Z0-9_-]+' | cut -d' ' -f3 | head -1)
echo -e "\nExtracted session_id: $SESSION_ID"

# Wait for database writes
sleep 2

# Check session count after test
echo -e "\nChecking database..."
FINAL_COUNT=$(psql -t -A $DATABASE_URL -c "SELECT COUNT(*) FROM sessions;")
echo "Final sessions: $FINAL_COUNT"

if [ $FINAL_COUNT -gt $INITIAL_COUNT ]; then
    echo "✅ Database logging is working!"
    
    # Show the latest session
    echo -e "\nLatest session details:"
    psql $DATABASE_URL -c "SELECT id, session_type, session_external_id, LEFT(query, 50) as query_snippet, status FROM sessions ORDER BY created_at DESC LIMIT 1;"
    
    # Show reasoning steps
    echo -e "\nReasoning steps:"
    psql $DATABASE_URL -c "
        SELECT rs.step_number, rs.step_type, rs.model_used, LEFT(rs.content, 50) as content_snippet
        FROM reasoning_steps rs 
        JOIN sessions s ON rs.session_id = s.id 
        WHERE s.session_external_id = '$SESSION_ID'
        ORDER BY rs.step_number 
        LIMIT 5;"
else
    echo "❌ No new session created"
    
    # Show server logs for debugging
    echo -e "\nServer logs (errors/warnings):"
    grep -E "(ERROR|WARN|Failed|failed|Database|database)" server.log | head -10
fi

# Clean up
rm -f server.log

echo -e "\nTest complete!"