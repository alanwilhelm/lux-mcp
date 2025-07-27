#!/usr/bin/env python3
"""
Test o3-pro and o4-mini models with the updated chat completions API
"""
import json
import subprocess
import sys
import os
import time

def send_message(proc, message):
    """Send a message to the MCP server"""
    json_msg = json.dumps(message)
    print(f"→ Sending: {json_msg}")
    proc.stdin.write(json_msg + '\n')
    proc.stdin.flush()
    
def read_response(proc):
    """Read a response from the MCP server"""
    response = proc.stdout.readline()
    if response:
        print(f"← Received: {response.strip()}")
        return json.loads(response)
    return None

def test_model(model_name):
    print(f"\n{'='*60}")
    print(f"Testing {model_name}")
    print(f"{'='*60}")
    
    # Start the MCP server
    env = os.environ.copy()
    proc = subprocess.Popen(
        ['./target/release/lux-mcp'],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        env=env,
        bufsize=1
    )
    
    time.sleep(0.5)
    
    try:
        # Initialize
        send_message(proc, {
            "jsonrpc": "2.0",
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "test-client",
                    "version": "1.0.0"
                }
            },
            "id": 1
        })
        
        response = read_response(proc)
        if response and 'result' in response:
            print("✓ Initialize successful")
        else:
            print("✗ Initialize failed")
            return False
        
        # Send initialized notification
        send_message(proc, {
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        })
        
        time.sleep(0.1)
        
        # Test confer with explicit model
        print(f"\nCalling confer with model={model_name}")
        send_message(proc, {
            "jsonrpc": "2.0",
            "method": "tools/call",
            "params": {
                "name": "confer",
                "arguments": {
                    "message": "Say 'Hello from OpenAI!' in exactly 5 words.",
                    "model": model_name,
                    "max_tokens": 20
                }
            },
            "id": 2
        })
        
        response = read_response(proc)
        if response and 'result' in response:
            print("✓ Tool call successful!")
            content = response['result'].get('content', [])
            if content:
                text = content[0].get('text', '')
                print(f"Response: {text}")
            return True
        elif response and 'error' in response:
            print("✗ Tool call failed!")
            print(f"Error: {response['error']}")
            return False
            
    finally:
        proc.terminate()
        proc.wait()
        
        # Show relevant logs
        stderr = proc.stderr.read()
        print("\nRelevant server logs:")
        for line in stderr.split('\n'):
            if any(x in line.lower() for x in ['model:', 'reasoning', 'o3', 'o4', 'api', 'error', 'chat']):
                print(f"  {line}")
    
    return False

def main():
    print("Testing o3-pro and o4-mini with chat completions API")
    
    # Test each model
    o3_success = test_model("o3-pro")
    o4_success = test_model("o4-mini")
    gpt4_success = test_model("gpt-4o")  # Control test
    
    print(f"\n{'='*60}")
    print("RESULTS:")
    print(f"  o3-pro:  {'✓ SUCCESS' if o3_success else '✗ FAILED'}")
    print(f"  o4-mini: {'✓ SUCCESS' if o4_success else '✗ FAILED'}")
    print(f"  gpt-4o:  {'✓ SUCCESS' if gpt4_success else '✗ FAILED'}")
    print(f"{'='*60}")

if __name__ == "__main__":
    main()