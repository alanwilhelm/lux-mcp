#!/bin/bash

source .env

echo "Testing OpenAI models directly..."
echo "================================="

# Test o3-pro with chat completions
echo -e "\n1. Testing o3-pro with chat completions API:"
curl -s https://api.openai.com/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -d '{
    "model": "o3-pro",
    "messages": [{"role": "user", "content": "Say hello"}],
    "max_completion_tokens": 20
  }' | python3 -m json.tool | head -20

# Test o3-pro with completions API
echo -e "\n2. Testing o3-pro with completions API:"
curl -s https://api.openai.com/v1/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -d '{
    "model": "o3-pro",
    "prompt": "Say hello",
    "max_completion_tokens": 20
  }' | python3 -m json.tool | head -20

# Test o4-mini with chat completions
echo -e "\n3. Testing o4-mini with chat completions API:"
curl -s https://api.openai.com/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -d '{
    "model": "o4-mini",
    "messages": [{"role": "user", "content": "Say hello"}],
    "max_completion_tokens": 20
  }' | python3 -m json.tool | head -30