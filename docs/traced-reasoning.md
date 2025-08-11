# Traced Reasoning Tool Documentation

## Overview

The `traced_reasoning` tool is an advanced reasoning system that provides step-by-step thinking with real-time metacognitive monitoring. It detects and prevents common reasoning failures like circular reasoning, distractor fixation, and quality degradation while maintaining detailed synthesis tracking.

## Key Features

- **Step-by-Step Reasoning**: Breaks complex problems into numbered thought steps
- **Metacognitive Monitoring**: Real-time quality checks and intervention system
- **Circular Reasoning Detection**: Identifies when thoughts become repetitive (>85% similarity)
- **Distractor Fixation Prevention**: Alerts when drifting from original query (<30% relevance)
- **Quality Tracking**: Monitors coherence, clarity, and confidence metrics
- **Synthesis Integration**: Tracks insights, actions, and understanding progression
- **Session Management**: Maintains context across multiple reasoning steps
- **Branching & Revisions**: Explore alternatives and revise previous thoughts

## When to Use

### Ideal For:
- **Complex Analysis**: Multi-faceted problems requiring deep exploration
- **Critical Decisions**: High-stakes scenarios needing quality assurance
- **Research Tasks**: Literature review, hypothesis testing, data analysis
- **Architecture Design**: System design with trade-off analysis
- **Debugging Complex Issues**: Root cause analysis with multiple factors

### Not Recommended For:
- Simple Q&A or factual lookups
- Tasks with tight time constraints (O3 models can take 30+ seconds)
- High-volume repetitive tasks (high token usage)
- Tasks where monitoring overhead isn't justified

## API Reference

### Request Parameters

```typescript
interface TracedReasoningRequest {
  // Required
  thought: string;                    // Query (step 1) or guidance (2+)
  thought_number: number;             // Current thought number (starts at 1)
  total_thoughts: number;             // Estimated total thoughts needed
  next_thought_needed: boolean;       // Whether another thought is needed
  
  // Optional
  continuation_id?: string;           // Thread ID for conversation continuity
  session_id?: string;               // Session ID for monitoring state
  model?: string;                    // Model to use (default: LUX_MODEL_REASONING)
  temperature?: number;              // Temperature 0.0-1.0 (default: 0.7)
  
  // Optional file reading
  file_paths?: string[];             // Files to read and include in context
  include_file_contents?: boolean;   // Whether to read files (default: true)
  
  // Revision & Branching
  is_revision?: boolean;             // True if revising a previous thought
  revises_thought?: number;          // Which thought is being revised
  branch_from_thought?: number;      // Branching point for alternatives
  branch_id?: string;               // Branch identifier
  needs_more_thoughts?: boolean;     // Extend beyond initial estimate
  
  // Monitoring Configuration
  guardrails?: {
    semantic_drift_check?: boolean;           // Default: true
    semantic_drift_threshold?: number;        // Default: 0.3
    perplexity_monitoring?: boolean;          // Default: true
    perplexity_threshold?: number;            // Default: 50.0
    circular_reasoning_detection?: boolean;   // Default: true
    consistency_validation?: boolean;         // Default: true
    attention_entropy_analysis?: boolean;     // Default: true
  }
}
```

### Response Format

```typescript
interface TracedReasoningResponse {
  thought_number: number;
  content: string;
  thought_type: string;              // Initial, Exploration, Synthesis, etc.
  next_thought_needed: boolean;
  session_id: string;
  thread_id: string;
  
  monitoring: {
    circular_score: number;          // 0.0-1.0 (lower is better)
    distractor_alert: boolean;
    quality_trend: string;           // improving, stable, degrading
    phase: string;                   // exploration, synthesis, conclusion
    intervention?: string;           // Optional warning/guidance message
  };
  
  synthesis: {
    current_understanding: string;
    key_insights: string[];
    next_actions: string[];
    confidence_level: string;        // low, medium, high
    clarity_level: string;           // unclear, partial, clear
    ready_for_conclusion: boolean;
  };
  
  metrics: {
    semantic_coherence: number;      // 0.0-1.0
    information_density: number;     // 0.0-1.0
    reasoning_depth: number;         // 0.0-1.0
    confidence: number;              // 0.0-1.0
  };
}
```

## Usage Examples

### Basic Multi-Step Reasoning

```json
// Step 1: Initial query
{
  "tool": "traced_reasoning",
  "arguments": {
    "thought": "How can we optimize database performance in a microservices architecture?",
    "thought_number": 1,
    "total_thoughts": 5,
    "next_thought_needed": true
  }
}

// Step 2: Guided exploration
{
  "tool": "traced_reasoning",
  "arguments": {
    "thought": "Focus on query optimization and caching strategies",
    "thought_number": 2,
    "total_thoughts": 5,
    "next_thought_needed": true,
    "session_id": "perf-opt-123"
  }
}

// Step 3-4: Continue reasoning...

// Step 5: Final synthesis
{
  "tool": "traced_reasoning",
  "arguments": {
    "thought": "Synthesize recommendations into actionable plan",
    "thought_number": 5,
    "total_thoughts": 5,
    "next_thought_needed": false,
    "session_id": "perf-opt-123"
  }
}
```

