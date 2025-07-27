#!/bin/bash

source .env

echo "Testing o3-pro and o4-mini models directly..."
echo "============================================="

# Test o3-pro
echo -e "\n1. Testing o3-pro model:"
curl -s https://api.openai.com/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -d '{
    "model": "o3-pro",
    "messages": [{"role": "user", "content": "Say hello"}],
    "max_tokens": 10
  }' | python3 -m json.tool | head -20

# Test o4-mini  
echo -e "\n2. Testing o4-mini model:"
curl -s https://api.openai.com/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -d '{
    "model": "o4-mini",
    "messages": [{"role": "user", "content": "Say hello"}],
    "max_tokens": 10
  }' | python3 -m json.tool | head -20

echo -e "\n3. Updating .env to use o3-pro and o4-mini..."
sed -i '' 's/LUX_DEFAULT_CHAT_MODEL=.*/LUX_DEFAULT_CHAT_MODEL=o3-pro/' .env
sed -i '' 's/LUX_DEFAULT_REASONING_MODEL=.*/LUX_DEFAULT_REASONING_MODEL=o3-pro/' .env
sed -i '' 's/LUX_DEFAULT_BIAS_CHECKER_MODEL=.*/LUX_DEFAULT_BIAS_CHECKER_MODEL=o4-mini/' .env

echo "Updated .env file:"
grep "LUX_DEFAULT" .env