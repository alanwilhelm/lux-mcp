#!/usr/bin/env python3
"""
Test o3-pro and o4-mini models with lux-mcp
"""
import json
import subprocess
import sys
import os
import time

def send_message(proc, message):
    """Send a message to the MCP server"""
    json_msg = json.dumps(message)
    print(f"→ Sending: {json_msg}", file=sys.stderr)
    proc.stdin.write(json_msg + '\n')
    proc.stdin.flush()
    
def read_response(proc):
    """Read a response from the MCP server"""
    response = proc.stdout.readline()
    if response:
        print(f"← Received: {response.strip()}", file=sys.stderr)
        return json.loads(response)
    return None

def test_model(model_name):
    """Test a specific model"""
    print(f"\n{'='*60}", file=sys.stderr)
    print(f"Testing {model_name}", file=sys.stderr)
    print(f"{'='*60}", file=sys.stderr)
    
    # Set up environment
    env = os.environ.copy()
    env['LUX_DEFAULT_CHAT_MODEL'] = model_name
    env['RUST_LOG'] = 'info'
    
    print(f"Starting MCP server with model: {model_name}", file=sys.stderr)
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
                "capabilities": {}
            },
            "id": 1
        })
        
        response = read_response(proc)
        if not response or 'result' not in response:
            print(f"✗ Initialize failed for {model_name}", file=sys.stderr)
            return False
        
        print(f"✓ Initialize successful", file=sys.stderr)
        
        # Send initialized notification
        send_message(proc, {
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        })
        
        time.sleep(0.1)
        
        # Test confer tool
        print(f"\nTesting confer tool with {model_name}...", file=sys.stderr)
        send_message(proc, {
            "jsonrpc": "2.0",
            "method": "tools/call",
            "params": {
                "name": "confer",
                "arguments": {
                    "message": "Hello! Can you count from 1 to 5?",
                    "model": model_name
                }
            },
            "id": 2
        })
        
        response = read_response(proc)
        if response and 'result' in response:
            print(f"✓ Tool call successful!", file=sys.stderr)
            content = response['result'].get('content', [])
            if content:
                text = content[0].get('text', '')
                print(f"\nResponse from {model_name}:", file=sys.stderr)
                print(f"{text[:200]}{'...' if len(text) > 200 else ''}", file=sys.stderr)
            return True
        elif response and 'error' in response:
            print(f"✗ Tool call failed!", file=sys.stderr)
            print(f"Error: {response['error']}", file=sys.stderr)
            return False
            
    finally:
        proc.terminate()
        proc.wait()
        
        # Check logs for errors
        stderr = proc.stderr.read()
        if "error" in stderr.lower() or "failed" in stderr.lower():
            print(f"\nServer logs showing errors:", file=sys.stderr)
            for line in stderr.split('\n'):
                if 'error' in line.lower() or 'failed' in line.lower():
                    print(f"  {line}", file=sys.stderr)
    
    return False

def main():
    print("Testing o3-pro and o4-mini models with lux-mcp", file=sys.stderr)
    
    # Test o3-pro (completions API)
    o3_success = test_model("o3-pro")
    
    # Test o4-mini (chat API with max_completion_tokens)
    o4_success = test_model("o4-mini")
    
    # Test standard model for comparison
    gpt4_success = test_model("gpt-4o")
    
    print(f"\n{'='*60}", file=sys.stderr)
    print("RESULTS:", file=sys.stderr)
    print(f"  o3-pro:  {'✓ SUCCESS' if o3_success else '✗ FAILED'}", file=sys.stderr)
    print(f"  o4-mini: {'✓ SUCCESS' if o4_success else '✗ FAILED'}", file=sys.stderr)
    print(f"  gpt-4o:  {'✓ SUCCESS' if gpt4_success else '✗ FAILED'}", file=sys.stderr)
    print(f"{'='*60}", file=sys.stderr)

if __name__ == "__main__":
    main()