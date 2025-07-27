#!/usr/bin/env python3
import json
import subprocess
import time
import os

def test_traced_reasoning():
    print("Testing traced_reasoning model display...")
    
    # Start the MCP server
    env = os.environ.copy()
    process = subprocess.Popen(
        ['./target/release/lux-mcp'],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        env=env
    )
    
    try:
        # Send initialize
        request = {
            "jsonrpc": "2.0",
            "method": "initialize",
            "params": {"capabilities": {}},
            "id": 1
        }
        process.stdin.write(json.dumps(request) + '\n')
        process.stdin.flush()
        
        # Read response
        response = process.stdout.readline()
        print(f"Initialize response: {response}")
        
        # Send traced_reasoning call
        request = {
            "jsonrpc": "2.0",
            "method": "tools/call",
            "params": {
                "name": "traced_reasoning",
                "arguments": {
                    "thought": "What is consciousness?",
                    "thought_number": 1,
                    "total_thoughts": 3,
                    "next_thought_needed": True,
                    "model": "gpt-4o"
                }
            },
            "id": 2
        }
        process.stdin.write(json.dumps(request) + '\n')
        process.stdin.flush()
        
        # Read response
        response = process.stdout.readline()
        result = json.loads(response)
        
        if 'result' in result and 'content' in result['result']:
            content = result['result']['content'][0]['text']
            print("\n=== TRACED REASONING OUTPUT ===")
            print(content)
            
            # Check if "Model:" appears in the output
            if "Model:" in content:
                print("\n✅ SUCCESS: Model is displayed in the output!")
                # Extract the model line
                for line in content.split('\n'):
                    if "Model:" in line:
                        print(f"Found: {line.strip()}")
            else:
                print("\n❌ FAIL: Model is NOT displayed in the output!")
        
    finally:
        process.terminate()
        process.wait()

def test_biased_reasoning():
    print("\n\nTesting biased_reasoning model display...")
    
    # Start the MCP server
    env = os.environ.copy()
    process = subprocess.Popen(
        ['./target/release/lux-mcp'],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        env=env
    )
    
    try:
        # Send initialize
        request = {
            "jsonrpc": "2.0",
            "method": "initialize",
            "params": {"capabilities": {}},
            "id": 1
        }
        process.stdin.write(json.dumps(request) + '\n')
        process.stdin.flush()
        
        # Read response
        response = process.stdout.readline()
        
        # Send biased_reasoning call
        request = {
            "jsonrpc": "2.0",
            "method": "tools/call",
            "params": {
                "name": "biased_reasoning",
                "arguments": {
                    "query": "Should we implement feature X?",
                    "primary_model": "gpt-4",
                    "verifier_model": "o4-mini",
                    "max_steps": 2
                }
            },
            "id": 2
        }
        process.stdin.write(json.dumps(request) + '\n')
        process.stdin.flush()
        
        # Read response
        response = process.stdout.readline()
        result = json.loads(response)
        
        if 'result' in result and 'content' in result['result']:
            content = result['result']['content'][0]['text']
            print("\n=== BIASED REASONING OUTPUT (first 50 lines) ===")
            lines = content.split('\n')[:50]
            for line in lines:
                print(line)
            
            # Check if models are displayed
            if "Models Used:" in content:
                print("\n✅ SUCCESS: Models are displayed in the output!")
            else:
                print("\n❌ FAIL: Models are NOT displayed in the output!")
        
    finally:
        process.terminate()
        process.wait()

if __name__ == "__main__":
    test_traced_reasoning()
    test_biased_reasoning()