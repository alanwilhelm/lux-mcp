#!/bin/bash

# Test the Lux MCP server

# Build in release mode
echo "Building Lux MCP server..."
cargo build --release

# Test initialize request
echo -e "\n\nTesting initialize request:"
echo '{"jsonrpc":"2.0","method":"initialize","params":{},"id":1}' | cargo run --release

# Test list tools request
echo -e "\n\nTesting list tools:"
echo '{"jsonrpc":"2.0","method":"tools/list","params":{},"id":2}' | cargo run --release

# Test lux_chat tool
echo -e "\n\nTesting lux_chat tool:"
echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"lux_chat","arguments":{"message":"What is 2+2?","model":"gpt4"}},"id":3}' | cargo run --release

# Test traced_reasoning tool
echo -e "\n\nTesting traced_reasoning tool:"
echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"traced_reasoning","arguments":{"thought":"I need to understand this complex system architecture","thought_number":1,"total_thoughts":5}},"id":4}' | cargo run --release

# Test illumination status
echo -e "\n\nTesting illumination status:"
echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"illumination_status","arguments":{}},"id":5}' | cargo run --release