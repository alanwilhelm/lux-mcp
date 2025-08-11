# Hybrid Biased Reasoning Tool

## Overview

The `hybrid_biased_reasoning` tool is a sophisticated bias detection system that combines Claude's reasoning capabilities with external LLM verification. This tool is designed for scenarios where Claude provides the main reasoning chain, but an external model checks for biases, logical fallacies, and problematic assumptions.

## Key Features

- **Hybrid Architecture**: Claude provides reasoning, external LLM checks for bias
- **Direct File Access**: Reads files independently to provide full context to external LLM
- **Comprehensive Bias Detection**: Identifies cognitive biases, logical fallacies, and assumptions
- **Session Management**: Tracks bias patterns and file contexts across reasoning steps
- **Confidence Scoring**: Provides confidence levels for bias assessments
- **Alternative Suggestions**: Offers improved phrasings when bias is detected
- **Flexible Model Selection**: Choose which model performs bias checking
- **Context Isolation**: Maintains separate context from Claude for independent verification

## When to Use

### Ideal Use Cases

1. **Critical Decision Making**: When reasoning quality is paramount
2. **Academic/Research Work**: Ensuring rigorous, unbiased analysis
3. **Legal/Medical Reasoning**: Where bias could have serious consequences
4. **Content Review**: Checking for unconscious bias in writing
5. **Algorithm Auditing**: Reviewing AI-generated reasoning for fairness

### Not Recommended For

- Quick, low-stakes decisions
- Simple factual queries
- Time-sensitive responses (adds latency)
- Budget-constrained projects (uses two models)

## API Reference

### Request Structure

```json
{
  "reasoning_step": "string",           // Required: The reasoning from Claude
  "context": "string",                  // Optional: Original query/context
  "step_number": 1,                     // Optional: Step in reasoning chain
  "previous_steps": ["step1", "step2"], // Optional: Previous reasoning
  "session_id": "session-123",          // Optional: For tracking
  "bias_check_model": "gpt-4o",         // Optional: Model for checking
  "temperature": 0.3,                   // Optional: Lower = consistent
  "bias_types": ["confirmation", "anchoring"], // Optional: Specific biases
  "file_paths": ["/path/to/file.py"],   // Optional: Files to include in context
  "include_file_contents": true         // Optional: Whether to read files (default: true)
}
```

### Response Structure

```json
{
  "bias_detected": true,
  "confidence": 0.85,
  "biases_found": [
    {
      "bias_type": "Confirmation Bias",
      "description": "Favoring information that confirms existing beliefs",
      "severity": "medium",
      "location": "First paragraph"
    }
  ],
  "suggestions": [
    "Consider alternative viewpoints",
    "Include contradictory evidence"
  ],
  "bias_score": 0.45,
  "revision_recommended": true,
  "alternative_phrasing": "A more balanced way to state this...",
  "model_used": "gpt-4o",
  "session_id": "session-123"
}
```

## Bias Types Detected

### Cognitive Biases
- **Confirmation Bias**: Favoring confirming information
- **Anchoring Bias**: Over-relying on first information
- **Availability Heuristic**: Overweighting recent/memorable info
- **Hindsight Bias**: Assuming past events were predictable
- **Dunning-Kruger Effect**: Overconfidence with limited knowledge

### Logical Fallacies
- **Hasty Generalization**: Broad conclusions from limited data
- **False Dichotomy**: Presenting only two options
- **Ad Hominem**: Attacking the person, not the argument
- **Straw Man**: Misrepresenting opposing arguments
- **Slippery Slope**: Assuming extreme consequences
- **Appeal to Authority**: Unjustified authority claims
- **Bandwagon Fallacy**: "Everyone believes it"
- **Circular Reasoning**: Using conclusion as premise

### Problematic Assumptions
- **Unstated Premises**: Hidden assumptions
- **Cultural Bias**: Culturally-specific reasoning
- **Temporal Bias**: Assuming current conditions persist
- **Scope Creep**: Expanding beyond original context

## Usage Examples

### Basic Bias Check

```json
{
  "tool": "hybrid_biased_reasoning",
  "arguments": {
    "reasoning_step": "Since most successful startups are in Silicon Valley, we should only invest in companies located there."
  }
}
```

**Expected Detection**: Hasty generalization, selection bias

### With File Context

