#!/usr/bin/env python3
"""Test traced reasoning synthesis output"""

import json
import subprocess
import time
import os

def send_mcp_message(proc, message):
    """Send a message to the MCP server"""
    proc.stdin.write((json.dumps(message) + '\n').encode())
    proc.stdin.flush()
    time.sleep(0.5)

def read_response(proc, timeout=10):
    """Read response from MCP server"""
    import select
    responses = []
    
    start_time = time.time()
    while time.time() - start_time < timeout:
        ready, _, _ = select.select([proc.stdout], [], [], 0.1)
        if ready:
            line = proc.stdout.readline().decode().strip()
            if line:
                try:
                    responses.append(json.loads(line))
                except json.JSONDecodeError:
                    pass
        else:
            if responses:
                break
    
    return responses

def main():
    print("🧪 Testing traced_reasoning synthesis output...\n")
    
    # Start MCP server
    proc = subprocess.Popen(
        ['./target/release/lux-mcp'],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        env={
            "RUST_LOG": "info",
            "OPENAI_API_KEY": os.environ.get("OPENAI_API_KEY", ""),
            "LUX_DEFAULT_REASONING_MODEL": "gpt-4o-mini"
        }
    )
    
    # Initialize
    send_mcp_message(proc, {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "test", "version": "1.0"}
        }
    })
    
    send_mcp_message(proc, {"jsonrpc": "2.0", "method": "notifications/initialized"})
    responses = read_response(proc)
    print(f"✅ Initialized")
    
    # Test traced reasoning
    print("\n📝 Testing traced reasoning with synthesis...")
    send_mcp_message(proc, {
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": "traced_reasoning",
            "arguments": {
                "thought": "Analyze the benefits and drawbacks of microservices architecture",
                "thought_number": 1,
                "total_thoughts": 2,
                "next_thought_needed": True,
                "temperature": 0.7
            }
        }
    })
    
    responses = read_response(proc, timeout=30)
    for resp in responses:
        if resp.get('id') == 2:
            content = resp.get('result', {}).get('content', [{}])[0].get('text', '')
            
            # Look for synthesis in the content
            if "Synthesis State:" in content:
                print("✅ Found synthesis in response!")
                # Extract synthesis section
                lines = content.split('\n')
                in_synthesis = False
                for line in lines:
                    if "Synthesis State:" in line:
                        in_synthesis = True
                    if in_synthesis and line.strip():
                        print(line)
                    if "Next Action:" in line:
                        break
            else:
                print("❌ No synthesis found in response")
                print("\nFirst 500 chars of response:")
                print(content[:500])
    
    # Clean up
    proc.kill()
    print("\n✅ Test completed!")

if __name__ == "__main__":
    main()