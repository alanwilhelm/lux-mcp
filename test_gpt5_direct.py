#!/usr/bin/env python3
import os
import requests
import json

# Test GPT-5 directly via OpenAI API
api_key = os.environ.get('OPENAI_API_KEY')
if not api_key:
    print("Error: OPENAI_API_KEY not set")
    exit(1)

url = "https://api.openai.com/v1/chat/completions"
headers = {
    "Authorization": f"Bearer {api_key}",
    "Content-Type": "application/json"
}

data = {
    "model": "gpt-5",
    "messages": [
        {"role": "user", "content": "What model are you? Please state your exact model name and version."}
    ],
    "max_tokens": 100,
    "temperature": 0.7
}

print("Testing GPT-5 availability via OpenAI API...")
print(f"Request: {json.dumps(data, indent=2)}")
print("\nSending request...")

try:
    response = requests.post(url, headers=headers, json=data)
    print(f"Status Code: {response.status_code}")
    print(f"Response: {json.dumps(response.json(), indent=2)}")
    
    if response.status_code == 200:
        print("\n✅ GPT-5 IS AVAILABLE!")
        content = response.json()['choices'][0]['message']['content']
        print(f"Model response: {content}")
    else:
        print("\n❌ GPT-5 request failed")
        
except Exception as e:
    print(f"Error: {e}")