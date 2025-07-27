#!/usr/bin/env python3
import json
import subprocess
import time
import os
import sys

def test_biased_reasoning():
    print("Testing biased_reasoning with MCP protocol...")
    print("=" * 50)
    
    # Set environment with debug logging
    env = os.environ.copy()
    env['RUST_LOG'] = 'debug'
    
    # Start the MCP server
    process = subprocess.Popen(
        ['./target/release/lux-mcp'],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        env=env
    )
    
    try:
        # Send initialize request (MCP protocol)
        init_request = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "1.0",
                "capabilities": {
                    "tools": {}
                }
            }
        }
        
        print(f"Sending: {json.dumps(init_request)}")
        process.stdin.write(json.dumps(init_request) + '\n')
        process.stdin.flush()
        
        # Read initialize response
        response_line = process.stdout.readline()
        print(f"Received: {response_line}")
        
        # Send initialized notification
        initialized_notif = {
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        }
        
        print(f"Sending: {json.dumps(initialized_notif)}")
        process.stdin.write(json.dumps(initialized_notif) + '\n')
        process.stdin.flush()
        
        # Now test biased_reasoning
        br_request = {
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {
                "name": "biased_reasoning",
                "arguments": {
                    "query": "plan a similar system for our stripe payments",
                    "primary_model": "gpt-4",
                    "verifier_model": "o4-mini",
                    "max_steps": 2
                }
            }
        }
        
        print(f"\nSending biased_reasoning request: {json.dumps(br_request)}")
        process.stdin.write(json.dumps(br_request) + '\n')
        process.stdin.flush()
        
        # Read response with timeout
        start_time = time.time()
        timeout = 60  # 60 seconds timeout
        
        while True:
            if process.stdout in subprocess.select.select([process.stdout], [], [], 0.1)[0]:
                response_line = process.stdout.readline()
                if response_line:
                    print(f"Response: {response_line}")
                    try:
                        response = json.loads(response_line)
                        if 'error' in response:
                            print(f"\n❌ ERROR: {response['error']}")
                        elif 'result' in response:
                            print("\n✅ SUCCESS: biased_reasoning completed")
                        break
                    except json.JSONDecodeError:
                        pass
            
            # Check stderr for error logs
            if process.stderr in subprocess.select.select([process.stderr], [], [], 0)[0]:
                stderr_line = process.stderr.readline()
                if stderr_line and ('error' in stderr_line.lower() or 'failed' in stderr_line.lower()):
                    print(f"STDERR: {stderr_line.strip()}")
            
            if time.time() - start_time > timeout:
                print("\n⏰ TIMEOUT: No response after 60 seconds")
                break
        
        # Capture any remaining stderr
        print("\n=== Checking stderr for errors ===")
        process.stdin.close()
        time.sleep(0.5)  # Give it time to flush logs
        
        stderr_output = process.stderr.read()
        for line in stderr_output.split('\n'):
            if 'verifier model' in line.lower() or ('failed' in line.lower() and 'bias' in line.lower()):
                print(f"ERROR LOG: {line}")
        
    finally:
        process.terminate()
        process.wait()

if __name__ == "__main__":
    test_biased_reasoning()