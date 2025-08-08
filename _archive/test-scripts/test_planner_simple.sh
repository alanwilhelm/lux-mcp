#!/bin/bash

echo "🧪 Simple planner synthesis test..."

# Enable debug logging to see synthesis operations
export RUST_LOG=debug
export DATABASE_URL="postgres://lux_user:lux_password@localhost/lux_mcp"

# Create a simple request
cat << 'EOF' | ./target/release/lux-mcp 2>&1 | grep -E "(Synthesis|synthesis|Planning|PLANNING|confidence|insight|action)" | head -30
{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "test", "version": "1.0"}}}
{"jsonrpc": "2.0", "method": "notifications/initialized"}
{"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {"name": "planner", "arguments": {"step": "Design a REST API", "step_number": 1, "total_steps": 3, "next_step_required": true}}}
EOF