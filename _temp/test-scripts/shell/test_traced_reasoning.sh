#!/bin/bash

# Test script for traced_reasoning tool

echo "Testing traced_reasoning tool..."
echo "================================"

# Load environment variables
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
fi

# Test with a simple query
echo -e "\nTest 1: Simple reasoning query"
echo '{"query": "What is the capital of France and why is it significant?", "model": "gpt4.1", "max_steps": 5}' | \
    cargo run --quiet | \
    jq -r '.response'

# Test with guardrails configuration
echo -e "\n\nTest 2: Reasoning with custom guardrails"
echo '{
    "query": "Explain the concept of recursion in programming", 
    "model": "o3",
    "max_steps": 7,
    "temperature": 0.5,
    "guardrails": {
        "semantic_drift_check": true,
        "perplexity_monitoring": true,
        "circular_reasoning_detection": true
    }
}' | \
    cargo run --quiet | \
    jq -r '.response'

# Test with OpenRouter model
echo -e "\n\nTest 3: Reasoning with OpenRouter model"
echo '{
    "query": "What are the implications of quantum computing?",
    "model": "claude",
    "max_steps": 8
}' | \
    cargo run --quiet | \
    jq -r '.response'

echo -e "\n\nAll tests completed!"