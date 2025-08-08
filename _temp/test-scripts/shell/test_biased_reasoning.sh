#!/bin/bash

# Test script for biased_reasoning tool

echo "Testing biased_reasoning tool..."
echo "================================"

# Load environment variables
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
fi

# Function to test and display results
test_scenario() {
    local name="$1"
    local request="$2"
    echo -e "\n\n=== Test: $name ==="
    echo "Request: $request"
    echo -e "\nRunning test...\n"
    
    # Run the test
    echo "$request" | cargo run --quiet 2>/dev/null | jq -r '.result.content[0].text // .error.message'
}

# Test 1: Simple reasoning with default models
test_scenario "Simple query with defaults" '{
    "jsonrpc": "2.0",
    "method": "tools/call",
    "params": {
        "name": "biased_reasoning",
        "arguments": {
            "query": "Is AI going to replace all human jobs?",
            "max_steps": 3
        }
    },
    "id": 1
}'

# Test 2: Complex reasoning with bias detection
test_scenario "Complex reasoning with likely bias" '{
    "jsonrpc": "2.0",
    "method": "tools/call",
    "params": {
        "name": "biased_reasoning",
        "arguments": {
            "query": "Why is my favorite programming language obviously the best?",
            "max_steps": 4,
            "temperature": 0.7
        }
    },
    "id": 2
}'

# Test 3: Custom model configuration
test_scenario "Custom models (GPT-4.1 + mini)" '{
    "jsonrpc": "2.0",
    "method": "tools/call",
    "params": {
        "name": "biased_reasoning",
        "arguments": {
            "query": "What are the benefits and drawbacks of remote work?",
            "primary_model": "gpt4.1",
            "verifier_model": "mini",
            "max_steps": 3
        }
    },
    "id": 3
}'

# Test 4: OpenRouter model test
test_scenario "OpenRouter models" '{
    "jsonrpc": "2.0",
    "method": "tools/call",
    "params": {
        "name": "biased_reasoning",
        "arguments": {
            "query": "Explain the trolley problem and its ethical implications",
            "primary_model": "claude",
            "verifier_model": "o4-mini",
            "max_steps": 5,
            "bias_config": {
                "check_confirmation_bias": true,
                "check_anchoring_bias": true,
                "check_reasoning_errors": true,
                "bias_threshold": 0.6
            }
        }
    },
    "id": 4
}'

# Test 5: Edge case - very short reasoning
test_scenario "Minimal steps test" '{
    "jsonrpc": "2.0",
    "method": "tools/call",
    "params": {
        "name": "biased_reasoning",
        "arguments": {
            "query": "What is 2 + 2?",
            "max_steps": 1,
            "temperature": 0.1
        }
    },
    "id": 5
}'

echo -e "\n\nAll tests completed!"
echo "===================="
echo -e "\nNote: Check the output for:"
echo "- Bias detection working correctly"
echo "- Step-by-step verification by secondary model"
echo "- Corrected thoughts when bias is detected"
echo "- Overall quality assessment"
echo -e "\nIf you see API errors, check your .env file for:"
echo "- OPENAI_API_KEY"
echo "- OPENROUTER_API_KEY"
echo "- LUX_DEFAULT_REASONING_MODEL"
echo "- LUX_DEFAULT_BIAS_CHECKER_MODEL"