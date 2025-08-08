#!/bin/bash

echo "Simple Database Test"
echo "==================="

# Set environment variables
export DATABASE_URL="postgres://lux_user:lux_password@localhost/lux_mcp"
export OPENAI_API_KEY=${OPENAI_API_KEY:-"test-key"}
export RUST_LOG=info

echo "Starting server with database logging..."

# Create a simple test request
cat > test_request.json << EOF
{
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
    "method": "tools/call",
    "params": {
        "name": "biased_reasoning",
        "arguments": {
            "query": "Test query for database logging"
        }
    },
    "id": 2
}
EOF

# Run the server and capture stderr for logs
echo -e "\nRunning request..."
cat test_request.json | ./target/release/lux-mcp 2>server.log | jq -r '.result.content[0].text // .result // .error' | head -20

echo -e "\nServer logs:"
cat server.log | grep -E "(Database|database|DB)" | head -10

# Check database
echo -e "\nChecking database for sessions:"
psql $DATABASE_URL -c "SELECT COUNT(*) as session_count FROM sessions;"

# Clean up
rm -f test_request.json server.log

echo "Test complete!"