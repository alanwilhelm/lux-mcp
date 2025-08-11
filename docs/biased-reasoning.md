# Biased Reasoning Tool Documentation

## Overview

The `biased_reasoning` tool implements a dual-model approach to reasoning where one model generates reasoning steps while another independently checks for biases, logical fallacies, and problematic assumptions. This step-by-step process provides transparency into both the reasoning and bias detection processes.

## Key Features

- **Dual-Model Architecture**: Primary reasoner + independent bias checker
- **Step-by-Step Visibility**: Each reasoning step followed by bias analysis
- **Automatic Correction**: Adjusts reasoning when biases are detected
- **Session Management**: Maintains context across multiple analysis rounds
- **File Reading**: Can analyze code/documents directly
- **Configurable Models**: Choose different models for each role
- **Transparent Process**: See exactly what each model contributes

## How It Works

1. **Query Processing**: Your question is sent to the primary reasoning model
2. **Reasoning Generation**: Primary model provides a reasoning step
3. **Bias Analysis**: Verifier model checks for biases and issues
4. **Correction (if needed)**: Primary model adjusts based on bias feedback
5. **Synthesis**: Final balanced conclusion after all rounds

Each step is returned individually, allowing you to see the entire thought process.

## When to Use

### Ideal For:
- **Controversial Topics**: Issues with potential for bias
- **Decision Making**: Important choices needing balanced analysis
- **Code Reviews**: Detecting architectural biases or assumptions
- **Research Analysis**: Ensuring objective interpretation
- **Policy Evaluation**: Checking for hidden assumptions

### Not Recommended For:
- Simple factual questions
- Time-critical tasks (multiple model calls add latency)
- High-volume processing (expensive token usage)
- Pure technical calculations

## API Reference

### Request Parameters

```typescript
interface BiasedReasoningRequest {
  // Required
  query: string;                      // The question to analyze
  
  // Optional
  session_id?: string;                // Session ID to continue existing analysis
  new_session?: boolean;              // Force start new session (default: false)
  max_analysis_rounds?: number;       // Maximum rounds (default: 3)
  primary_model?: string;             // Primary reasoning model
  verifier_model?: string;            // Bias checking model
  
  // Optional file reading
  file_paths?: string[];              // Files to read and include in context
  include_file_contents?: boolean;    // Whether to read files (default: true)
}
```

### Response Format (Per Step)

```typescript
interface BiasedReasoningResponse {
  step_type: StepType;                // Query, Reasoning, BiasAnalysis, Correction, Synthesis
  step_number: number;                // Sequential step counter
  content: string;                    // The actual content of this step
  model_used: string;                 // Which model generated this step
  next_action: NextAction;            // What happens next
  session_status: SessionStatus;      // Overall progress tracking
}

enum StepType {
  Query = "Query",                    // Initial question
  Reasoning = "Reasoning",            // Primary model reasoning
  BiasAnalysis = "BiasAnalysis",      // Bias checker analysis
  Correction = "Correction",          // Corrected reasoning
  Guidance = "Guidance",              // Additional guidance
  Synthesis = "Synthesis"             // Final synthesis
}

enum NextAction {
  BiasCheck = "BiasCheck",            // Next: check for biases
  ContinueReasoning = "ContinueReasoning", // Next: more reasoning
  Synthesize = "Synthesize",          // Next: final synthesis
  Complete = "Complete"               // Analysis complete
}

interface SessionStatus {
  session_id: string;
  total_steps: number;
  reasoning_steps: number;
  bias_checks: number;
  corrections_made: number;
  current_round: number;
  max_rounds: number;
}
```

## Usage Examples

### Basic Bias Analysis

```json
// Initial request
{
  "tool": "biased_reasoning",
  "arguments": {
    "query": "What programming language should beginners learn first?"
  }
}

// Response 1: Query acknowledgment
{
  "step_type": "Query",
  "step_number": 1,
  "content": "What programming language should beginners learn first?",
  "model_used": "system",
  "next_action": "ContinueReasoning",
  "session_status": {
    "session_id": "bias-abc123",
    "total_steps": 1,
    "reasoning_steps": 0,
    "bias_checks": 0,
    "corrections_made": 0,
    "current_round": 1,
    "max_rounds": 3
  }
}

// Continue with same query and session
{
  "tool": "biased_reasoning",
  "arguments": {
    "query": "What programming language should beginners learn first?",
    "session_id": "bias-abc123"
  }
}

// Response 2: Reasoning step
{
  "step_type": "Reasoning",
  "step_number": 2,
  "content": "Python is often recommended because...",
  "model_used": "o3-pro",
  "next_action": "BiasCheck",
  "session_status": {
    "total_steps": 2,
    "reasoning_steps": 1,
    // ...
  }
}

// Continue for bias check...
// Response 3: Bias analysis
{
  "step_type": "BiasAnalysis",
  "step_number": 3,
  "content": "Potential biases detected: 1) Popularity bias...",
  "model_used": "o4-mini",
  "next_action": "ContinueReasoning",
  // ...
}
```

