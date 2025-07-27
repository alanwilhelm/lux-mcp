# Linear & Structured Thinking Tools in Lux-MCP

## Overview
We have three main tools that provide different approaches to structured and linear thinking:

## 1. **Traced Reasoning Tool** (`traced_reasoning`)
**Purpose**: Multi-call step-by-step reasoning with metacognitive monitoring and detailed output per thought

### Key Features:
- **Multi-call Sequential Thoughts**: Generate variable number of thoughts with detailed output for each
- **State Persistence**: Maintains thought history across calls
- **Multi-metric Monitoring**: Tracks semantic drift, perplexity, attention entropy
- **Real-time Interventions**: Detects and corrects reasoning issues per thought
- **Thought Types**: Initial → Exploration → Analysis → Synthesis → Validation → Conclusion
- **Branching & Revisions**: Support for alternative reasoning paths and thought corrections
- **Model Display**: Shows which model is used for each thought
- **Guardrails**:
  - Semantic drift check (threshold: 0.3)
  - Perplexity monitoring (threshold: 50.0)
  - Circular reasoning detection
  - Consistency validation
  - Attention entropy analysis

### Structure:
```json
{
  "thought": "What is the meaning of consciousness?",  // Thought 1: query, Thought 2+: guidance
  "thought_number": 1,
  "total_thoughts": 7,
  "next_thought_needed": true,
  "model": "o3",
  "temperature": 0.7,
  
  // Advanced features:
  "is_revision": true,
  "revises_thought": 3,
  "branch_from_thought": 2,
  "branch_id": "alternative-hypothesis",
  "needs_more_thoughts": true,
  
  "guardrails": {
    "semantic_drift_check": true,
    "circular_reasoning_detection": true,
    "perplexity_monitoring": true
  }
}
```

### Thought Flow:
1. **Thought 1**: User provides initial query/problem
2. **Thought 2+**: LLM generates reasoning content based on:
   - Previous thought history
   - User guidance in `thought` parameter
   - Context (branches, revisions, interventions)
3. **Each Thought Output**: Displays metrics, confidence, model used
4. **Interventions**: Real-time alerts when issues detected
5. **Completion**: Final answer with overall metrics

### Output Per Thought:
- Thought content with type classification
- **Model used for generation** (displayed as "Model: {model_name}")
- Confidence score and semantic coherence
- Intervention alerts if triggered
- Metrics dashboard
- Next action guidance

## 2. **Biased Reasoning Tool** (`biased_reasoning`)
**Purpose**: Dual-model reasoning with bias detection and correction

### Key Features:
- **Dual-Model Architecture**: Primary model reasons, verifier checks each step
- **Bias Detection**: Identifies multiple bias types:
  - Confirmation bias
  - Anchoring bias
  - Availability bias
  - Reasoning errors
  - Over-generalization
  - False equivalence
  - Circular reasoning
  - Hasty conclusions
- **Step Correction**: Generates corrected thoughts when bias is detected
- **Quality Metrics**: Tracks step quality and overall reasoning assessment

### Structure:
```json
{
  "query": "Question to analyze",
  "primary_model": "gpt-4",
  "verifier_model": "o4-mini",
  "max_steps": 10,
  "bias_config": {
    "check_confirmation_bias": true,
    "check_anchoring_bias": true,
    "bias_threshold": 0.7
  }
}
```

### Output:
- **Models displayed**: Shows primary and verifier models at start
- **Detailed process log**: Every action shows which model was used
- Final answer with dual verification
- Reasoning steps with bias annotations
- Corrected thoughts for biased steps
- Overall assessment with quality metrics
- Most common biases identified
- **Process log entries include**:
  - Action type and step number
  - Timestamp and duration
  - Model used for each action
  - Detailed content of the action

## 3. **Interactive Planner Tool** (`planner`)
**Purpose**: LLM-powered sequential planning with state management

### Key Features:
- **Interactive Planning**: Build plans step-by-step with LLM generation
- **State Persistence**: Maintains planning history across calls
- **Branching**: Explore alternative approaches
- **Revisions**: Update earlier steps based on new insights
- **Deep Thinking Pauses**: For complex plans (≥5 steps), enforces reflection
- **Metacognitive Monitoring**: Detects circular reasoning in planning

### Structure:
```json
{
  "step": "Build a distributed system",  // Step 1: goal, Step 2+: guidance
  "step_number": 1,
  "total_steps": 7,
  "next_step_required": true,
  "model": "o3-pro",  // Uses LLM to generate steps
  "temperature": 0.7,
  
  // Advanced features:
  "is_branch_point": true,
  "branch_id": "alternative-approach",
  "is_step_revision": true,
  "revises_step_number": 2
}
```

### Planning Flow:
1. **Step 1**: User provides goal/task description
2. **Step 2+**: LLM generates planning content based on:
   - Previous step history
   - User guidance in `step` parameter
   - Context (branches, revisions)
3. **Deep Thinking**: Steps 1-3 of complex plans require reflection
4. **Completion**: Generates comprehensive plan summary

### Output:
- Generated planning step content
- Status (pause_for_planner, pause_for_deep_thinking, planning_complete)
- Metadata (branches, revisions, history)
- Plan summary with full journey

## Comparison Table

| Feature | Traced Reasoning | Biased Reasoning | Planner |
|---------|-----------------|------------------|---------| 
| **Thinking Style** | Sequential thoughts | Dual-model verification | Sequential planning |
| **LLM Usage** | Single model with state | Two models (primary + verifier) | Single model with state |
| **Monitoring** | Multi-metric guardrails | Bias detection | Circular reasoning check |
| **Interactivity** | Multi-call stateful | Single call | Multi-call stateful |
| **Best For** | Deep reasoning analysis | Critical decisions | Multi-step planning |
| **Special Features** | Per-thought metrics & interventions | Bias correction | Branching & revisions |
| **Output Format** | Individual thought outputs | Verified steps | Planning journey |
| **State Management** | Thought history & branches | None (single call) | Step history & branches |

## Usage Recommendations

### Use **Traced Reasoning** when:
- You need deep, iterative analysis of a complex problem
- Want to see detailed metrics for each reasoning thought
- Need to monitor reasoning quality in real-time per thought
- Want to revise or branch reasoning paths dynamically
- Need intervention alerts during the reasoning process
- Solving technical or analytical problems step-by-step

### Use **Biased Reasoning** when:
- Making critical decisions that need verification
- Want to identify and correct cognitive biases
- Need dual-perspective analysis
- Evaluating proposals or controversial topics

### Use **Planner** when:
- Breaking down complex projects into steps
- Need interactive, iterative planning
- Want to explore alternative approaches
- Building implementation roadmaps

## Model Defaults
- **Chat**: `LUX_DEFAULT_CHAT_MODEL` (default: o3-pro)
- **Reasoning**: `LUX_DEFAULT_REASONING_MODEL` (default: o3-pro)
- **Bias Checker**: `LUX_DEFAULT_BIAS_CHECKER_MODEL` (default: o4-mini)

All tools support model override via the `model` parameter in requests.