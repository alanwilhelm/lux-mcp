#!/bin/bash

# Quick test to verify Lux MCP is ready for testing

echo "=== Lux MCP Readiness Check ==="
echo

# Check for .env file
if [ -f ".env" ]; then
    echo "✓ .env file found"
    # Show configured keys (masked)
    if grep -q "OPENAI_API_KEY" .env; then
        echo "  - OPENAI_API_KEY configured"
    fi
    if grep -q "OPENROUTER_API_KEY" .env; then
        echo "  - OPENROUTER_API_KEY configured"
    fi
else
    echo "✗ No .env file found"
    echo "  Create one with your API keys:"
    echo "  OPENAI_API_KEY=sk-..."
    echo "  OPENROUTER_API_KEY=sk-or-..."
fi

echo

# Check if binary exists
if [ -f "./target/release/lux-mcp" ]; then
    echo "✓ Binary found at ./target/release/lux-mcp"
else
    echo "✗ Binary not found. Building..."
    cargo build --release
fi

echo

# Show current configuration
echo "Current Model Configuration:"
echo "  Chat: gpt4.1 (fast, balanced)"
echo "  Reasoning: o3-pro (deep thinking)"
echo "  Bias Check: o4-mini (quick verification)"

echo
echo "=== Quick Test Commands ==="
echo
echo "1. Test basic chat:"
echo "   ./test_chat.sh"
echo
echo "2. Test traced reasoning with monitoring:"
echo "   ./test_traced_reasoning.sh"
echo
echo "3. Test biased reasoning:"
echo "   ./test_biased_reasoning.sh"
echo
echo "4. Run with custom models:"
echo "   LUX_DEFAULT_REASONING_MODEL=claude ./target/release/lux-mcp"
echo
echo "5. Use with Claude Desktop:"
echo "   - Add to claude_desktop_config.json"
echo "   - See MODEL_CONFIG.md for details"

echo
echo "Ready to test? Make sure your API keys are set in .env!"