### With File Context

```json
{
  "tool": "biased_reasoning",
  "arguments": {
    "query": "Review this API design for potential biases in user data handling",
    "file_paths": [
      "/api/users/model.py",
      "/api/users/routes.py",
      "/api/middleware/auth.py"
    ],
    "max_analysis_rounds": 4
  }
}
```

### Custom Model Selection

```json
{
  "tool": "biased_reasoning",
  "arguments": {
    "query": "Analyze the environmental impact of electric vehicles",
    "primary_model": "gpt-5",           // Deep reasoning
    "verifier_model": "claude",          // Different perspective
    "max_analysis_rounds": 5
  }
}
```

## Step-by-Step Process

### Round 1: Initial Analysis

1. **Query** - System acknowledges the question
2. **Reasoning** - Primary model provides initial analysis
3. **BiasAnalysis** - Verifier checks for biases
4. **Correction** (if needed) - Primary model adjusts

### Round 2-N: Deeper Analysis

1. **Reasoning** - Explores different angles
2. **BiasAnalysis** - Continued bias checking
3. **Guidance** - Verifier provides direction

### Final Round: Synthesis

1. **Synthesis** - Balanced final conclusion
2. **Complete** - Analysis finished

## Model Configuration

### Default Models

The tool uses environment-configured defaults:
- **Primary Reasoner**: `LUX_MODEL_REASONING` (default: gpt-5)
- **Bias Checker**: `LUX_MODEL_MINI` (default: gpt-5-mini)

### Recommended Combinations

| Use Case | Primary Model | Verifier Model | Rationale |
|----------|---------------|----------------|-----------|
| General Analysis | o3-pro | o4-mini | Deep reasoning + fast verification |
| Code Review | gpt-5 | claude | Technical depth + different perspective |
| Fast Iteration | gpt-4o | gpt-4o-mini | Balanced speed and quality |
| Maximum Quality | gpt-5 | gpt-5 | Highest quality both sides |
| Budget Conscious | gpt-4o-mini | gpt-4o-mini | Acceptable quality, low cost |

## Bias Types Detected

The verifier model checks for:

### Cognitive Biases
- **Confirmation Bias**: Favoring information that confirms existing beliefs
- **Anchoring Bias**: Over-relying on first piece of information
- **Availability Heuristic**: Overweighting easily recalled examples
- **Recency Bias**: Favoring recent events over historical data

### Logical Fallacies
- **Ad Hominem**: Attacking the person not the argument
- **Straw Man**: Misrepresenting opposing positions
- **False Dichotomy**: Presenting only two options
- **Slippery Slope**: Assuming extreme consequences

### Domain-Specific Biases
- **Technology Stack Bias**: Favoring familiar technologies
- **Scale Bias**: Assuming all problems need "big" solutions
- **Complexity Bias**: Over-engineering or over-simplifying
- **Cultural Assumptions**: Hidden cultural preferences

## Best Practices

### 1. Query Formulation

```json
// Good: Specific and neutral
{
  "query": "Compare SQL and NoSQL databases for a social media application with 10M users"
}

// Poor: Leading or vague
{
  "query": "Why is MongoDB better than PostgreSQL?"
}
```

### 2. Session Management

Always continue sessions for complete analysis:

```python
def complete_bias_analysis(query, max_rounds=3):
    session_id = None
    results = []
    
    while True:
        response = call_tool("biased_reasoning", {
            "query": query,
            "session_id": session_id,
            "max_analysis_rounds": max_rounds
        })
        
        results.append(response)
        session_id = response["session_status"]["session_id"]
        
        if response["next_action"] == "Complete":
            break
    
    return results
```

### 3. Handling Corrections

When biases are detected and corrected:

```python
def process_bias_correction(responses):
    corrections = []
    
    for i, response in enumerate(responses):
        if response["step_type"] == "Correction":
            previous_reasoning = responses[i-2]["content"]
            bias_detected = responses[i-1]["content"]
            correction = response["content"]
            
            corrections.append({
                "original": previous_reasoning,
                "bias": bias_detected,
                "corrected": correction
            })
    
    return corrections
```

### 4. File Context Best Practices

- Include relevant files for complete context
- Limit to essential files (3-5 recommended)
- Ensure files are readable before calling
- Use for code reviews, documentation analysis

### 5. Cost Optimization

- Start with default 3 rounds
- Increase rounds only for complex topics
- Use faster models for initial exploration
- Reserve premium models for critical decisions

