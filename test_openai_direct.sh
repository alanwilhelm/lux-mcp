#!/bin/bash

# Test OpenAI API directly to verify credentials and model access

echo "Testing OpenAI API directly..."
echo "======================="

# Check if API key is set
if [ -z "$OPENAI_API_KEY" ]; then
    echo "ERROR: OPENAI_API_KEY is not set"
    exit 1
fi

echo "API Key is set (showing first 10 chars): ${OPENAI_API_KEY:0:10}..."
echo

# Test with gpt-4-turbo-preview
echo "Testing gpt-4-turbo-preview..."
curl -s https://api.openai.com/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -d '{
    "model": "gpt-4-turbo-preview",
    "messages": [{"role": "user", "content": "Say hello"}],
    "temperature": 0.7,
    "max_tokens": 100
  }' | jq . 2>/dev/null || echo "Failed to parse response"

echo
echo "Testing o3-pro..."
curl -s https://api.openai.com/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -d '{
    "model": "o3-pro",
    "messages": [{"role": "user", "content": "Say hello"}],
    "temperature": 0.7,
    "max_tokens": 100
  }' | jq . 2>/dev/null || echo "Failed to parse response"

echo
echo "Testing o4-mini..."
curl -s https://api.openai.com/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -d '{
    "model": "o4-mini",
    "messages": [{"role": "user", "content": "Say hello"}],
    "temperature": 0.7,
    "max_tokens": 100
  }' | jq . 2>/dev/null || echo "Failed to parse response"