#!/bin/bash

echo "Testing server without .env file..."

# Temporarily rename .env
if [ -f .env ]; then
    mv .env .env.backup
    echo "Moved .env to .env.backup"
fi

# Start server and send a simple request
echo "Starting server..."
echo '{
  "jsonrpc": "2.0",
  "method": "initialize",
  "params": {
    "protocolVersion": "2024-11-05",
    "capabilities": {}
  },
  "id": 1
}' | ./target/release/lux-mcp 2>&1 | head -20

# Restore .env
if [ -f .env.backup ]; then
    mv .env.backup .env
    echo "Restored .env file"
fi