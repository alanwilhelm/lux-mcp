# Lux MCP Quick Start Guide

## Minimal Setup (No Database Required!)

Lux MCP works perfectly without a database - all core features are available with in-memory storage.

### 1. Prerequisites
- Rust 1.70+
- OpenAI API key OR OpenRouter API key

### 2. Build
```bash
git clone https://github.com/yourusername/lux-mcp.git
cd lux-mcp
cargo build --release
```

### 3. Configure Environment

#### Option A: OpenAI Only
```bash
export OPENAI_API_KEY="sk-..."
```

#### Option B: OpenRouter Only
```bash
export OPENROUTER_API_KEY="sk-..."
```

#### Option C: Both (Recommended)
```bash
export OPENAI_API_KEY="sk-..."
export OPENROUTER_API_KEY="sk-..."
```

### 4. Run
```bash
./target/release/lux-mcp
```

That's it! The server is now running on stdio for MCP communication.

## Claude Desktop Setup

Add to `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "lux": {
      "command": "/absolute/path/to/lux-mcp/target/release/lux-mcp",
      "env": {
        "OPENAI_API_KEY": "your-openai-key"
      }
    }
  }
}
```

Restart Claude Desktop and you'll see Lux tools available!

## Test the Server

### Quick Test
```bash
# In one terminal
./target/release/lux-mcp

# In another terminal
echo '{"jsonrpc":"2.0","method":"tools/list","id":1}' | nc localhost 3333
```

### Test a Tool
```bash
cat << 'EOF' | python3 -m json.tool | nc localhost 3333
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "confer",
    "arguments": {
      "message": "Hello! What is metacognition?"
    }
  }
}
EOF
```

## Available Tools

1. **confer** - Chat with any LLM
   ```json
   {
     "tool": "confer",
     "arguments": {
       "message": "Your question here"
     }
   }
   ```

2. **traced_reasoning** - Step-by-step reasoning with monitoring
   ```json
   {
     "tool": "traced_reasoning",
     "arguments": {
       "thought": "Complex problem to solve",
       "thought_number": 1,
       "total_thoughts": 5,
       "next_thought_needed": true
     }
   }
   ```

3. **biased_reasoning** - Dual-model bias detection
   ```json
   {
     "tool": "biased_reasoning",
     "arguments": {
       "query": "Question to analyze for bias"
     }
   }
   ```

4. **planner** - Interactive planning
   ```json
   {
     "tool": "planner",
     "arguments": {
       "step": "Task to plan",
       "step_number": 1,
       "total_steps": 7,
       "next_step_required": true
     }
   }
   ```

5. **illumination_status** - Check system status
   ```json
   {
     "tool": "illumination_status",
     "arguments": {}
   }
   ```

## Optional: Model Configuration

### Default Models
```bash
# Change default models (optional)
export LUX_DEFAULT_CHAT_MODEL="gpt-4o"        # for confer
export LUX_DEFAULT_REASONING_MODEL="o3-pro"   # for traced_reasoning
export LUX_DEFAULT_BIAS_CHECKER_MODEL="o4-mini" # for bias checking
```

### Supported Models

#### OpenAI
- `gpt-4o`, `gpt-4o-mini` - Latest GPT-4
- `o3`, `o3-pro`, `o3-mini` - Deep reasoning (30s-5min)
- `o4-mini` - Fast reasoning
- `gpt-3.5-turbo` - Fast, economical

#### OpenRouter
- `claude` → Claude 3 Opus
- `gemini` → Gemini Pro
- `llama3` → Llama 3 70B

## Key Features

✅ All tools work with in-memory state
✅ Conversation threading (3-hour memory)
✅ Metacognitive monitoring
✅ Quality tracking
✅ Session management (30-minute sessions)

## Troubleshooting

### "Model not found"
- Check your API key is set correctly
- Verify the model name or use an alias
- Some models require specific API access

### Empty responses from O4
- O4 models need high token limits
- Already configured by default (10,000 tokens)

### Slow O3 responses
- Normal - O3 does deep reasoning
- Expect 30 seconds to 5 minutes

### Can't connect to server
- Ensure the server is running
- Check no other process is using port 3333
- Try `lsof -i :3333` to check

## Next Steps

1. Read [API_REFERENCE.md](API_REFERENCE.md) for detailed documentation
2. Try the example scripts in `_archive/test-scripts/`
3. Explore threading with `continuation_id`
4. Experiment with different models
5. Join our Discord for support!

---

**Remember**: Database is completely optional! Lux MCP is designed to work great without any external dependencies.