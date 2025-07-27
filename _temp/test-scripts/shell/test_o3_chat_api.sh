#!/bin/bash

source .env

echo "Testing o3-pro and o4-mini with chat completions API..."
echo "======================================================="

# Test o3-pro with chat completions API
echo -e "\n1. Testing o3-pro with chat completions API:"
response=$(curl -s -w "\nHTTP_STATUS:%{http_code}" https://api.openai.com/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -d '{
    "model": "o3-pro",
    "messages": [{"role": "user", "content": "Hello, can you hear me?"}],
    "max_tokens": 50
  }')

body=$(echo "$response" | sed -n '1,/^HTTP_STATUS:/p' | sed '$d')
status=$(echo "$response" | grep "HTTP_STATUS:" | cut -d: -f2)

echo "Status: $status"
echo "$body" | python3 -m json.tool | head -20

# Test o4-mini with chat completions API
echo -e "\n2. Testing o4-mini with chat completions API:"
response=$(curl -s -w "\nHTTP_STATUS:%{http_code}" https://api.openai.com/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -d '{
    "model": "o4-mini",
    "messages": [{"role": "user", "content": "Hello, can you hear me?"}],
    "max_tokens": 50
  }')

body=$(echo "$response" | sed -n '1,/^HTTP_STATUS:/p' | sed '$d')
status=$(echo "$response" | grep "HTTP_STATUS:" | cut -d: -f2)

echo "Status: $status"
echo "$body" | python3 -m json.tool | head -20

# Test o4-mini with max_completion_tokens
echo -e "\n3. Testing o4-mini with max_completion_tokens instead of max_tokens:"
response=$(curl -s -w "\nHTTP_STATUS:%{http_code}" https://api.openai.com/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -d '{
    "model": "o4-mini",
    "messages": [{"role": "user", "content": "Hello, can you hear me?"}],
    "max_completion_tokens": 50
  }')

body=$(echo "$response" | sed -n '1,/^HTTP_STATUS:/p' | sed '$d')
status=$(echo "$response" | grep "HTTP_STATUS:" | cut -d: -f2)

echo "Status: $status"
echo "$body" | python3 -m json.tool | head -20