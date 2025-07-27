#!/bin/bash

# Simple test - just send one line at a time and see what happens
echo "Testing simple initialization..."

# Just send the initialize request
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"clientInfo":{"name":"test","version":"1.0"},"capabilities":{}}}' | \
    RUST_LOG=debug ./target/release/lux-mcp 2>&1 | head -20