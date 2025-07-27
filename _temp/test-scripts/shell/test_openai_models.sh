#!/bin/bash

echo "Testing OpenAI models to find REAL model names..."
echo "=================================================="

# Load API key from .env
source .env

if [[ -z "$OPENAI_API_KEY" || "$OPENAI_API_KEY" == "sk-..." ]]; then
    echo "Error: Need a valid OPENAI_API_KEY in .env"
    exit 1
fi

# Models to test
models=(
    # GPT-4 variants
    "gpt-4"
    "gpt-4-turbo"
    "gpt-4-turbo-preview"
    "gpt-4-1106-preview" 
    "gpt-4-0125-preview"
    "gpt-4o"
    "gpt-4o-mini"
    
    # GPT-3.5 variants
    "gpt-3.5-turbo"
    "gpt-3.5-turbo-0125"
    
    # O-series models (what you tried)
    "o1"
    "o1-preview"
    "o1-mini"
    "o3"
    "o3-pro"
    "o4-mini"
)

echo "Testing each model with OpenAI API..."
echo

for model in "${models[@]}"; do
    echo -n "Testing $model... "
    
    response=$(curl -s -w "\n%{http_code}" https://api.openai.com/v1/chat/completions \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer $OPENAI_API_KEY" \
        -d '{
            "model": "'"$model"'",
            "messages": [{"role": "user", "content": "hi"}],
            "max_tokens": 1
        }')
    
    http_code=$(echo "$response" | tail -1)
    body=$(echo "$response" | sed '$d')
    
    if [[ "$http_code" == "200" ]]; then
        echo "✅ EXISTS!"
        actual_model=$(echo "$body" | grep -o '"model":"[^"]*"' | cut -d'"' -f4)
        echo "   Actual model: $actual_model"
    elif [[ "$http_code" == "404" ]]; then
        echo "❌ NOT FOUND"
    elif [[ "$http_code" == "401" ]]; then
        echo "❌ API KEY ERROR"
        break
    else
        echo "❌ Error $http_code"
        error_msg=$(echo "$body" | grep -o '"message":"[^"]*"' | cut -d'"' -f4)
        if [[ -n "$error_msg" ]]; then
            echo "   Error: $error_msg"
        fi
    fi
done

echo
echo "=================================================="
echo "VALID MODELS for your Claude Code config:"
echo
echo "Update your config with these REAL model names:"
echo '  "LUX_DEFAULT_CHAT_MODEL": "gpt-4-turbo-preview",'
echo '  "LUX_DEFAULT_REASONING_MODEL": "gpt-4-turbo-preview",'  
echo '  "LUX_DEFAULT_BIAS_CHECKER_MODEL": "gpt-4o-mini",'
echo
echo "Or use gpt-3.5-turbo for a cheaper option"