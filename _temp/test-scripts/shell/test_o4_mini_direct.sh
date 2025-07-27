#!/bin/bash

# Source environment
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
fi

echo "Testing o4-mini model directly with OpenAI API..."
echo "================================================="

# Test o4-mini with the new chat completions API
curl -s https://api.openai.com/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -d '{
    "model": "o4-mini",
    "messages": [
      {
        "role": "system",
        "content": "You are a critical thinking expert who identifies biases."
      },
      {
        "role": "user", 
        "content": "Analyze this thought for bias: We should use Redis because everyone uses it."
      }
    ],
    "max_completion_tokens": 200
  }' | jq .

echo -e "\n\nNow testing with o4-mini-2025-04-16..."
echo "======================================="

# Also test with the full model name
curl -s https://api.openai.com/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -d '{
    "model": "o4-mini-2025-04-16",
    "messages": [
      {
        "role": "system",
        "content": "You are a critical thinking expert who identifies biases."
      },
      {
        "role": "user",
        "content": "Analyze this thought for bias: We should use Redis because everyone uses it."
      }
    ],
    "max_completion_tokens": 200
  }' | jq .