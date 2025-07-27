#!/usr/bin/env python3
import json
import subprocess
import os

# Set environment for OpenRouter
env = os.environ.copy()
env['LUX_DEFAULT_CHAT_MODEL'] = 'claude'
env['LUX_DEFAULT_REASONING_MODEL'] = 'gemini'
env['LUX_DEFAULT_BIAS_CHECKER_MODEL'] = 'flash'

# Start the server
print("Starting Lux MCP server...")
proc = subprocess.Popen(
    ['./target/release/lux-mcp'],
    stdin=subprocess.PIPE,
    stdout=subprocess.PIPE,
    stderr=subprocess.PIPE,
    env=env,
    text=True
)

def send_message(msg):
    """Send a message and read response"""
    print(f"\nSending: {json.dumps(msg)}")
    proc.stdin.write(json.dumps(msg) + '\n')
    proc.stdin.flush()
    
    # Try to read response
    response = proc.stdout.readline()
    if response:
        print(f"Response: {response.strip()}")
        return json.loads(response)
    return None

try:
    # Send initialize
    resp = send_message({
        "jsonrpc": "2.0",
        "method": "initialize",
        "params": {
            "protocolVersion": "0.1.0",
            "capabilities": {
                "tools": {}
            }
        },
        "id": 1
    })
    
    # Send tools/list
    resp = send_message({
        "jsonrpc": "2.0",
        "method": "tools/list",
        "params": {},
        "id": 2
    })
    
    # Try confer with explicit model
    resp = send_message({
        "jsonrpc": "2.0",
        "method": "tools/call",
        "params": {
            "name": "confer",
            "arguments": {
                "message": "Hello, testing with Claude model",
                "model": "claude"
            }
        },
        "id": 3
    })
    
    # Read any remaining output
    for line in proc.stdout:
        print(f"Additional output: {line.strip()}")
        
except Exception as e:
    print(f"Error: {e}")
finally:
    # Print stderr
    print("\nStderr output:")
    for line in proc.stderr:
        print(line.strip())
    
    proc.terminate()
    proc.wait()

print("\nTest complete!")