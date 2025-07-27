#!/usr/bin/env python3
"""
Proper MCP protocol test for rmcp
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
    
    # Check for API keys
    if not os.environ.get('OPENAI_API_KEY') and not os.environ.get('OPENROUTER_API_KEY'):
        print("Error: No API keys found!")
        print("Please set OPENAI_API_KEY or OPENROUTER_API_KEY environment variable")
        print("Example: export OPENAI_API_KEY='your-api-key-here'")
        sys.exit(1)
    
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
        # Step 1: Send initialize request with proper protocol version
        print("\n1. Sending initialize request...")
        send_message(proc, {
            "jsonrpc": "2.0",
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",  # Date format!
                "capabilities": {
                    "tools": {},
                    "prompts": {}
                },
                "clientInfo": {
                    "name": "test-client",
                    "version": "1.0.0"
                }
            },
            "id": 1
        })
        
        # Read initialize response
        response = read_response(proc)
        if response and 'result' in response:
            print("✓ Initialize successful")
            print(f"  Server: {response['result'].get('serverInfo', {})}")
            print(f"  Capabilities: {response['result'].get('capabilities', {})}")
        else:
            print("✗ Initialize failed")
            print(f"  Response: {response}")
            return
        
        # Step 2: Send initialized notification (with proper method name)
        print("\n2. Sending initialized notification...")
        send_message(proc, {
            "jsonrpc": "2.0",
            "method": "notifications/initialized"  # Full method name!
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
            print(f"  Response: {response['result']}")
        else:
            print("✗ Tool call failed")
            if response and 'error' in response:
                print(f"  Error: {response['error']}")
        
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