#!/usr/bin/env python3
"""
Test o3-pro responses API directly with curl
"""
import os
import json
import subprocess

def test_o3_direct():
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
    
    # Test with a simple request
    request = {
        "model": "o3-pro-2025-06-10",
        "input": "User: Say 'hello world'",
        "max_output_tokens": 50
    }
    
    print("Testing o3-pro directly with OpenAI API...")
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
    print(result.stdout)
    
    if result.stderr:
        print("\nError:")
        print(result.stderr)
    
    # Try to parse the response
    try:
        response = json.loads(result.stdout)
        print("\nParsed response:")
        print(json.dumps(response, indent=2))
        
        # Check if we can extract the content
        if 'output' in response:
            for output in response['output']:
                if output.get('type') == 'message' and 'content' in output:
                    for content in output['content']:
                        if content.get('type') == 'output_text' and 'text' in content:
                            print(f"\nExtracted text: {content['text']}")
    except Exception as e:
        print(f"\nFailed to parse response: {e}")

if __name__ == "__main__":
    test_o3_direct()