#!/usr/bin/env python3
"""
Test which OpenAI models actually exist
"""
import os
import requests
import json

# Get API key from environment
api_key = os.environ.get('OPENAI_API_KEY', '')
if not api_key or api_key.startswith('sk-...'):
    print("Error: Need a valid OPENAI_API_KEY")
    exit(1)

print("Testing OpenAI models...")
print("=" * 50)

# Models to test - including common ones and the ones you tried
test_models = [
    # GPT-4 variants
    "gpt-4",
    "gpt-4-turbo",
    "gpt-4-turbo-preview", 
    "gpt-4-1106-preview",
    "gpt-4-0125-preview",
    "gpt-4o",
    "gpt-4o-mini",
    
    # GPT-3.5 variants
    "gpt-3.5-turbo",
    "gpt-3.5-turbo-0125",
    "gpt-3.5-turbo-1106",
    
    # The models you were trying to use
    "o3",
    "o3-pro", 
    "o4-mini",
    "o1",
    "o1-preview",
    "o1-mini",
]

headers = {
    "Authorization": f"Bearer {api_key}",
    "Content-Type": "application/json"
}

# Test each model
for model in test_models:
    print(f"\nTesting: {model}")
    
    data = {
        "model": model,
        "messages": [{"role": "user", "content": "Say 'yes' if you exist"}],
        "max_tokens": 5,
        "temperature": 0
    }
    
    try:
        response = requests.post(
            "https://api.openai.com/v1/chat/completions",
            headers=headers,
            json=data,
            timeout=10
        )
        
        if response.status_code == 200:
            print(f"  ✅ {model} - EXISTS!")
            result = response.json()
            actual_model = result.get('model', 'unknown')
            print(f"     Actual model returned: {actual_model}")
        elif response.status_code == 404:
            print(f"  ❌ {model} - NOT FOUND")
            error_data = response.json()
            if 'error' in error_data:
                print(f"     Error: {error_data['error'].get('message', '')}")
        else:
            print(f"  ❌ {model} - Error {response.status_code}")
            
    except Exception as e:
        print(f"  ❌ {model} - Request failed: {str(e)}")

print("\n" + "=" * 50)
print("SUMMARY: Only models marked with ✅ actually exist!")
print("\nRecommended models for your config:")
print("- Chat: gpt-4-turbo-preview or gpt-4o-mini")
print("- Reasoning: gpt-4-turbo-preview or gpt-4")
print("- Bias Checker: gpt-4o-mini or gpt-3.5-turbo")