#!/bin/bash

source .env

echo "Testing o3-pro and o4-mini with completions API..."
echo "=================================================="

# Test o3-pro with completions API
echo -e "\n1. Testing o3-pro model with completions API:"
curl -s https://api.openai.com/v1/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -d '{
    "model": "o3-pro",
    "prompt": "User: Hello, can you hear me?\nAssistant:",
    "max_tokens": 50,
    "temperature": 0.7
  }' | python3 -m json.tool | head -30

# Test o4-mini with max_completion_tokens
echo -e "\n2. Testing o4-mini model with max_completion_tokens:"
curl -s https://api.openai.com/v1/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -d '{
    "model": "o4-mini",
    "prompt": "User: Hello, can you hear me?\nAssistant:",
    "max_completion_tokens": 50,
    "temperature": 0.7
  }' | python3 -m json.tool | head -30

echo -e "\n3. Testing lux-mcp with o3-pro and o4-mini..."
./test_rmcp_correct.py 2>&1 | tail -40