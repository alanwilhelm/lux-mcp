#!/bin/bash

# Source environment
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
fi

echo "Testing biased_reasoning with fixed o4-mini token limits..."
echo "==========================================================="

# Run the tool via cargo with the same parameters the user used
echo '{
    "jsonrpc": "2.0",
    "method": "tools/call",
    "params": {
        "name": "biased_reasoning",
        "arguments": {
            "query": "plan a similar system for our stripe payments",
            "primary_model": "gpt-4",
            "verifier_model": "o4-mini",
            "max_steps": 2
        }
    },
    "id": 1
}' | cargo run --release 2>&1 | jq -r '.result.content[0].text // .error.message' | head -100

echo -e "\n\nTesting complete! Check if the output shows detailed process logs."