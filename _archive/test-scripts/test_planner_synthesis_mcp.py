#!/usr/bin/env python3
"""Test planner synthesis integration through MCP"""

import json
import subprocess
import time
import re
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

def extract_synthesis_info(content):
    """Extract synthesis information from planner response"""
    synthesis_info = {
        'current_plan': None,
        'confidence': None,
        'key_decisions': [],
        'next_actions': []
    }
    
    # Look for synthesis state section
    if "Synthesis State:" in content:
        lines = content.split('\n')
        for i, line in enumerate(lines):
            if "Current Plan:" in line:
                synthesis_info['current_plan'] = line.split("Current Plan:")[1].strip()
            elif "Confidence:" in line:
                synthesis_info['confidence'] = line.split("Confidence:")[1].strip()
            elif "Key Decisions:" in line:
                # Collect key decisions
                j = i + 1
                while j < len(lines) and lines[j].startswith("•"):
                    synthesis_info['key_decisions'].append(lines[j].strip("• "))
                    j += 1
            elif "Next Actions:" in line:
                # Collect next actions
                j = i + 1
                while j < len(lines) and lines[j].startswith("•"):
                    synthesis_info['next_actions'].append(lines[j].strip("• "))
                    j += 1
    
    return synthesis_info

def main():
    print("🧪 Testing planner synthesis integration...\n")
    
    # Start MCP server
    proc = subprocess.Popen(
        ['./target/release/lux-mcp'],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        env={
            "RUST_LOG": "info",
            "DATABASE_URL": "postgres://lux_user:lux_password@localhost/lux_mcp",
            "OPENAI_API_KEY": os.environ.get("OPENAI_API_KEY", ""),
            "OPENROUTER_API_KEY": os.environ.get("OPENROUTER_API_KEY", ""),
            "LUX_DEFAULT_REASONING_MODEL": "gpt-4o-mini"  # Use a fast model for testing
        }
    )
    
    # Initialize
    print("📝 Step 1: Initialize MCP server...")
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
    print(f"✅ Initialized (got {len(responses)} responses)")
    
    # Test planner with multiple steps
    steps = [
        (1, "Design a REST API for a task management system", True),
        (2, "Define the core endpoints and data models", True),
        (3, "Plan authentication and authorization strategy", True),
        (4, "Design database schema and relationships", True),
        (5, "Plan deployment and monitoring strategy", False)
    ]
    
    for step_num, description, next_required in steps:
        print(f"\n📝 Step {step_num}: {description}")
        
        send_mcp_message(proc, {
            "jsonrpc": "2.0",
            "id": step_num + 1,
            "method": "tools/call",
            "params": {
                "name": "planner",
                "arguments": {
                    "step": description,
                    "step_number": step_num,
                    "total_steps": 5,
                    "next_step_required": next_required
                }
            }
        })
        
        responses = read_response(proc, timeout=30)
        for resp in responses:
            if resp.get('id') == step_num + 1:
                content = resp.get('result', {}).get('content', [{}])[0].get('text', '')
                
                # Extract synthesis info
                synthesis = extract_synthesis_info(content)
                
                if synthesis['current_plan'] or synthesis['key_decisions'] or synthesis['next_actions']:
                    print("\n🎯 Synthesis State:")
                    if synthesis['current_plan']:
                        print(f"   Current Plan: {synthesis['current_plan']}")
                    if synthesis['confidence']:
                        print(f"   Confidence: {synthesis['confidence']}")
                    if synthesis['key_decisions']:
                        print("   Key Decisions:")
                        for decision in synthesis['key_decisions']:
                            print(f"      • {decision}")
                    if synthesis['next_actions']:
                        print("   Next Actions:")
                        for action in synthesis['next_actions']:
                            print(f"      • {action}")
                else:
                    print("   No synthesis information found in response")
                
                break
    
    # Clean up
    proc.kill()
    print("\n✅ Test completed!")

if __name__ == "__main__":
    main()