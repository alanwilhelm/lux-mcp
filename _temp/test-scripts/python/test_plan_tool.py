#!/usr/bin/env python3
"""
Test the plan tool specifically with o3-pro
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

def test_plan_tool():
    print("Testing plan tool with o3-pro model...")
    print("="*60)
    
    # Start the MCP server
    env = os.environ.copy()
    env['RUST_LOG'] = 'debug'
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
            return
        
        # Send initialized notification
        send_message(proc, {
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        })
        
        time.sleep(0.1)
        
        # Test plan tool with a simple goal
        print("\nCalling plan tool...")
        send_message(proc, {
            "jsonrpc": "2.0",
            "method": "tools/call",
            "params": {
                "name": "plan",
                "arguments": {
                    "goal": "Create a simple hello world program",
                    "max_steps": 3
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
                print(f"\nResponse ({len(text)} chars):")
                print("-" * 40)
                print(text[:500] + "..." if len(text) > 500 else text)
                print("-" * 40)
        elif response and 'error' in response:
            print("✗ Tool call failed!")
            print(f"Error: {response['error']}")
            
    finally:
        proc.terminate()
        proc.wait()
        
        # Show relevant logs
        stderr = proc.stderr.read()
        print("\nRelevant server logs:")
        for line in stderr.split('\n'):
            if any(x in line.lower() for x in ['error', 'failed', 'plan', 'o3', 'responses']):
                print(f"  {line}")

if __name__ == "__main__":
    test_plan_tool()