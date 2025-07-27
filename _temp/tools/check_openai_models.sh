#!/bin/bash

source .env

echo "Fetching all available OpenAI models..."
echo "======================================"

response=$(curl -s https://api.openai.com/v1/models \
  -H "Authorization: Bearer $OPENAI_API_KEY")

echo "$response" | python3 -c "
import json
import sys

data = json.load(sys.stdin)
models = data.get('data', [])

print(f'Found {len(models)} models:\n')

# Filter and sort models
relevant_models = []
for model in models:
    model_id = model.get('id', '')
    if any(x in model_id.lower() for x in ['gpt', 'o1', 'o2', 'o3', 'o4', 'davinci', 'turbo']):
        relevant_models.append(model_id)

relevant_models.sort()

print('GPT Models:')
for m in relevant_models:
    if 'gpt' in m:
        print(f'  - {m}')

print('\nO-series Models:')
for m in relevant_models:
    if m.startswith('o') and m[1].isdigit():
        print(f'  - {m}')

print('\nOther Models:')
for m in relevant_models:
    if 'gpt' not in m and not (m.startswith('o') and m[1].isdigit()):
        print(f'  - {m}')
"