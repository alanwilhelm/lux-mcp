#!/bin/bash

echo "Testing Database Integration..."
echo "=============================="

# Load environment variables
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
fi

# Ensure DATABASE_URL is set
export DATABASE_URL=${DATABASE_URL:-"postgres://lux_user:lux_password@localhost/lux_mcp"}

echo "Database URL: $DATABASE_URL"

# Test database connection
echo -e "\n1. Testing database connection..."
psql $DATABASE_URL -c "SELECT 1;" > /dev/null 2>&1
if [ $? -eq 0 ]; then
    echo "✅ Database connection successful"
else
    echo "❌ Database connection failed"
    exit 1
fi

# Check initial session count
echo -e "\n2. Checking initial session count..."
INITIAL_COUNT=$(psql -t -A $DATABASE_URL -c "SELECT COUNT(*) FROM sessions;")
echo "Initial sessions: $INITIAL_COUNT"

# Run a biased reasoning test
echo -e "\n3. Running biased_reasoning tool..."
REQUEST='{
    "jsonrpc": "2.0",
    "method": "tools/call",
    "params": {
        "name": "biased_reasoning",
        "arguments": {
            "query": "Should we use microservices architecture for our new project?",
            "max_analysis_rounds": 2
        }
    },
    "id": 1
}'

# Initialize and run request
INIT_AND_REQUEST='{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {
        "protocolVersion": "2024-11-05",
        "capabilities": {},
        "clientInfo": {
            "name": "test-client",
            "version": "1.0"
        }
    }
}
'"$REQUEST"

echo "$INIT_AND_REQUEST" | ./target/release/lux-mcp 2>/dev/null | grep -A50 '"result"' | head -20

# Wait a moment for database writes
sleep 2

# Check session count after test
echo -e "\n4. Checking session count after test..."
FINAL_COUNT=$(psql -t -A $DATABASE_URL -c "SELECT COUNT(*) FROM sessions;")
echo "Final sessions: $FINAL_COUNT"

if [ $FINAL_COUNT -gt $INITIAL_COUNT ]; then
    echo "✅ New session created"
    
    # Show the latest session
    echo -e "\n5. Latest session details:"
    psql $DATABASE_URL -c "SELECT id, session_type, session_external_id, query, status FROM sessions ORDER BY created_at DESC LIMIT 1;"
    
    # Show reasoning steps
    echo -e "\n6. Reasoning steps for latest session:"
    psql $DATABASE_URL -c "
        SELECT rs.step_number, rs.step_type, rs.model_used, rs.confidence_score 
        FROM reasoning_steps rs 
        JOIN sessions s ON rs.session_id = s.id 
        ORDER BY s.created_at DESC, rs.step_number 
        LIMIT 10;"
    
    # Show synthesis states
    echo -e "\n7. Synthesis states for latest session:"
    psql $DATABASE_URL -c "
        SELECT ss.version, ss.confidence_score, ss.clarity_score 
        FROM synthesis_states ss 
        JOIN sessions s ON ss.session_id = s.id 
        ORDER BY s.created_at DESC, ss.version 
        LIMIT 5;"
else
    echo "❌ No new session created - database logging may not be working"
fi

echo -e "\nTest complete!"