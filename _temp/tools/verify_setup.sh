#!/bin/bash

echo "=== Lux MCP Setup Verification ==="
echo

# Check for API keys
echo "Checking API keys..."

# Check environment variables
if [ -n "$OPENAI_API_KEY" ]; then
    if [[ "$OPENAI_API_KEY" == "sk-..." || ${#OPENAI_API_KEY} -lt 20 ]]; then
        echo "❌ OPENAI_API_KEY appears to be a placeholder or too short"
    else
        echo "✅ OPENAI_API_KEY is set (${#OPENAI_API_KEY} chars)"
    fi
else
    echo "❌ OPENAI_API_KEY not set in environment"
fi

if [ -n "$OPENROUTER_API_KEY" ]; then
    if [[ "$OPENROUTER_API_KEY" == "sk-or-v1-..." || ${#OPENROUTER_API_KEY} -lt 20 ]]; then
        echo "❌ OPENROUTER_API_KEY appears to be a placeholder or too short"
    else
        echo "✅ OPENROUTER_API_KEY is set (${#OPENROUTER_API_KEY} chars)"
    fi
else
    echo "❌ OPENROUTER_API_KEY not set in environment"
fi

echo

# Check .env file
if [ -f .env ]; then
    echo "Checking .env file..."
    
    # Check OPENAI_API_KEY in .env
    if grep -q "^OPENAI_API_KEY=" .env; then
        KEY=$(grep "^OPENAI_API_KEY=" .env | cut -d'=' -f2)
        if [[ "$KEY" == "sk-..." || ${#KEY} -lt 20 ]]; then
            echo "❌ .env: OPENAI_API_KEY is a placeholder or too short"
        else
            echo "✅ .env: OPENAI_API_KEY looks valid (${#KEY} chars)"
        fi
    fi
    
    # Check OPENROUTER_API_KEY in .env
    if grep -q "^OPENROUTER_API_KEY=" .env; then
        KEY=$(grep "^OPENROUTER_API_KEY=" .env | cut -d'=' -f2)
        if [[ "$KEY" == "sk-or-v1-..." || ${#KEY} -lt 20 ]]; then
            echo "❌ .env: OPENROUTER_API_KEY is a placeholder or too short"
        else
            echo "✅ .env: OPENROUTER_API_KEY looks valid (${#KEY} chars)"
        fi
    fi
else
    echo "⚠️  No .env file found"
fi

echo
echo "=== Recommendations ==="
echo
echo "1. Edit .env and replace placeholder keys with real ones:"
echo "   OPENAI_API_KEY=sk-proj-... (your actual key)"
echo "   OPENROUTER_API_KEY=sk-or-v1-... (your actual key)"
echo
echo "2. Or set environment variables:"
echo "   export OPENAI_API_KEY='your-key-here'"
echo "   export OPENROUTER_API_KEY='your-key-here'"
echo
echo "3. You only need ONE API key to work (either OpenAI OR OpenRouter)"
echo
echo "For o3-pro and o4-mini models, you need a valid OPENAI_API_KEY"