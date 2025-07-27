#!/bin/bash

# Test Monitor Integration - Circular Reasoning Detection

echo "=== Testing MetacognitiveMonitor Integration ==="
echo

# Build if needed
if [ ! -f target/release/lux-mcp ]; then
    echo "Building Lux MCP..."
    cargo build --release || exit 1
fi

# Source environment
export $(cat .env | grep -v '^#' | xargs)

echo "1. Testing Circular Reasoning Detection:"
echo "   Query designed to trigger circular reasoning..."
echo

# This query should trigger circular reasoning detection
echo '{
  "jsonrpc": "2.0",
  "method": "initialize",
  "params": {"capabilities": {}},
  "id": 1
}' | ./target/release/lux-mcp

sleep 1

echo '{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "traced_reasoning",
    "arguments": {
      "query": "To understand recursion, first you need to understand recursion. What is recursion?",
      "max_steps": 5,
      "guardrails": {
        "circular_reasoning_detection": true
      }
    }
  },
  "id": 2
}' | ./target/release/lux-mcp

echo -e "\n\n2. Testing Distractor Fixation Detection:"
echo "   Query designed to drift from original topic..."
echo

# This should trigger distractor fixation
echo '{
  "jsonrpc": "2.0",
  "method": "initialize",
  "params": {"capabilities": {}},
  "id": 3
}' | ./target/release/lux-mcp

sleep 1

echo '{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "traced_reasoning",
    "arguments": {
      "query": "Explain TCP/IP networking",
      "max_steps": 5,
      "guardrails": {
        "semantic_drift_check": true,
        "semantic_drift_threshold": 0.3
      }
    }
  },
  "id": 4
}' | ./target/release/lux-mcp

echo -e "\n\n=== Test Complete ==="
echo "Look for intervention messages in the output above."
echo "Successful detection will show interventions for circular reasoning or semantic drift."