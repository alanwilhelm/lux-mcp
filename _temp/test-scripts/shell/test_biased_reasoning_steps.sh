#!/bin/bash

# Test script for biased_reasoning step-by-step functionality with proper MCP protocol

echo "Testing biased_reasoning step-by-step operation..."

# Create a temporary file for the session
SESSION_FILE=$(mktemp)

# Function to send MCP request
send_request() {
    local id=$1
    local method=$2
    local params=$3
    
    cat <<EOF
{"jsonrpc": "2.0", "id": $id, "method": "$method", "params": $params}
EOF
}

# Initialize and test in one session
{
    # Send initialize
    send_request 1 "initialize" '{
        "protocolVersion": "2024-11-05",
        "capabilities": {},
        "clientInfo": {"name": "test-client", "version": "1.0.0"}
    }'
    
    # Send initialized notification
    echo '{"jsonrpc": "2.0", "method": "notifications/initialized"}'
    
    # Test 1: First call - should create new session
    send_request 2 "tools/call" '{
        "name": "biased_reasoning",
        "arguments": {
            "query": "What are the potential biases in assuming all software bugs are due to developer errors?"
        }
    }'
    
} | ./target/release/lux-mcp 2>/dev/null > "$SESSION_FILE"

# Extract the first response
echo -e "\n1. First call - should create new session:"
FIRST_RESPONSE=$(cat "$SESSION_FILE" | grep -A200 '"id":2' | head -n 200)
echo "$FIRST_RESPONSE" | jq -r '.result.content[0].text' | head -20

# Extract session_id
SESSION_ID=$(echo "$FIRST_RESPONSE" | jq -r '.result.content[0].text' | grep -oE 'Session ID: bias_[a-f0-9]+' | cut -d' ' -f3)
if [ -z "$SESSION_ID" ]; then
    # Try alternative pattern
    SESSION_ID=$(echo "$FIRST_RESPONSE" | jq -r '.result.content[0].text' | grep -oE '"session_id":\s*"bias_[a-f0-9]+"' | cut -d'"' -f4)
fi

echo -e "\nExtracted session_id: $SESSION_ID"

# Continue with second call
rm -f "$SESSION_FILE"
SESSION_FILE=$(mktemp)

{
    # Re-initialize for new session
    send_request 1 "initialize" '{
        "protocolVersion": "2024-11-05",
        "capabilities": {},
        "clientInfo": {"name": "test-client", "version": "1.0.0"}
    }'
    
    echo '{"jsonrpc": "2.0", "method": "notifications/initialized"}'
    
    # Test 2: Continue with session_id
    send_request 2 "tools/call" "{
        \"name\": \"biased_reasoning\",
        \"arguments\": {
            \"query\": \"What are the potential biases in assuming all software bugs are due to developer errors?\",
            \"session_id\": \"$SESSION_ID\"
        }
    }"
    
} | ./target/release/lux-mcp 2>/dev/null > "$SESSION_FILE"

echo -e "\n2. Second call - continue with session_id:"
cat "$SESSION_FILE" | grep -A200 '"id":2' | jq -r '.result.content[0].text' | head -20

# Test with new_session flag
rm -f "$SESSION_FILE"
SESSION_FILE=$(mktemp)

{
    send_request 1 "initialize" '{
        "protocolVersion": "2024-11-05",
        "capabilities": {},
        "clientInfo": {"name": "test-client", "version": "1.0.0"}
    }'
    
    echo '{"jsonrpc": "2.0", "method": "notifications/initialized"}'
    
    # Test 3: Force new session
    send_request 2 "tools/call" '{
        "name": "biased_reasoning",
        "arguments": {
            "query": "What are the potential biases in assuming all software bugs are due to developer errors?",
            "new_session": true
        }
    }'
    
} | ./target/release/lux-mcp 2>/dev/null > "$SESSION_FILE"

echo -e "\n3. Test new_session flag - should create different session:"
NEW_SESSION_RESPONSE=$(cat "$SESSION_FILE" | grep -A200 '"id":2' | head -n 200)
echo "$NEW_SESSION_RESPONSE" | jq -r '.result.content[0].text' | head -10

# Extract new session ID
NEW_SESSION_ID=$(echo "$NEW_SESSION_RESPONSE" | jq -r '.result.content[0].text' | grep -oE 'Session ID: bias_[a-f0-9]+' | cut -d' ' -f3)
echo -e "\nNew session_id: $NEW_SESSION_ID"

if [ "$SESSION_ID" = "$NEW_SESSION_ID" ]; then
    echo "❌ ERROR: new_session flag didn't create a new session!"
else
    echo "✅ new_session flag worked correctly"
fi

# Clean up
rm -f "$SESSION_FILE"

echo -e "\n✅ Test complete!"