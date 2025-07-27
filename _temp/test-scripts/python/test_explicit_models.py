#!/usr/bin/env python3
"""
Test o3-pro and o4-mini by explicitly specifying the model
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
    
    # Test with confer tool, explicitly specifying model
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
            return
        
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
                    "message": "Hello! Please respond with 'Yes, I can hear you.'",
                    "model": model_name,  # Explicitly specify the model
                    "max_tokens": 30
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
            if any(x in line for x in ['Using', 'Model:', 'API', 'completions', 'error', 'Error']):
                print(f"  {line}")

def main():
    print("Testing o3-pro and o4-mini with explicit model specification")
    
    # Load .env to get API keys
    from dotenv import load_dotenv
    load_dotenv()
    
    # Test each model
    test_model("o3-pro")
    test_model("o4-mini")
    test_model("gpt-4o")  # Control test

if __name__ == "__main__":
    # Check if python-dotenv is installed
    try:
        import dotenv
    except ImportError:
        print("Installing python-dotenv...")
        subprocess.check_call([sys.executable, "-m", "pip", "install", "python-dotenv"])
        print("Please run the script again.")
        sys.exit(1)
    
    main()