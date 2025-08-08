#!/bin/bash

echo "Testing Database Integration with MCP Protocol"
echo "============================================="

# Load environment variables
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
fi

# Ensure DATABASE_URL is set
export DATABASE_URL=${DATABASE_URL:-"postgres://lux_user:lux_password@localhost/lux_mcp"}
export RUST_LOG=info

echo "Database URL: $DATABASE_URL"

# Check initial session count
INITIAL_COUNT=$(psql -t -A $DATABASE_URL -c "SELECT COUNT(*) FROM sessions;")
echo "Initial sessions: $INITIAL_COUNT"

# Run a biased reasoning test using cargo run (which handles MCP properly)
echo -e "\nRunning biased_reasoning tool..."
REQUEST='{
    "jsonrpc": "2.0",
    "method": "tools/call",
    "params": {
        "name": "biased_reasoning",
        "arguments": {
            "query": "Should we use microservices architecture?"
        }
    },
    "id": 1
}'

# Run with cargo which handles the MCP protocol correctly
echo "$REQUEST" | cargo run --release 2>server.log | jq -r '.result.content[0].text // empty' | head -20

# Wait for database writes
sleep 2

# Check session count after test
echo -e "\nChecking database..."
FINAL_COUNT=$(psql -t -A $DATABASE_URL -c "SELECT COUNT(*) FROM sessions;")
echo "Final sessions: $FINAL_COUNT"

if [ $FINAL_COUNT -gt $INITIAL_COUNT ]; then
    echo "✅ Database logging is working!"
    
    # Show the latest session
    echo -e "\nLatest session:"
    psql $DATABASE_URL -x -c "SELECT * FROM sessions ORDER BY created_at DESC LIMIT 1;"
else
    echo "❌ No new session created"
    
    # Check server logs
    echo -e "\nServer logs:"
    grep -E "(Database|database|DB|Failed|Error|error)" server.log || echo "No relevant logs found"
fi

# Clean up
rm -f server.log

echo -e "\nTest complete!"