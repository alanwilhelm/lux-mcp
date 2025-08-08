#!/bin/bash

echo "🧪 Simple traced_reasoning synthesis test..."

# Enable info logging
export RUST_LOG=info

# Create a simple request and look for synthesis
cat << 'EOF' | ./target/release/lux-mcp 2>&1 | grep -A50 "REASONING THOUGHT" | grep -E "(Synthesis|Understanding|Confidence|Clarity|Insights)" | head -20
{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "test", "version": "1.0"}}}
{"jsonrpc": "2.0", "method": "notifications/initialized"}
{"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {"name": "traced_reasoning", "arguments": {"thought": "Analyze the trade-offs of using microservices", "thought_number": 1, "total_thoughts": 2, "next_thought_needed": true}}}
EOF