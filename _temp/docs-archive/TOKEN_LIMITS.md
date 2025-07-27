# Token Limits Configuration

## Overview
All models now use the maximum token limit (10,000 tokens) to ensure no truncation or empty responses.

## Current Token Limits

### All Models
- **Primary Reasoning Steps**: 10,000 tokens
- **Bias Checking**: 10,000 tokens
- **Final Answer Generation**: 10,000 tokens

## Reasoning Effort for O3 Models
O3 models now use `reasoning_effort: "high"` to ensure maximum reasoning capability.

## Why These Limits?

### O4 Models
O4 models (especially o4-mini) use a significant portion of their token budget for internal reasoning before producing output. With insufficient tokens, they return empty responses because all tokens are consumed by reasoning, leaving none for the actual response.

### O3 Models
O3 models also benefit from higher token limits as they perform deep reasoning, though not to the same extent as O4 models.

### Standard Models
Traditional models like GPT-4, Claude, etc., don't have the same internal reasoning overhead and work well with standard token limits.

## Adjusting Limits

If you encounter empty responses or truncated outputs, consider increasing the token limits further. The limits can be adjusted in `src/tools/biased_reasoning.rs`:

```rust
// For bias checking
let max_tokens = if verifier_model_name.starts_with("o4") { 10000 } else { 2000 };

// For primary reasoning
let primary_max_tokens = if primary_model.starts_with("o3") || primary_model.starts_with("o4") {
    8000
} else {
    3000
};
```

## Cost Considerations

Higher token limits mean higher API costs. However, for reasoning models like O3/O4, these higher limits are necessary for proper functionality. Consider your use case and budget when selecting models and adjusting limits.