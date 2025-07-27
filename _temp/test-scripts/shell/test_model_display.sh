#!/bin/bash

echo "Testing model display in traced_reasoning and biased_reasoning tools..."

# Test traced_reasoning
echo -e "\n=== Testing traced_reasoning ==="
echo '{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "traced_reasoning",
    "arguments": {
      "thought": "What is consciousness and how does it emerge?",
      "thought_number": 1,
      "total_thoughts": 3,
      "next_thought_needed": true,
      "model": "gpt-4",
      "temperature": 0.7
    }
  }
}' | ./target/release/lux-mcp 2>&1 | grep -A 20 "Model:"

# Test biased_reasoning
echo -e "\n=== Testing biased_reasoning ==="
echo '{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "biased_reasoning",
    "arguments": {
      "query": "Should we implement a new feature?",
      "primary_model": "gpt-4",
      "verifier_model": "o4-mini",
      "max_steps": 3
    }
  }
}' | ./target/release/lux-mcp 2>&1 | grep -A 5 "Models Used:"