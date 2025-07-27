#!/usr/bin/env python3
"""
Test o3-pro to see the reasoning output structure
"""
import os
import json
import subprocess

def test_o3_reasoning():
    # Read API key from .env file
    api_key = None
    try:
        with open('.env', 'r') as f:
            for line in f:
                if line.startswith('OPENAI_API_KEY='):
                    api_key = line.split('=', 1)[1].strip()
                    break
    except:
        pass
    
    if not api_key:
        print("OPENAI_API_KEY not found in .env!")
        return
    
    # Test with a request that should trigger reasoning
    request = {
        "model": "o3-pro-2025-06-10",
        "input": "User: What is 25 * 37? Think step by step.",
        "max_output_tokens": 200
    }
    
    print("Testing o3-pro reasoning output...")
    print(f"Request: {json.dumps(request, indent=2)}")
    
    cmd = [
        'curl', '-X', 'POST',
        'https://api.openai.com/v1/responses',
        '-H', 'Content-Type: application/json',
        '-H', f'Authorization: Bearer {api_key}',
        '-d', json.dumps(request)
    ]
    
    result = subprocess.run(cmd, capture_output=True, text=True)
    
    print("\nResponse:")
    try:
        response = json.loads(result.stdout)
        print(json.dumps(response, indent=2))
        
        # Analyze the output structure
        print("\n--- Output Analysis ---")
        if 'output' in response:
            for i, output in enumerate(response['output']):
                print(f"\nOutput {i}:")
                print(f"  Type: {output.get('type')}")
                if 'content' in output:
                    print(f"  Content items: {len(output['content'])}")
                    for j, content in enumerate(output['content']):
                        print(f"    Content {j}:")
                        print(f"      Type: {content.get('type')}")
                        if 'text' in content:
                            print(f"      Text: {content['text'][:100]}...")
                if 'summary' in output:
                    print(f"  Summary: {output.get('summary')}")
    except Exception as e:
        print(f"\nFailed to parse response: {e}")
        print(result.stdout)

if __name__ == "__main__":
    test_o3_reasoning()