### With File Context

```json
{
  "tool": "traced_reasoning",
  "arguments": {
    "thought": "Analyze the current authentication implementation for security vulnerabilities",
    "thought_number": 1,
    "total_thoughts": 4,
    "next_thought_needed": true,
    "file_paths": [
      "/app/auth/login.js",
      "/app/auth/session.js",
      "/app/middleware/auth.js"
    ]
  }
}
```

### Revision Example

```json
{
  "tool": "traced_reasoning",
  "arguments": {
    "thought": "Actually, we should reconsider the caching approach with Redis instead",
    "thought_number": 3,
    "total_thoughts": 5,
    "next_thought_needed": true,
    "is_revision": true,
    "revises_thought": 2,
    "session_id": "perf-opt-123"
  }
}
```

### Branching for Alternatives

```json
{
  "tool": "traced_reasoning",
  "arguments": {
    "thought": "Explore event-driven architecture as an alternative approach",
    "thought_number": 4,
    "total_thoughts": 6,
    "next_thought_needed": true,
    "branch_from_thought": 3,
    "branch_id": "event-driven-alternative",
    "session_id": "perf-opt-123"
  }
}
```

## Monitoring System

### Quality Metrics

The tool tracks four key quality dimensions:

1. **Semantic Coherence** (0.0-1.0)
   - Logical consistency between thoughts
   - Conceptual alignment with previous steps
   - Maintains focus on the problem domain

2. **Information Density** (0.0-1.0)
   - Amount of new information per thought
   - Avoids repetition and redundancy
   - Balances detail with conciseness

3. **Reasoning Depth** (0.0-1.0)
   - Sophistication of analysis
   - Consideration of edge cases
   - Exploration of implications

4. **Confidence** (0.0-1.0)
   - Certainty in conclusions
   - Based on evidence quality
   - Acknowledgment of limitations

### Intervention Triggers

The monitoring system intervenes when it detects:

- **Circular Reasoning**: Same concepts repeated 3+ times
- **Distractor Fixation**: Drifting off-topic for 2+ thoughts
- **Quality Degradation**: 40%+ drop in quality metrics
- **Stalled Progress**: No new insights for 3+ thoughts

### Phases of Reasoning

1. **Exploration Phase** (thoughts 1-40%)
   - Broad exploration of problem space
   - Gathering context and constraints
   - Identifying key challenges

2. **Synthesis Phase** (thoughts 40-80%)
   - Connecting insights
   - Forming hypotheses
   - Evaluating trade-offs

3. **Conclusion Phase** (thoughts 80-100%)
   - Finalizing recommendations
   - Summarizing key points
   - Defining next steps

## Model Selection

### Recommended Models

- **GPT-5**: Best overall performance, 128K tokens
- **O3-Pro**: Deep reasoning for complex problems
- **GPT-4o**: Balanced performance and speed
- **Claude (via OpenRouter)**: Alternative perspective

### Performance Considerations

| Model | Response Time | Token Usage | Best For |
|-------|--------------|-------------|----------|
| GPT-5 | 5-30s | ~128K/step | Complex analysis |
| O3-Pro | 30s-5min | ~32K/step | Critical reasoning |
| GPT-4o | 2-10s | ~10K/step | General tasks |
| Claude | 3-15s | ~10K/step | Creative problems |

## Best Practices

### 1. Thought Planning

```json
// Good: Clear progression
{
  "thought_number": 1, "thought": "Understand the problem domain"
  "thought_number": 2, "thought": "Analyze current limitations"
  "thought_number": 3, "thought": "Explore solution approaches"
  "thought_number": 4, "thought": "Evaluate trade-offs"
  "thought_number": 5, "thought": "Synthesize recommendations"
}

// Poor: Vague or repetitive
{
  "thought_number": 1, "thought": "Think about the problem"
  "thought_number": 2, "thought": "Think more about it"
  "thought_number": 3, "thought": "Consider options"
}
```

### 2. Session Management

Always maintain session IDs for multi-step reasoning:

```json
{
  "session_id": "analysis-2024-01-15-001",
  "continuation_id": "thread-abc-123"
}
```

### 3. Handling Interventions

When the monitor provides intervention messages, adjust your approach:

```json
// Response with intervention
{
  "monitoring": {
    "intervention": "Warning: Circular reasoning detected. Consider exploring different angles."
  }
}

// Next request should pivot
{
  "thought": "Let's approach this from a user experience perspective instead",
  "thought_number": 4,
  // ...
}
```

