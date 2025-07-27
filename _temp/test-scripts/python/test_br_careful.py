#!/usr/bin/env python3
import json
import subprocess
import time
import os

def send_and_receive(proc, message, wait=0.5):
    """Send a message and wait for response"""
    print(f"\n→ Sending: {json.dumps(message)}")
    proc.stdin.write(json.dumps(message) + '\n')
    proc.stdin.flush()
    time.sleep(wait)
    
    # Read any available output
    while True:
        try:
            line = proc.stdout.readline()
            if line:
                print(f"← Response: {line.strip()}")
                return json.loads(line)
            else:
                break
        except:
            break
    return None

def main():
    print("Testing biased_reasoning with careful timing...")
    print("=" * 60)
    
    # Set environment
    env = os.environ.copy()
    env['RUST_LOG'] = 'info'
    
    # Start the MCP server
    proc = subprocess.Popen(
        ['./target/release/lux-mcp'],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        env=env,
        bufsize=0
    )
    
    time.sleep(1)  # Give server time to start
    
    try:
        # Initialize
        init_msg = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "0.1.0",
                "capabilities": {"tools": {}}
            }
        }
        
        resp = send_and_receive(proc, init_msg, 1)
        
        if resp and 'result' in resp:
            print("✅ Initialize successful")
            
            # Test biased_reasoning with small input
            br_msg = {
                "jsonrpc": "2.0",
                "id": 2,
                "method": "tools/call",
                "params": {
                    "name": "biased_reasoning",
                    "arguments": {
                        "query": "Should we use Redis?",
                        "primary_model": "gpt-4",
                        "verifier_model": "gpt-4",
                        "max_steps": 1
                    }
                }
            }
            
            print("\n📤 Sending biased_reasoning request...")
            proc.stdin.write(json.dumps(br_msg) + '\n')
            proc.stdin.flush()
            
            # Wait for response with timeout
            start_time = time.time()
            timeout = 30
            
            while time.time() - start_time < timeout:
                line = proc.stdout.readline()
                if line:
                    print(f"← Response: {line.strip()}")
                    try:
                        resp = json.loads(line)
                        if 'error' in resp:
                            print(f"\n❌ ERROR: {resp['error']}")
                            # Check stderr for details
                            time.sleep(0.5)
                            stderr = proc.stderr.read()
                            if stderr:
                                print("\n=== STDERR OUTPUT ===")
                                print(stderr)
                        break
                    except:
                        pass
                time.sleep(0.1)
        
    except Exception as e:
        print(f"\n❌ Exception: {e}")
    finally:
        proc.terminate()
        proc.wait()
        
        # Print any remaining stderr
        stderr = proc.stderr.read()
        if stderr:
            print("\n=== Final STDERR ===")
            print(stderr)

if __name__ == "__main__":
    main()