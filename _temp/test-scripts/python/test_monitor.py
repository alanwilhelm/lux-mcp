#!/usr/bin/env python3
"""Test the MetacognitiveMonitor integration"""

import json
import subprocess
import sys
import time

def send_request(proc, request):
    """Send a JSON-RPC request and get response"""
    request_str = json.dumps(request) + '\n'
    proc.stdin.write(request_str.encode())
    proc.stdin.flush()
    
    # Read response
    response_line = proc.stdout.readline()
    if response_line:
        return json.loads(response_line)
    return None

def main():
    print("=== Testing MetacognitiveMonitor Integration ===\n")
    
    # Start the MCP server
    proc = subprocess.Popen(
        ['./target/release/lux-mcp'],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=False
    )
    
    try:
        # Initialize
        print("1. Initializing...")
        init_request = {
            "jsonrpc": "2.0",
            "method": "initialize",
            "params": {"capabilities": {}},
            "id": 1
        }
        response = send_request(proc, init_request)
        print(f"Response: {json.dumps(response, indent=2)}\n")
        
        # Test circular reasoning
        print("2. Testing circular reasoning detection...")
        circular_request = {
            "jsonrpc": "2.0",
            "method": "tools/call",
            "params": {
                "name": "traced_reasoning",
                "arguments": {
                    "query": "Understanding recursion requires understanding recursion. To understand recursion, you must understand recursion.",
                    "max_steps": 3,
                    "guardrails": {
                        "circular_reasoning_detection": True
                    }
                }
            },
            "id": 2
        }
        response = send_request(proc, circular_request)
        
        if response and 'result' in response:
            result = response['result']
            print(f"\nResult received. Looking for interventions...")
            
            # Check for interventions
            if 'interventions' in result:
                print(f"Interventions found: {len(result['interventions'])}")
                for intervention in result['interventions']:
                    print(f"  - Step {intervention['step']}: {intervention['intervention_type']} - {intervention['description']}")
            else:
                print("No interventions field found in response")
                
            # Check reasoning steps
            if 'reasoning_steps' in result:
                print(f"\nReasoning steps: {len(result['reasoning_steps'])}")
                for step in result['reasoning_steps']:
                    if 'metrics' in step:
                        print(f"  Step {step['step_number']}: circular_score={step['metrics'].get('semantic_similarity', 'N/A')}")
                        
        else:
            print("Error or no result in response")
            print(f"Full response: {json.dumps(response, indent=2)}")
            
    except Exception as e:
        print(f"Error: {e}")
    finally:
        # Clean up
        proc.terminate()
        proc.wait()

if __name__ == "__main__":
    main()