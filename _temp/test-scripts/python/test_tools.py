#!/usr/bin/env python3
import json
import subprocess
import time
import os
import sys

def test_mcp_tool(tool_name, arguments):
    """Test an MCP tool with proper initialization"""
    
    # Set up environment
    env = os.environ.copy()
    env['OPENAI_API_KEY'] = os.environ.get('OPENAI_API_KEY', 'test-key')
    env['OPENROUTER_API_KEY'] = os.environ.get('OPENROUTER_API_KEY', 'test-key')
    env['RUST_LOG'] = 'info'
    
    # Start the server
    proc = subprocess.Popen(
        ['./target/release/lux-mcp'],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        env=env
    )
    
    try:
        # Send initialize request
        init_request = {
            "jsonrpc": "2.0",
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "clientInfo": {
                    "name": "test-client",
                    "version": "1.0.0"
                }
            },
            "id": 1
        }
        
        proc.stdin.write(json.dumps(init_request) + '\n')
        proc.stdin.flush()
        
        # Read initialize response
        response = proc.stdout.readline()
        init_result = json.loads(response)
        print(f"Initialize response: {json.dumps(init_result, indent=2)}")
        
        # Call the tool
        tool_request = {
            "jsonrpc": "2.0",
            "method": "tools/call",
            "params": {
                "name": tool_name,
                "arguments": arguments
            },
            "id": 2
        }
        
        proc.stdin.write(json.dumps(tool_request) + '\n')
        proc.stdin.flush()
        
        # Read tool response with timeout
        response = proc.stdout.readline()
        if response:
            result = json.loads(response)
            print(f"\n{tool_name} response: {json.dumps(result, indent=2)}")
            return result
        else:
            print(f"No response for {tool_name}")
            return None
            
    finally:
        proc.terminate()
        proc.wait()

# Test each tool
print("=== Testing Lux MCP Tools ===\n")

# Test 1: confer (chat)
print("1. Testing 'confer' tool:")
test_mcp_tool("confer", {"message": "Say hello in one word"})

# Test 2: traced_reasoning
print("\n2. Testing 'traced_reasoning' tool:")
test_mcp_tool("traced_reasoning", {
    "thought": "What is 2+2?",
    "thought_number": 1,
    "total_thoughts": 1,
    "next_thought_needed": False
})

# Test 3: biased_reasoning
print("\n3. Testing 'biased_reasoning' tool:")
test_mcp_tool("biased_reasoning", {
    "query": "Is the sky blue?"
})

# Test 4: illumination_status
print("\n4. Testing 'illumination_status' tool:")
test_mcp_tool("illumination_status", {})