```json
{
  "tool": "hybrid_biased_reasoning",
  "arguments": {
    "reasoning_step": "This function is secure because it validates user input",
    "file_paths": [
      "/project/src/auth.py",
      "/project/src/validation.py"
    ],
    "context": "Security review of authentication system",
    "session_id": "security-review-001"
  }
}
```

**Benefit**: External LLM can see the actual code to verify if the reasoning about security is accurate

### Multi-Step Analysis

```json
{
  "tool": "hybrid_biased_reasoning",
  "arguments": {
    "reasoning_step": "Based on the previous analysis, we should proceed with Option A",
    "context": "Choosing between three investment strategies",
    "step_number": 3,
    "previous_steps": [
      "Option A has the highest historical returns",
      "Option B is too risky based on one bad year"
    ],
    "session_id": "investment-analysis",
    "bias_check_model": "gpt-4o"
  }
}
```

### Targeted Bias Detection

```json
{
  "tool": "hybrid_biased_reasoning",
  "arguments": {
    "reasoning_step": "We should hire this candidate because they went to an Ivy League school",
    "bias_types": ["appeal_to_authority", "halo_effect", "anchoring"],
    "temperature": 0.2
  }
}
```

## File Context Management

### Independent Context

The hybrid_biased_reasoning tool maintains its own file context separate from Claude's, ensuring:
- **Independent Verification**: External LLM sees files directly, not Claude's interpretation
- **Session Persistence**: Files loaded once are cached for the session
- **Smart Truncation**: Large files are truncated to 5000 chars to avoid token limits
- **Graceful Failures**: Missing files don't stop the bias check

### File Reading Strategy

```python
# Example: Code review with file context
def review_with_context(reasoning, related_files):
    response = hybrid_biased_reasoning({
        "reasoning_step": reasoning,
        "file_paths": related_files,
        "session_id": "code-review-session",
        "include_file_contents": True  # Default, but explicit is better
    })
    
    # Files are now cached in session
    # Subsequent calls with same session_id can access them
    return response
```

### Session File Management

```python
# Check what files are in the session context
session_files = tool.get_session_files("code-review-session")
print(f"Files loaded: {session_files.keys()}")

# Get session summary including file info
summary = tool.get_session_summary("code-review-session")
print(summary)
# Output: Session 'code-review-session': 5 steps checked, 2 biases found, 
#         average bias score: 0.35, 3 files loaded
#         Files in context: auth.py, validation.py, test_auth.py
```

## Integration Patterns

### With Sequential Thinking

```python
# Claude provides reasoning
reasoning = sequential_thinking("Analyze market opportunity")

# Check reasoning for bias
bias_check = hybrid_biased_reasoning({
    "reasoning_step": reasoning.content,
    "step_number": reasoning.thought_number,
    "session_id": "market-analysis"
})

if bias_check.revision_recommended:
    # Revise the reasoning
    revised = sequential_thinking_revision(
        bias_check.alternative_phrasing
    )
```

### With Traced Reasoning

```python
# Use traced reasoning for deep analysis
traced = traced_reasoning("Complex problem analysis")

# Check each step for bias
for i, step in enumerate(traced.steps):
    bias_result = hybrid_biased_reasoning({
        "reasoning_step": step.content,
        "step_number": i + 1,
        "previous_steps": traced.steps[:i],
        "session_id": traced.session_id
    })
    
    if bias_result.bias_score > 0.6:
        # High bias detected, flag for review
        flag_for_human_review(step, bias_result)
```

### Continuous Monitoring

```python
class BiasMonitoredReasoning:
    def __init__(self):
        self.session_id = generate_session_id()
        self.reasoning_steps = []
        self.bias_scores = []
    
    def add_reasoning(self, step):
        # Store reasoning
        self.reasoning_steps.append(step)
        
        # Check for bias
        bias_check = hybrid_biased_reasoning({
            "reasoning_step": step,
            "step_number": len(self.reasoning_steps),
            "previous_steps": self.reasoning_steps[:-1],
            "session_id": self.session_id
        })
        
        self.bias_scores.append(bias_check.bias_score)
        
        # Alert if bias trend is increasing
        if len(self.bias_scores) > 3:
            recent_trend = self.bias_scores[-3:]
            if all(recent_trend[i] < recent_trend[i+1] 
                   for i in range(2)):
                alert("Increasing bias trend detected")
        
        return bias_check
```

## Performance Considerations

### Latency
- **Additional API Call**: Adds 1-3 seconds per check
- **Model-Dependent**: Faster with smaller models (gpt-4o-mini)
- **Batch Processing**: Check multiple steps together when possible