## Performance Considerations

### Response Times

| Round | Steps | Typical Duration |
|-------|-------|------------------|
| 1 | 3-4 | 15-45 seconds |
| 2 | 2-3 | 10-30 seconds |
| 3 | 2-3 | 10-30 seconds |
| Synthesis | 1 | 5-15 seconds |

**Total**: 40-120 seconds for complete analysis

### Token Usage

- **Per Reasoning Step**: 5K-10K tokens
- **Per Bias Check**: 3K-8K tokens
- **Total per Analysis**: 30K-60K tokens

## Comparison with Other Tools

| Feature | biased_reasoning | traced_reasoning | sequential_thinking | planner |
|---------|-----------------|------------------|--------------------|---------| 
| Bias Detection | ✅ Primary focus | ❌ | ❌ | ❌ |
| Dual Models | ✅ Always | ❌ | ❌ | ❌ |
| Step Visibility | ✅ Full | ✅ Full | ✅ Full | ✅ Full |
| Monitoring | Bias only | ✅ Full metacognitive | ❌ | ❌ |
| Speed | Slow | Medium | Fast | Medium |
| Cost | High | Medium-High | Low | Medium |

## Troubleshooting

### Common Issues

1. **"Session not found"**
   ```json
   // Solution: Use the session_id from previous response
   {
     "session_id": "bias-abc123"  // From session_status
   }
   ```

2. **"Analysis seems stuck"**
   - Check if you're continuing the session
   - Verify max_analysis_rounds isn't too high
   - Consider using faster models

3. **"No biases detected repeatedly"**
   - Query might be genuinely neutral
   - Try more specific or controversial topics
   - Consider different verifier model

4. **High latency**
   - Normal for dual-model approach
   - Use faster model combinations
   - Reduce max_analysis_rounds

### Debug Output

Enable detailed logging:

```bash
RUST_LOG=debug ./target/release/lux-mcp
```

## Advanced Usage

### Custom Bias Categories

Request specific bias checking:

```json
{
  "query": "Evaluate this hiring process",
  "session_id": "hiring-analysis",
  // In the query itself, specify focus areas:
  "query": "Evaluate this hiring process. Focus on: demographic biases, educational elitism, and cultural fit assumptions"
}
```

### Comparative Analysis

Use branching with sessions:

```python
def compare_approaches(base_query, approaches):
    results = {}
    
    for approach in approaches:
        query = f"{base_query} Specifically considering: {approach}"
        session_results = complete_bias_analysis(query)
        results[approach] = session_results
    
    return results

# Example
approaches = ["microservices", "monolith", "serverless"]
comparison = compare_approaches(
    "Best architecture for an e-commerce platform",
    approaches
)
```

### Integration with Other Tools

Combine with traced_reasoning for comprehensive analysis:

```python
def comprehensive_analysis(query):
    # First: Deep reasoning with monitoring
    reasoning = call_tool("traced_reasoning", {
        "thought": query,
        "thought_number": 1,
        "total_thoughts": 3,
        "next_thought_needed": True
    })
    
    # Then: Bias check the reasoning
    bias_check = complete_bias_analysis(
        f"Check this reasoning for biases: {reasoning['content']}"
    )
    
    return {
        "reasoning": reasoning,
        "bias_analysis": bias_check
    }
```

## Use Case Examples

### Technical Decision Making

```json
{
  "query": "Should we migrate from REST to GraphQL for our API?",
  "file_paths": [
    "/api/rest/routes.py",
    "/api/requirements.txt",
    "/docs/api-usage-stats.md"
  ],
  "max_analysis_rounds": 4
}
```

### Code Review

```json
{
  "query": "Review this authentication system for security biases and assumptions",
  "file_paths": [
    "/auth/login.py",
    "/auth/permissions.py",
    "/tests/test_auth.py"
  ],
  "primary_model": "gpt-5",
  "verifier_model": "claude"
}
```

### Research Analysis

```json
{
  "query": "Analyze the conclusion that 'remote work reduces productivity' based on recent studies",
  "max_analysis_rounds": 5,
  "primary_model": "o3-pro",
  "verifier_model": "gpt-5"
}
```

## Summary

The `biased_reasoning` tool provides a unique dual-model approach to ensuring balanced, unbiased analysis. By separating reasoning and bias detection into independent models, it offers transparency and correction mechanisms that single-model approaches cannot match.

Key takeaways:
- Best for controversial or high-stakes decisions
- Provides step-by-step visibility into reasoning and bias detection
- Higher latency and cost due to multiple model calls
- Configurable models for different perspectives
- Excellent for code reviews, research analysis, and decision making

For faster analysis without bias checking, use `traced_reasoning`. For simple conversations, use `confer`.