# Troubleshooting Guide

## Common Errors and Solutions

### 1. "Failed to complete chat request" (confer tool)
**Cause**: Using slow reasoning model (o3-pro) for chat
**Solution**: 
- Use `model: "gpt-4o"` parameter in the tool call
- Or update `.env`: `LUX_DEFAULT_CHAT_MODEL=gpt-4o`

### 2. "Failed to generate planning step" (planner tool)
**Cause**: Model timeout - o3-pro takes too long
**Solution**:
- Use `model: "gpt-4"` parameter in the tool call
- Or update `.env`: `LUX_DEFAULT_REASONING_MODEL=gpt-4`
- Timeout has been increased to 5 minutes (300 seconds)

### 3. "Failed to check for bias" (biased_reasoning)
**Cause**: Insufficient tokens for reasoning models
**Solution**: Already fixed - all models now use 10,000 tokens

### 4. Empty responses from o4-mini
**Cause**: o4-mini uses tokens for internal reasoning
**Solution**: Already fixed - increased to 10,000 tokens

## Checking Logs

### In Claude Desktop
```bash
# macOS
~/Library/Logs/Claude/mcp-*.log

# or check Console.app and filter for "mcp"
```

### Manual Testing
```bash
# Run with debug logs
RUST_LOG=debug ./target/release/lux-mcp 2> debug.log

# Watch logs in real-time
tail -f debug.log
```

## Response Time Expectations

| Model | Expected Response Time | Use Case |
|-------|----------------------|----------|
| gpt-4o | 1-3 seconds | Chat, quick tasks |
| gpt-4 | 2-5 seconds | General reasoning |
| o4-mini | 5-15 seconds | Fast reasoning |
| o3-mini | 20-60 seconds | Deeper reasoning |
| o3 | 30s-2min | Complex reasoning |
| o3-pro | 30s-5min | Very complex reasoning |

## Timeout Configuration

All HTTP clients now use a 5 minute (300 second) timeout to accommodate o3-pro's long response times.

## Emergency Fixes

If tools keep timing out:
1. Add explicit model parameter: `model: "gpt-4"`
2. Update all defaults in `.env` to fast models
3. Use o3 models only when you specifically need deep reasoning

## Recommended .env Configuration

```bash
# For general use (fast responses)
LUX_DEFAULT_CHAT_MODEL=gpt-4o
LUX_DEFAULT_REASONING_MODEL=gpt-4
LUX_DEFAULT_BIAS_CHECKER_MODEL=gpt-4

# For deep reasoning (slower but more capable)
LUX_DEFAULT_CHAT_MODEL=gpt-4o          # Keep chat fast\!
LUX_DEFAULT_REASONING_MODEL=o3-pro     # Deep reasoning
LUX_DEFAULT_BIAS_CHECKER_MODEL=o4-mini # Fast verification
```

## Testing Tools

```bash
# Test with fast models
./test_confer_with_model.sh gpt-4o

# Test o3-pro with 5 minute timeout
./test_o3_timeout.sh

# Check configuration and logs
./check_mcp_logs.sh
```
EOF < /dev/null