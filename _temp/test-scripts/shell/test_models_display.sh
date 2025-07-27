#!/bin/bash

echo "=== Testing Model Display in traced_reasoning ==="

# Test with explicit model
echo '{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "traced_reasoning",
    "arguments": {
      "thought": "What is consciousness?",
      "thought_number": 1,
      "total_thoughts": 3,
      "next_thought_needed": true,
      "model": "gpt-4o",
      "temperature": 0.7
    }
  }
}' | RUST_LOG=info ./target/release/lux-mcp 2>&1 | grep -E "(Model:|model_used:|Using specified model)"

echo -e "\n=== Testing Model Display in biased_reasoning ==="

# Test with explicit models
echo '{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "biased_reasoning",
    "arguments": {
      "query": "Should we implement feature X?",
      "primary_model": "gpt-4",
      "verifier_model": "o4-mini",
      "max_steps": 2
    }
  }
}' | ./target/release/lux-mcp 2>&1 | grep -E "(Models Used:|Model:)" | head -10