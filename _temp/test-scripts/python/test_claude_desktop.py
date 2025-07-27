#!/usr/bin/env python3
"""
Test script that simulates Claude Desktop's MCP connection
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

def test_server():
    # Load .env file values
    from dotenv import load_dotenv
    load_dotenv()
    
    # Set up environment as Claude Desktop would
    env = os.environ.copy()
    
    print("=== Testing Lux MCP Server (Claude Desktop Simulation) ===", file=sys.stderr)
    print(f"OPENAI_API_KEY: {'✓ Set' if env.get('OPENAI_API_KEY') else '✗ Not set'}", file=sys.stderr)
    print(f"OPENROUTER_API_KEY: {'✓ Set' if env.get('OPENROUTER_API_KEY') else '✗ Not set'}", file=sys.stderr)
    print(file=sys.stderr)
    
    # Start the server
    print("Starting MCP server...", file=sys.stderr)
    try:
        proc = subprocess.Popen(
            ['./target/release/lux-mcp'],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            env=env,
            bufsize=1
        )
    except Exception as e:
        print(f"Failed to start server: {e}", file=sys.stderr)
        return
    
    # Give server time to start
    time.sleep(0.5)
    
    try:
        # Step 1: Send initialize request
        print("\n1. Sending initialize request...", file=sys.stderr)
        send_message(proc, {
            "jsonrpc": "2.0",
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {},
                    "prompts": {}
                },
                "clientInfo": {
                    "name": "claude-desktop",
                    "version": "1.0"
                }
            },
            "id": 1
        })
        
        # Read initialize response
        response = read_response(proc)
        if not response or 'result' not in response:
            print("✗ Initialize failed!", file=sys.stderr)
            print(f"Response: {response}", file=sys.stderr)
            return
        
        print("✓ Initialize successful", file=sys.stderr)
        server_info = response['result'].get('serverInfo', {})
        print(f"  Server: {server_info.get('name')} v{server_info.get('version')}", file=sys.stderr)
        
        # Step 2: Send initialized notification
        print("\n2. Sending initialized notification...", file=sys.stderr)
        send_message(proc, {
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        })
        
        # Give server time to process
        time.sleep(0.1)
        
        # Step 3: List available tools
        print("\n3. Listing available tools...", file=sys.stderr)
        send_message(proc, {
            "jsonrpc": "2.0",
            "method": "tools/list",
            "params": {},
            "id": 2
        })
        
        response = read_response(proc)
        if response and 'result' in response:
            tools = response['result'].get('tools', [])
            print(f"✓ Found {len(tools)} tools:", file=sys.stderr)
            for tool in tools:
                print(f"  - {tool['name']}: {tool.get('description', '')}", file=sys.stderr)
        
        # Step 4: Test confer tool
        print("\n4. Testing confer tool...", file=sys.stderr)
        send_message(proc, {
            "jsonrpc": "2.0",
            "method": "tools/call",
            "params": {
                "name": "confer",
                "arguments": {
                    "message": "Hello! Can you hear me?"
                }
            },
            "id": 3
        })
        
        response = read_response(proc)
        if response and 'result' in response:
            print("✓ Tool call successful!", file=sys.stderr)
            content = response['result'].get('content', [])
            if content:
                print(f"  Response: {content[0].get('text', '')}", file=sys.stderr)
        elif response and 'error' in response:
            print("✗ Tool call failed!", file=sys.stderr)
            print(f"  Error: {response['error']}", file=sys.stderr)
        
    except Exception as e:
        print(f"\nError during test: {e}", file=sys.stderr)
        import traceback
        traceback.print_exc()
    
    finally:
        # Clean up
        proc.terminate()
        proc.wait()
        
        # Print server logs
        stderr = proc.stderr.read()
        if stderr:
            print("\n=== Server logs ===", file=sys.stderr)
            print(stderr, file=sys.stderr)
        
        print("\n=== Test complete ===", file=sys.stderr)

if __name__ == "__main__":
    # Check if python-dotenv is installed
    try:
        import dotenv
    except ImportError:
        print("Installing python-dotenv...", file=sys.stderr)
        subprocess.check_call([sys.executable, "-m", "pip", "install", "python-dotenv"])
        print("Installed. Please run the script again.", file=sys.stderr)
        sys.exit(1)
    
    test_server()