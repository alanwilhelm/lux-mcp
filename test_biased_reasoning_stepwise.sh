#!/bin/bash

# Test script for biased_reasoning step-by-step functionality

echo "Testing biased_reasoning step-by-step operation..."

# Set test query
QUERY="What are the potential biases in assuming all software bugs are due to developer errors?"

echo -e "\n1. First call - should create new session:"
SESSION_RESPONSE=$(echo '{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "biased_reasoning",
    "arguments": {
      "query": "'"$QUERY"'"
    }
  }
}' | ./target/release/lux-mcp 2>/dev/null | grep -A100 '"result"' | jq -r '.result.content[] | select(.type == "text") | .text')

echo "$SESSION_RESPONSE"

# Extract session ID from the response
SESSION_ID=$(echo "$SESSION_RESPONSE" | grep -oE 'bias_[a-f0-9]+' | head -1)
echo -e "\nExtracted session_id: $SESSION_ID"

if [ -z "$SESSION_ID" ]; then
    echo "ERROR: No session_id found in response!"
    exit 1
fi

echo -e "\n2. Second call - continue with session_id:"
echo '{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "biased_reasoning",
    "arguments": {
      "query": "'"$QUERY"'",
      "session_id": "'"$SESSION_ID"'"
    }
  }
}' | ./target/release/lux-mcp 2>/dev/null | grep -A100 '"result"' | jq -r '.result.content[] | select(.type == "text") | .text'

echo -e "\n3. Third call - continue reasoning:"
echo '{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "biased_reasoning",
    "arguments": {
      "query": "'"$QUERY"'",
      "session_id": "'"$SESSION_ID"'"
    }
  }
}' | ./target/release/lux-mcp 2>/dev/null | grep -A100 '"result"' | jq -r '.result.content[] | select(.type == "text") | .text'

echo -e "\n4. Test new_session flag - should create different session:"
echo '{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "tools/call",
  "params": {
    "name": "biased_reasoning",
    "arguments": {
      "query": "'"$QUERY"'",
      "new_session": true
    }
  }
}' | ./target/release/lux-mcp 2>/dev/null | grep -A100 '"result"' | jq -r '.result.content[] | select(.type == "text") | .text'

echo -e "\n5. Test deterministic ID - same query without session_id:"
echo '{
  "jsonrpc": "2.0",
  "id": 5,
  "method": "tools/call",
  "params": {
    "name": "biased_reasoning",
    "arguments": {
      "query": "'"$QUERY"'"
    }
  }
}' | ./target/release/lux-mcp 2>/dev/null | grep -A100 '"result"' | jq -r '.result.content[] | select(.type == "text") | .text'

echo -e "\n✅ Test complete!"