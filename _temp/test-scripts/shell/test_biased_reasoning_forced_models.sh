#!/bin/bash

echo "Testing biased_reasoning with forced model configuration"
echo "========================================================"
echo ""
echo "This test verifies that biased_reasoning ALWAYS uses:"
echo "- Primary: LUX_DEFAULT_REASONING_MODEL (o3-pro)"
echo "- Verifier: LUX_DEFAULT_BIAS_CHECKER_MODEL (o4-mini)"
echo ""
echo "Even when user tries to override with other models..."
echo ""

# Set up environment with expected defaults
export OPENAI_API_KEY="${OPENAI_API_KEY}"
export LUX_DEFAULT_REASONING_MODEL="o3-pro"
export LUX_DEFAULT_BIAS_CHECKER_MODEL="o4-mini"
export RUST_LOG="info"

# Build if needed
if [ ! -f ./target/release/lux-mcp ]; then
    echo "Building lux-mcp..."
    cargo build --release
fi

# Test with user trying to override models
echo "Test: User tries to override with claude-3-5-sonnet as verifier"
echo "--------------------------------------------------------------"

cat << 'EOF' | ./target/release/lux-mcp 2>&1 | grep -E "(Starting biased reasoning|Note: biased_reasoning always uses)"
{
  "jsonrpc": "2.0",
  "method": "initialize",
  "params": {
    "protocolVersion": "1.0.0",
    "capabilities": {},
    "clientInfo": {
      "name": "test-client",
      "version": "1.0.0"
    }
  },
  "id": 1
}
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "biased_reasoning",
    "arguments": {
      "query": "Test query",
      "primary_model": "gpt-4",
      "verifier_model": "claude-3-5-sonnet-20241022",
      "max_steps": 1
    }
  },
  "id": 2
}
EOF

echo ""
echo "Expected behavior:"
echo "1. Log should show: 'Note: biased_reasoning always uses configured defaults'"
echo "2. Models used should be o3-pro and o4-mini regardless of request parameters"