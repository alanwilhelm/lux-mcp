# Fixing the Planner Tool Error

## The Issue
The planner tool is failing with "Failed to generate planning step" because it's using o3-pro which can take 30 seconds to 5 minutes to respond.

## Quick Solutions

### Option 1: Use a Faster Model (Recommended)
In Claude Desktop, explicitly specify a faster model:

```
⏺ lux - planner (MCP)(
    step: "Add publishing metadata fields...", 
    step_number: 2, 
    total_steps: 10, 
    next_step_required: true,
    model: "gpt-4"  // Add this parameter
)
```

### Option 2: Update Your .env Configuration
Change the default reasoning model to something faster:

```bash
# In your .env file, change:
LUX_DEFAULT_REASONING_MODEL=o3-pro

# To:
LUX_DEFAULT_REASONING_MODEL=gpt-4
```

### Option 3: Use o3-pro Only When Needed
Keep o3-pro as default but override for quick planning:

```
# For quick planning tasks
model: "gpt-4"

# For deep reasoning when you have time
model: "o3-pro"
```

## What I've Fixed

1. **Increased token limits**: Planner now uses 32,768 tokens for o3 models
2. **Better error logging**: You'll see more detailed errors in logs
3. **Improved error messages**: Shows which model failed

## Checking Logs

To see what's happening:
```bash
# Check Claude Desktop logs
tail -f ~/Library/Logs/Claude/mcp-*.log | grep -i planner
```

## Example Working Call

```
⏺ lux - planner (MCP)(
    step: "Add publishing metadata fields (subtitle, tagline, promoTitle) to ColumnFormBasicInfo component", 
    step_number: 2, 
    total_steps: 10, 
    next_step_required: true,
    model: "gpt-4o",
    temperature: 0.7
)
```

## Why This Happens

- o3-pro is designed for deep reasoning, not quick planning
- It can take 30s-5min per response
- The 5-minute timeout might still be too short for complex requests
- o3-pro costs significantly more than GPT-4

## Recommendation

Use GPT-4 or GPT-4o for planning tasks - they're fast, reliable, and still very capable for structured planning.