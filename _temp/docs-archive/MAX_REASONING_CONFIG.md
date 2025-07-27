# Maximum Reasoning Configuration

## Using o3-pro for Everything (Maximum Reasoning)

If you want the highest reasoning capability for all tools, including chat:

### .env Configuration
```bash
OPENAI_API_KEY=your-key
OPENROUTER_API_KEY=your-key

# Maximum reasoning for all tools
LUX_DEFAULT_CHAT_MODEL=o3-pro
LUX_DEFAULT_REASONING_MODEL=o3-pro
LUX_DEFAULT_BIAS_CHECKER_MODEL=o3-pro

# Enable detailed logging to see progress
RUST_LOG=info
```

## What This Means

- **Chat responses**: 30 seconds to 5 minutes per response
- **Maximum tokens**: 32,768 tokens for o3 models (increased from 10,000)
- **Reasoning effort**: Automatically set to "high" for o3 models
- **Timeout**: 5 minutes (300 seconds) - won't timeout prematurely

## Usage Tips

1. **Be patient**: o3-pro takes time to think deeply
2. **Watch the logs**: You'll see "Reasoning effort: Some("high")" in logs
3. **Complex queries benefit most**: Simple questions don't need o3-pro
4. **Cost consideration**: o3-pro is expensive - use wisely

## Testing Maximum Reasoning

```bash
# Test with maximum reasoning
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"confer","arguments":{"message":"Explain the philosophical implications of consciousness emerging from physical matter"}}}' | ./target/release/lux-mcp

# Monitor logs in another terminal
RUST_LOG=info ./target/release/lux-mcp 2>&1 | grep -E "(reasoning_effort|max_tokens|Model)"
```

## When to Use This Configuration

✅ **Good for**:
- Deep philosophical questions
- Complex technical analysis
- Multi-faceted problems requiring careful consideration
- Research and exploration of difficult topics

❌ **Not ideal for**:
- Quick questions
- Simple lookups
- Interactive coding assistance
- Time-sensitive tasks

## Alternative: Mixed Configuration

For a balance of speed and capability:

```bash
# Fast chat, deep reasoning when needed
LUX_DEFAULT_CHAT_MODEL=gpt-4o          # Fast responses
LUX_DEFAULT_REASONING_MODEL=o3-pro     # Deep reasoning for traced_reasoning
LUX_DEFAULT_BIAS_CHECKER_MODEL=o4-mini # Fast verification

# Then use o3-pro explicitly when needed:
# ⏺ lux - confer (MCP)(message: "complex question", model: "o3-pro")
```