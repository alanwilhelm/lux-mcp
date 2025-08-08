#!/bin/bash

# Test database integration with biased_reasoning

echo "Testing database integration..."

# Export database URL
export DATABASE_URL=postgres://lux_user:lux_password@localhost/lux_mcp

# Make sure database is set up
echo "Verifying database connection..."
psql $DATABASE_URL -c "SELECT COUNT(*) FROM sessions;" || {
    echo "Database not accessible. Make sure PostgreSQL is running and the database exists."
    exit 1
}

# Start the server in background
echo "Starting lux-mcp server..."
./target/release/lux-mcp &
SERVER_PID=$!

# Wait for server to start
sleep 2

# Test biased reasoning with database logging
echo "Testing biased reasoning with database logging..."
echo '{
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
{
    "jsonrpc": "2.0",
    "id": 2,
    "method": "tools/call",
    "params": {
        "name": "biased_reasoning",
        "arguments": {
            "query": "Should we use microservices architecture for our new project?"
        }
    }
}' | nc -N localhost 3000 | jq .

# Check database for logged session
echo -e "\nChecking database for logged sessions..."
psql $DATABASE_URL -c "SELECT id, session_type, query, status FROM sessions ORDER BY created_at DESC LIMIT 1;"

echo -e "\nChecking reasoning steps..."
psql $DATABASE_URL -c "SELECT step_number, step_type, model_used FROM reasoning_steps ORDER BY created_at DESC LIMIT 5;"

# Kill the server
kill $SERVER_PID

echo "Test complete!"