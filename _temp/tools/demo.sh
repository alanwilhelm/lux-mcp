#!/bin/bash

# Lux MCP Demo Script
# Demonstrates all three tools with example queries

echo "🔦 Lux MCP Demo"
echo "==============="
echo

# Load environment variables
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
fi

# Check for API keys
if [ -z "$OPENAI_API_KEY" ] && [ -z "$OPENROUTER_API_KEY" ]; then
    echo "❌ Error: No API keys found in .env file"
    echo "Please add OPENAI_API_KEY or OPENROUTER_API_KEY to your .env file"
    exit 1
fi

echo "✓ API keys loaded"
echo

# Demo 1: Chat Tool
echo "📝 Demo 1: Chat Tool"
echo "===================="
echo "Query: 'What are the key benefits of Rust for systems programming?'"
echo

REQUEST='{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "lux:chat",
    "arguments": {
      "message": "What are the key benefits of Rust for systems programming?",
      "model": "gpt4.1",
      "temperature": 0.7
    }
  },
  "id": "demo-chat"
}'

echo "$REQUEST" | ./target/release/lux-mcp | jq '.result.content[0].text' -r

echo
echo "Press Enter to continue..."
read

# Demo 2: Traced Reasoning
echo "🧠 Demo 2: Traced Reasoning"
echo "==========================="
echo "Query: 'Design a simple rate limiter for a REST API'"
echo

REQUEST='{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "traced_reasoning",
    "arguments": {
      "query": "Design a simple rate limiter for a REST API. Consider different strategies and their trade-offs.",
      "model": "o3",
      "max_thinking_steps": 5
    }
  },
  "id": "demo-traced"
}'

echo "$REQUEST" | ./target/release/lux-mcp | jq '.result.content[0].text' -r

echo
echo "Press Enter to continue..."
read

# Demo 3: Biased Reasoning
echo "🔍 Demo 3: Biased Reasoning"
echo "==========================="
echo "Query: 'Should all software be open source?'"
echo

REQUEST='{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "biased_reasoning",
    "arguments": {
      "query": "Should all software be open source? Consider both perspectives.",
      "primary_model": "o3",
      "verifier_model": "claude",
      "max_analysis_rounds": 2
    }
  },
  "id": "demo-biased"
}'

echo "$REQUEST" | ./target/release/lux-mcp | jq '.result.content[0].text' -r

echo
echo "✨ Demo complete!"
echo
echo "These examples show:"
echo "• Chat: Simple Q&A with configurable models"
echo "• Traced Reasoning: Step-by-step transparent thinking"
echo "• Biased Reasoning: Dual-model verification for balanced analysis"