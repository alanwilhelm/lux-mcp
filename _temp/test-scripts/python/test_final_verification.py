#!/usr/bin/env python3
import subprocess
import json
import time

def test_mcp_server():
    print("🔍 Testing lux-mcp server implementation...")
    print()
    
    # Start the server process
    proc = subprocess.Popen(
        ['./target/release/lux-mcp'],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True
    )
    
    try:
        # Send initialize
        init_req = {"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"1.0.0","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}
        proc.stdin.write(json.dumps(init_req) + '\n')
        proc.stdin.flush()
        
        # Read response
        init_resp = json.loads(proc.stdout.readline())
        print("✅ Server initialized:", init_resp.get('result', {}).get('serverInfo', {}).get('name'))
        print()
        
        # List tools
        tools_req = {"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}
        proc.stdin.write(json.dumps(tools_req) + '\n')
        proc.stdin.flush()
        
        tools_resp = json.loads(proc.stdout.readline())
        tools = tools_resp.get('result', {}).get('tools', [])
        
        print("📋 Registered Tools:")
        for tool in tools:
            print(f"  ✅ {tool['name']}")
            if 'plan_iterative' in tool['name']:
                print("    ❌ ERROR: plan_iterative should have been removed!")
        
        # Verify expected tools
        tool_names = [t['name'] for t in tools]
        expected_tools = ['confer', 'traced_reasoning', 'biased_reasoning', 'planner', 'illumination_status']
        
        print()
        print("🔍 Verification:")
        for expected in expected_tools:
            if expected in tool_names:
                print(f"  ✅ {expected} found")
            else:
                print(f"  ❌ {expected} MISSING!")
        
        if 'plan_iterative' in tool_names:
            print("  ❌ plan_iterative still exists (should be removed)")
        else:
            print("  ✅ plan_iterative successfully removed")
        
        # List prompts
        print()
        prompts_req = {"jsonrpc":"2.0","id":3,"method":"prompts/list","params":{}}
        proc.stdin.write(json.dumps(prompts_req) + '\n')
        proc.stdin.flush()
        
        prompts_resp = json.loads(proc.stdout.readline())
        prompts = prompts_resp.get('result', {}).get('prompts', [])
        
        print("📋 Registered Prompts:")
        for prompt in prompts:
            print(f"  ✅ {prompt['name']}")
        
        prompt_names = [p['name'] for p in prompts]
        if 'planner' in prompt_names:
            print()
            print("✅ Planner is in prompts list")
        else:
            print()
            print("❌ Planner is MISSING from prompts list!")
            
        # Test a simple confer call with progress indicators
        print()
        print("🧪 Testing confer tool with progress indicators...")
        confer_req = {
            "jsonrpc": "2.0",
            "id": 4,
            "method": "tools/call",
            "params": {
                "name": "confer",
                "arguments": {
                    "message": "What is 2+2?",
                    "model": "gpt-4o-mini"  # Fast model for testing
                }
            }
        }
        proc.stdin.write(json.dumps(confer_req) + '\n')
        proc.stdin.flush()
        
        confer_resp = json.loads(proc.stdout.readline())
        if 'result' in confer_resp:
            print("✅ Confer tool working")
        else:
            print("❌ Confer tool error:", confer_resp.get('error'))
            
    finally:
        proc.terminate()
        proc.wait()
    
    print()
    print("✨ All tests completed!")

if __name__ == "__main__":
    test_mcp_server()