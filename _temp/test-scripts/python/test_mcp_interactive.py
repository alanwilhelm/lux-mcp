#!/usr/bin/env python3
"""
Interactive MCP protocol test that properly handles initialization
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

def main():
    # Set up environment
    env = os.environ.copy()
    env['OPENAI_API_KEY'] = os.environ.get('OPENAI_API_KEY', '')
    env['LUX_DEFAULT_CHAT_MODEL'] = 'gpt-4-turbo-preview'
    env['LUX_DEFAULT_REASONING_MODEL'] = 'o3-pro'
    env['LUX_DEFAULT_BIAS_CHECKER_MODEL'] = 'o4-mini'
    env['RUST_LOG'] = 'info'
    
    print("Starting MCP server...")
    proc = subprocess.Popen(
        ['./target/release/lux-mcp'],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        env=env,
        bufsize=1
    )
    
    # Give server time to start
    time.sleep(0.5)
    
    try:
        # Step 1: Send initialize request
        print("\n1. Sending initialize request...")
        send_message(proc, {
            "jsonrpc": "2.0",
            "method": "initialize",
            "params": {
                "protocolVersion": "1.0",
                "clientInfo": {
                    "name": "test-client",
                    "version": "1.0"
                }
            },
            "id": 1
        })
        
        # Read initialize response
        response = read_response(proc)
        if response and 'result' in response:
            print("✓ Initialize successful")
        else:
            print("✗ Initialize failed")
            return
        
        # Step 2: Send initialized notification
        print("\n2. Sending initialized notification...")
        send_message(proc, {
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        })
        
        # Give server time to process
        time.sleep(0.1)
        
        # Step 3: Now we can call tools
        print("\n3. Calling confer tool...")
        send_message(proc, {
            "jsonrpc": "2.0",
            "method": "tools/call",
            "params": {
                "name": "confer",
                "arguments": {
                    "message": "Hello, can you hear me?"
                }
            },
            "id": 2
        })
        
        # Read tool response
        response = read_response(proc)
        if response and 'result' in response:
            print("✓ Tool call successful")
            print(f"Response: {response['result']}")
        else:
            print("✗ Tool call failed")
            if response and 'error' in response:
                print(f"Error: {response['error']}")
        
    finally:
        # Clean up
        proc.terminate()
        proc.wait()
        
        # Print stderr for debugging
        stderr = proc.stderr.read()
        if stderr:
            print("\nServer logs:")
            print(stderr)

if __name__ == "__main__":
    main()