#!/bin/bash

# Source environment
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
fi

echo "Testing o3-pro with reasoning_effort=high..."
echo "==========================================="

# Enable debug logging to see the API request details
export RUST_LOG=info

# Test with o3-pro to see reasoning_effort in logs
echo '{
    "jsonrpc": "2.0",
    "method": "tools/call",
    "params": {
        "name": "biased_reasoning",
        "arguments": {
            "query": "What are the pros and cons of microservices architecture?",
            "primary_model": "o3-pro",
            "verifier_model": "gpt-4",
            "max_steps": 2
        }
    },
    "id": 1
}' | cargo run --release 2>&1 | grep -E "(reasoning_effort|Reasoning effort)" | head -10

echo -e "\n\nTest complete! Check logs above for 'Reasoning effort: Some(\"high\")'."