### Cost
- **Double Token Usage**: Uses tokens for both reasoning and checking
- **Model Selection**: Balance quality vs cost
  - High stakes: Use gpt-4o or o3-mini
  - Routine checks: Use gpt-4o-mini
  - Budget mode: Sample checks (every Nth step)

### Optimization Strategies

1. **Selective Checking**: Only check critical reasoning steps
2. **Batch Analysis**: Combine multiple steps in one check
3. **Caching**: Cache bias checks for identical reasoning
4. **Threshold-Based**: Only check when confidence is low
5. **Progressive Enhancement**: Start with fast models, escalate if needed

## Best Practices

### DO:
1. **Provide Context**: Include original query for better analysis
2. **Include Relevant Files**: Add file_paths when reasoning about code/documents
3. **Track Sessions**: Use session IDs for pattern detection and file caching
4. **Include History**: Provide previous steps for context
5. **Set Temperature Low**: Use 0.2-0.3 for consistent bias detection
6. **Act on High Scores**: Revise when bias_score > 0.5
7. **Reuse Sessions**: Files are cached per session, reducing redundant reads

### DON'T:
1. **Check Everything**: Not every thought needs bias checking
2. **Include Huge Files**: Files > 5000 chars are truncated automatically
3. **Ignore Suggestions**: The suggestions are actionable improvements
4. **Use High Temperature**: Avoid temperature > 0.5 for bias checking
5. **Skip Context**: Bias detection needs context to be accurate
6. **Rely Solely on Score**: Consider the specific biases found
7. **Mix Sessions**: Keep related reasoning in the same session for context

## Configuration

### Environment Variables
```bash
# Default model for mini/fast operations
export LUX_MODEL_MINI="gpt-5-mini"

# API Keys
export OPENAI_API_KEY="your-key"
export OPENROUTER_API_KEY="your-key"
```

### Model Recommendations

| Use Case | Recommended Model | Why |
|----------|------------------|-----|
| Critical Analysis | gpt-4o | High accuracy, comprehensive |
| Standard Checking | gpt-4o-mini | Good balance of speed/quality |
| Research Work | o3-mini | Deep reasoning capabilities |
| High Volume | gpt-3.5-turbo | Fast and cheap |
| Specialized | Claude (via OpenRouter) | Different perspective |

## Limitations

1. **False Positives**: May flag valid reasoning as biased
2. **Context Sensitivity**: Needs sufficient context for accuracy
3. **Cultural Nuance**: May not detect all cultural biases
4. **Latency Impact**: Adds overhead to reasoning pipeline
5. **Model Dependence**: Quality depends on bias-checking model

## Troubleshooting

### Common Issues

**"No bias detected" for obvious bias:**
- Check temperature (should be low)
- Provide more context
- Try a different model
- Include specific bias_types to check

**High false positive rate:**
- Lower the revision threshold
- Provide more previous_steps
- Use a more sophisticated model
- Adjust bias_types parameter

**Inconsistent results:**
- Set temperature to 0.2 or lower
- Use the same model consistently
- Provide session_id for tracking

## Future Enhancements

- **Bias Pattern Learning**: Learn from correction patterns
- **Custom Bias Definitions**: User-defined bias types
- **Streaming Analysis**: Real-time bias detection
- **Bias Explanation**: Detailed reasoning for detections
- **Multi-Model Consensus**: Use multiple models for verification
- **Bias Metrics Dashboard**: Visualization of bias trends

## Related Tools

- **biased_reasoning**: Full dual-model reasoning with bias detection
- **traced_reasoning**: Reasoning with metacognitive monitoring
- **sequential_thinking**: Manual step-by-step reasoning
- **sequential_thinking_external**: AI-powered sequential reasoning

## Summary

The hybrid_biased_reasoning tool provides a powerful way to ensure reasoning quality by combining Claude's capabilities with external verification. Its unique ability to read files directly and maintain independent context from Claude ensures truly unbiased verification - the external LLM sees the actual files, not Claude's interpretation of them. This makes it particularly valuable for:

- **Code Reviews**: Verifying security claims against actual implementation
- **Document Analysis**: Checking reasoning against source documents
- **High-Stakes Decisions**: Independent verification with full context
- **Research Work**: Ensuring conclusions match the data

While it adds some latency and cost, the improvement in reasoning quality, bias awareness, and independent verification often justifies these trade-offs.