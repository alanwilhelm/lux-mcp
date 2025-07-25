#!/bin/bash

# Test the Lux MCP chat functionality
# This script tests the chat tool without needing API keys

echo "Testing Lux MCP Chat Tool"
echo "========================="

# First, check if .env exists
if [ ! -f .env ]; then
    echo "Creating .env file from example..."
    cp .env.example .env
    echo "Please edit .env and add your API keys, then run this script again."
    exit 1
fi

# Source the .env file
export $(cat .env | grep -v '^#' | xargs)

# Check if API keys are set
if [ -z "$OPENAI_API_KEY" ] && [ -z "$OPENROUTER_API_KEY" ]; then
    echo "Error: No API keys found in .env"
    echo "Please set OPENAI_API_KEY or OPENROUTER_API_KEY"
    exit 1
fi

# Build if needed
if [ ! -f target/release/lux-mcp ]; then
    echo "Building Lux MCP..."
    cargo build --release || exit 1
fi

echo -e "\n1. Testing simple chat with default model:"
echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"lux_chat","arguments":{"message":"Say hello in one word"}},"id":1}'
echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"lux_chat","arguments":{"message":"Say hello in one word"}},"id":1}' | ./target/release/lux-mcp

echo -e "\n\n2. Testing with model alias 'mini':"
echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"lux_chat","arguments":{"message":"What is 2+2?","model":"mini"}},"id":2}'
echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"lux_chat","arguments":{"message":"What is 2+2?","model":"mini"}},"id":2}' | ./target/release/lux-mcp

echo -e "\n\n3. Testing with temperature control:"
echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"lux_chat","arguments":{"message":"Write a haiku about coding","model":"gpt4","temperature":0.9}},"id":3}'
echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"lux_chat","arguments":{"message":"Write a haiku about coding","model":"gpt4","temperature":0.9}},"id":3}' | ./target/release/lux-mcp

echo -e "\n\nChat tool testing complete!"