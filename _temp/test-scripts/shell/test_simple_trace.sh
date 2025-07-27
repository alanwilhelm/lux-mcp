#!/bin/bash

# Simple test of traced_reasoning
echo "Testing traced_reasoning with model specified..."
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
      "model": "gpt-4o"
    }
  }
}' | RUST_LOG=info ./target/release/lux-mcp 2>&1 | head -50