### 4. File Context Best Practices

- Include relevant files for code analysis
- Limit to 5-10 files per request
- Files are read once at request time
- Use specific paths, not glob patterns

### 5. Cost Optimization

- Start with estimated total_thoughts
- Use needs_more_thoughts sparingly
- Consider cheaper models for exploration
- Use premium models for critical synthesis

## Troubleshooting

### Common Issues

1. **"Session not found"**
   - Ensure consistent session_id across calls
   - Check for server restarts

2. **"Circular reasoning detected"**
   - Vary your guidance between thoughts
   - Introduce new perspectives
   - Use revision to break cycles

3. **High latency with O3 models**
   - Expected behavior (deep reasoning)
   - Consider GPT-4o for faster responses
   - Use async patterns in your application

4. **Token limit exceeded**
   - Reduce file_paths count
   - Summarize context in earlier thoughts
   - Use branching to explore separately

### Debug Mode

Enable detailed logging to troubleshoot:

```bash
RUST_LOG=debug ./target/release/lux-mcp
```

## Integration Patterns

### Sequential Processing

```python
def deep_analysis(query, steps=5):
    session_id = f"analysis-{uuid.uuid4()}"
    results = []
    
    for i in range(1, steps + 1):
        response = call_tool("traced_reasoning", {
            "thought": query if i == 1 else f"Continue analysis step {i}",
            "thought_number": i,
            "total_thoughts": steps,
            "next_thought_needed": i < steps,
            "session_id": session_id
        })
        
        results.append(response)
        
        # Check for interventions
        if response.get("monitoring", {}).get("intervention"):
            # Adjust strategy based on intervention
            pass
    
    return results
```

### Parallel Exploration

```python
def explore_alternatives(base_query, alternatives):
    base_session = f"explore-{uuid.uuid4()}"
    
    # Initial exploration
    base = call_tool("traced_reasoning", {
        "thought": base_query,
        "thought_number": 1,
        "total_thoughts": 3,
        "next_thought_needed": True,
        "session_id": base_session
    })
    
    # Branch for each alternative
    branches = []
    for alt in alternatives:
        branch = call_tool("traced_reasoning", {
            "thought": f"Explore alternative: {alt}",
            "thought_number": 2,
            "total_thoughts": 3,
            "next_thought_needed": True,
            "branch_from_thought": 1,
            "branch_id": f"alt-{alt}",
            "session_id": base_session
        })
        branches.append(branch)
    
    return base, branches
```

## Comparison with Other Tools

| Feature | traced_reasoning | sequential_thinking | planner | biased_reasoning |
|---------|-----------------|--------------------|---------|--------------------|
| AI-Powered | ✅ | Optional | ✅ | ✅ |
| Monitoring | ✅ Full | ❌ | ❌ | Bias only |
| File Reading | ✅ | ❌ | ✅ | ✅ |
| Branching | ✅ | ✅ | ✅ | ❌ |
| Speed | Medium | Fast | Medium | Slow |
| Token Usage | High | Low/None | Medium | Very High |
| Best For | Critical analysis | Manual control | Planning | Bias detection |

## Advanced Configuration

### Custom Guardrails

Fine-tune monitoring thresholds:

```json
{
  "guardrails": {
    "semantic_drift_threshold": 0.25,      // More strict (default: 0.3)
    "perplexity_threshold": 40.0,         // Lower tolerance (default: 50.0)
    "circular_reasoning_detection": true,
    "consistency_validation": true,
    "attention_entropy_analysis": false    // Disable for speed
  }
}
```

### Model-Specific Optimizations

```json
// For O3 models - maximize reasoning
{
  "model": "o3-pro",
  "temperature": 0.7,
  "total_thoughts": 7,  // More steps for deep reasoning
  "guardrails": {
    "semantic_drift_threshold": 0.35  // Allow more exploration
  }
}

// For GPT-4o - balance speed and quality
{
  "model": "gpt-4o",
  "temperature": 0.5,
  "total_thoughts": 4,  // Fewer steps for efficiency
  "guardrails": {
    "perplexity_threshold": 45.0  // Tighter control
  }
}
```

## Summary

The `traced_reasoning` tool is the most sophisticated reasoning tool in the Lux MCP arsenal, providing unparalleled quality assurance for critical thinking tasks. Use it when the cost of reasoning errors is high and you need confidence in the analytical process.

Key takeaways:
- Ideal for complex, high-stakes analysis
- Provides real-time quality monitoring
- Supports branching and revision for thorough exploration
- Higher latency and token usage than simpler tools
- Best with premium models (GPT-5, O3-Pro)

For simpler tasks, consider `sequential_thinking` or `confer`. For bias-specific analysis, use `biased_reasoning`.