# Lux MCP - Quick Start Guide

## 1. Setup (2 minutes)

```bash
# Clone the repository
git clone https://github.com/yourusername/lux-mcp
cd lux-mcp

# Copy environment config
cp .env.example .env

# Edit .env and add your API key(s)
# You need at least ONE of these:
# - OPENAI_API_KEY=sk-...
# - OPENROUTER_API_KEY=sk-or-v1-...

# Build the project
cargo build --release
```

## 2. Configure Claude Desktop

Add to your Claude Desktop config:

```json
{
  "mcpServers": {
    "lux": {
      "command": "/path/to/lux-mcp/target/release/lux-mcp",
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

## 3. Test the Chat Tool

In Claude Desktop, try:

```
Use the lux_chat tool to ask GPT-4: "What is the capital of France?"
```

Or with model aliases:

```
Use lux_chat with model "mini" to explain quantum computing in simple terms
```

## Model Aliases

Super easy model selection:

### OpenAI Models (Primary)

| Alias | Resolves To | Description |
|-------|-------------|-------------|
| `gpt4.1`, `4.1` | `gpt-4-turbo-preview` | Latest GPT-4 |
| `o3` | `o3` | Advanced reasoning |
| `o3-pro`, `o3pro` | `o3-pro` | Professional reasoning |
| `o4-mini`, `mini` | `o4-mini` | Fast & efficient |

### OpenRouter Models

| Alias | Resolves To | Description |
|-------|-------------|-------------|
| `claude`, `opus` | `claude-3-opus` | Claude 3 Opus |
| `sonnet` | `claude-3-sonnet` | Claude 3 Sonnet |
| `llama3` | `meta-llama/llama-3-70b` | Llama 3 70B |
| `mixtral` | `mistralai/mixtral-8x7b` | Mixtral MoE |
| `gemini` | `google/gemini-2.5-pro` | Latest Gemini 2.5 Pro |
| `flash` | `google/gemini-2.5-flash` | Fast Gemini 2.5 |
| `gemini-free` | `google/gemini-2.0-flash-exp:free` | Free Gemini |

## Environment Variables

```bash
# Required (at least one)
OPENAI_API_KEY=sk-...
OPENROUTER_API_KEY=sk-or-v1-...

# Optional defaults
LUX_DEFAULT_CHAT_MODEL=gpt4.1       # for lux_chat
LUX_DEFAULT_REASONING_MODEL=o3      # for traced_reasoning
LUX_DEFAULT_BIAS_CHECKER_MODEL=o4-mini # for biased_reasoning
```

## Examples

### Simple Chat
```json
{
  "tool": "lux_chat",
  "arguments": {
    "message": "Explain recursion",
    "model": "gpt4.1"
  }
}
```

### With Temperature
```json
{
  "tool": "lux_chat",
  "arguments": {
    "message": "Write a creative story",
    "model": "claude",
    "temperature": 0.9
  }
}
```

### OpenRouter Models
```json
{
  "tool": "lux_chat",
  "arguments": {
    "message": "Analyze this code",
    "model": "deepseek/deepseek-coder"
  }
}
```

## Troubleshooting

### No API Keys Error
```
Error: No API keys configured
```
→ Set `OPENAI_API_KEY` or `OPENROUTER_API_KEY` in `.env`

### Model Not Found
```
Error: Invalid model
```
→ Check model name or use an alias from the table above

### Timeout
```
Error: Request timeout
```
→ Increase `LUX_REQUEST_TIMEOUT_SECS=60` in `.env`

## Traced Reasoning Tool

The `traced_reasoning` tool provides advanced chain-of-thought reasoning with multi-metric monitoring based on cutting-edge research:

### Basic Usage
```json
{
  "tool": "traced_reasoning",
  "arguments": {
    "query": "What are the ethical implications of AI?",
    "model": "gpt4.1"
  }
}
```

### With Custom Configuration
```json
{
  "tool": "traced_reasoning",
  "arguments": {
    "query": "Explain quantum entanglement",
    "model": "o3",
    "max_steps": 8,
    "temperature": 0.6,
    "guardrails": {
      "semantic_drift_check": true,
      "perplexity_monitoring": true,
      "circular_reasoning_detection": true
    }
  }
}
```

### Features
- **Multi-metric Monitoring**: Tracks semantic drift, perplexity, attention entropy
- **Real-time Interventions**: Detects and corrects reasoning issues
- **Structured Output**: Clear reasoning steps with confidence scores
- **Guardrails**: Prevents circular reasoning, hallucinations, and quality degradation

### Example Output
The tool provides:
- Final answer with confidence score
- Step-by-step reasoning trace
- Metrics for each step (semantic similarity, perplexity, etc.)
- Any interventions triggered during reasoning
- Overall reasoning quality assessment

## Biased Reasoning Tool

The `biased_reasoning` tool provides dual-model reasoning where each step is verified for bias and reasoning errors:

### Basic Usage
```json
{
  "tool": "biased_reasoning",
  "arguments": {
    "query": "Is nuclear energy the best solution for climate change?",
    "max_steps": 5
  }
}
```

### With Custom Models
```json
{
  "tool": "biased_reasoning",
  "arguments": {
    "query": "What are the pros and cons of cryptocurrency?",
    "primary_model": "gpt4.1",
    "verifier_model": "o4-mini",
    "max_steps": 6,
    "temperature": 0.7
  }
}
```

### With Bias Configuration
```json
{
  "tool": "biased_reasoning",
  "arguments": {
    "query": "Why is my approach the best solution?",
    "bias_config": {
      "check_confirmation_bias": true,
      "check_anchoring_bias": true,
      "check_availability_bias": true,
      "check_reasoning_errors": true,
      "bias_threshold": 0.6
    }
  }
}
```

### Features
- **Dual-Model Architecture**: Primary model reasons, verifier checks each step
- **Real-time Bias Detection**: Identifies confirmation bias, anchoring, and more
- **Step Correction**: Generates corrected thoughts for biased reasoning
- **Quality Metrics**: Tracks step quality and overall reasoning assessment
- **Configurable Models**: Use any combination of OpenAI/OpenRouter models

### Example Output
The tool provides:
- Final answer with dual-model verification
- Step-by-step reasoning with bias annotations
- Corrected thoughts when bias is detected
- Quality score for each step
- Overall assessment with most common biases
- Models used for transparency

### Default Models
- **Primary Reasoning**: Uses `LUX_DEFAULT_REASONING_MODEL` (default: o3)
- **Bias Verification**: Uses `LUX_DEFAULT_BIAS_CHECKER_MODEL` (default: o4-mini)

Override these via environment variables or request parameters.

## Next Steps

- Try different models with `lux_chat`
- Explore `traced_reasoning` for complex problem solving
- Use `biased_reasoning` for dual-model verification and bias detection

## Support

- Issues: https://github.com/yourusername/lux-mcp/issues
- Docs: See README.md for full documentation