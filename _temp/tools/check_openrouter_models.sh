#!/bin/bash

# Script to check available models on OpenRouter
# Specifically looking for Gemini models

echo "Checking OpenRouter for available Gemini models..."
echo "================================================"

# Check if OPENROUTER_API_KEY is set
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
fi

if [ -z "$OPENROUTER_API_KEY" ]; then
    echo "Warning: OPENROUTER_API_KEY not set. Results may be limited."
fi

# Call OpenRouter API to get models
echo -e "\nFetching model list from OpenRouter...\n"

# Make the API call
response=$(curl -s https://openrouter.ai/api/v1/models \
  -H "Authorization: Bearer ${OPENROUTER_API_KEY:-}" \
  -H "Content-Type: application/json")

# Parse and filter for Gemini models
echo "Gemini Models Available on OpenRouter:"
echo "-------------------------------------"

# Use jq if available, otherwise use grep
if command -v jq &> /dev/null; then
    echo "$response" | jq -r '.data[] | select(.id | contains("gemini")) | "\(.id) - \(.name // .id)"' | sort
else
    # Fallback to grep/sed if jq not available
    echo "$response" | grep -o '"id":"[^"]*gemini[^"]*"' | sed 's/"id":"//g' | sed 's/"//g' | sort | uniq
fi

echo -e "\n\nAll Google Models (including Gemini, PaLM, etc.):"
echo "-----------------------------------------------"

if command -v jq &> /dev/null; then
    echo "$response" | jq -r '.data[] | select(.id | contains("google")) | "\(.id) - \(.name // .id) [Context: \(.context_length // "unknown")]"' | sort
else
    echo "$response" | grep -o '"id":"[^"]*google[^"]*"' | sed 's/"id":"//g' | sed 's/"//g' | sort | uniq
fi

# Also check for any models with "gemini" in the name but different ID pattern
echo -e "\n\nAdditional models with 'gemini' in name:"
echo "---------------------------------------"

if command -v jq &> /dev/null; then
    echo "$response" | jq -r '.data[] | select(.name // "" | ascii_downcase | contains("gemini")) | "\(.id) - \(.name // .id)"' | sort | uniq
fi

# Save full response for inspection
echo "$response" > openrouter_models_full.json
echo -e "\n\nFull response saved to: openrouter_models_full.json"
echo "You can inspect it with: jq '.data[] | select(.id | contains(\"gemini\"))' openrouter_models_full.json"