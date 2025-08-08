# Thought Model Structures in Lux MCP

This document outlines the data structures used by the three main reasoning tools in Lux MCP.

## 1. Chat Tool (`confer`)

### Request Structure
```rust
ChatRequest {
    message: String,              // The message to send to the AI
    model: Option<String>,        // Optional model selection
    temperature: Option<f32>,     // Optional temperature (0.0-1.0)
    max_tokens: Option<u32>,      // Optional max tokens
    session_id: Option<String>,   // Optional session tracking
}
```

### Response Structure
```rust
ChatResponse {
    content: String,              // The AI's response
    model: String,                // Model that was used
    usage: Option<TokenUsage>,    // Token usage statistics
}

TokenUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}
```

## 2. Traced Reasoning Tool (`traced_reasoning`)

### Request Structure
```rust
TracedReasoningRequest {
    // Core fields
    thought: String,                      // Query for thought 1, guidance for 2+
    thought_number: u32,                  // Current thought number (starts at 1)
    total_thoughts: u32,                  // Estimated total thoughts needed
    next_thought_needed: bool,            // Whether another thought is required
    
    // Revision fields
    is_revision: bool,                    // True if revising a previous thought
    revises_thought: Option<u32>,         // Which thought is being revised
    
    // Branching fields
    branch_from_thought: Option<u32>,     // Branching point
    branch_id: Option<String>,            // Branch identifier
    
    // Extension fields
    needs_more_thoughts: bool,            // More thoughts needed beyond estimate
    
    // Configuration
    session_id: Option<String>,           // Session tracking
    model: Option<String>,                // Model selection
    temperature: f32,                     // Temperature (default: 0.7)
    guardrails: GuardrailConfig,         // Monitoring configuration
}

GuardrailConfig {
    semantic_drift_check: bool,          // Check for semantic drift (default: true)
    semantic_drift_threshold: f32,       // Drift threshold (default: 0.3)
    perplexity_monitoring: bool,         // Monitor perplexity (default: true)
    perplexity_threshold: f32,           // Perplexity threshold (default: 50.0)
    circular_reasoning_detection: bool,  // Detect circular reasoning (default: true)
    consistency_validation: bool,        // Validate consistency (default: true)
    attention_entropy_analysis: bool,    // Analyze attention entropy (default: true)
}
```

### Response Structure
```rust
TracedReasoningResponse {
    // Core fields
    status: String,                       // "thinking", "intervention_needed", "conclusion_reached"
    thought_number: u32,
    total_thoughts: u32,
    next_thought_needed: bool,
    thought_content: String,              // The actual thought content
    thought_type: StepType,               // Type of reasoning step
    
    // Metrics
    metrics: StepMetrics,                 // Metrics for this step
    metadata: TracedReasoningMetadata,    // Additional metadata
    
    // Optional fields
    continuation_id: Option<String>,      // For continuing sessions
    reasoning_complete: Option<bool>,     // Whether reasoning is complete
    final_answer: Option<String>,         // Final answer if complete
    next_steps: Option<String>,           // Suggested next steps
    intervention: Option<Intervention>,   // If intervention needed
    overall_metrics: Option<ReasoningMetrics>,  // Overall reasoning metrics
    model_used: Option<String>,           // Model that was used
    synthesis_snapshot: Option<SynthesisSnapshot>,  // Current synthesis state
}

StepType (enum) {
    Initial,      // Starting point
    Exploration,  // Exploring possibilities
    Analysis,     // Analyzing information
    Synthesis,    // Combining insights
    Validation,   // Validating conclusions
    Conclusion,   // Final conclusion
}

StepMetrics {
    semantic_similarity: Option<f32>,    // Similarity to previous steps
    perplexity: Option<f32>,             // Perplexity score
    attention_entropy: Option<f32>,      // Attention distribution entropy
    consistency_score: Option<f32>,      // Logical consistency
}

TracedReasoningMetadata {
    thought_history_length: u32,         // Number of thoughts so far
    interventions_count: u32,            // Number of interventions
    semantic_coherence: f32,             // Overall coherence score
    current_confidence: f32,             // Current confidence level
    is_revision: bool,
    revises_thought: Option<u32>,
    branch_id: Option<String>,
    needs_more_thoughts: bool,
}

SynthesisSnapshot {
    current_understanding: String,       // Current synthesis
    key_insights: Vec<String>,          // Key insights collected
    next_actions: Vec<String>,          // Suggested actions
    confidence_level: String,           // "low", "medium", "high"
    clarity_level: String,              // Clarity assessment
    ready_for_conclusion: bool,         // Ready to conclude
}

Intervention {
    step: u32,                          // Step where intervention occurred
    intervention_type: InterventionType,
    description: String,
    severity: Severity,                 // Low, Medium, High, Critical
}

InterventionType (enum) {
    SemanticDrift,      // Drifting from original topic
    HighPerplexity,     // Confused or uncertain
    CircularReasoning,  // Repeating same ideas
    InconsistentLogic,  // Logical inconsistencies
    AttentionScatter,   // Unfocused attention
    HallucinationRisk,  // Risk of hallucination
}
```

## 3. Biased Reasoning Tool (`biased_reasoning`)

### Request Structure
```rust
BiasedReasoningRequest {
    query: String,                        // The question to analyze
    session_id: Option<String>,           // Continue existing session
    new_session: bool,                    // Force new session
    max_analysis_rounds: Option<u32>,     // Max rounds (default: 3)
    primary_model: Option<String>,        // Primary reasoning model
    verifier_model: Option<String>,       // Bias checking model
}
```

### Response Structure (Step-by-Step)
```rust
BiasedReasoningResponse {
    // Step information
    step_type: StepType,                  // Type of this step
    step_number: u32,                     // Current step number
    content: String,                      // Step content
    model_used: String,                   // Model used for this step
    next_action: NextAction,              // What happens next
    
    // Session tracking
    session_id: String,                   // Session identifier
    session_status: SessionStatus,        // Overall session status
    
    // Optional fields
    bias_detected: Option<BiasInfo>,      // Bias information if detected
    synthesis_snapshot: Option<SynthesisSnapshot>,  // Current synthesis
    final_synthesis: Option<String>,      // Final synthesis if complete
}

StepType (enum) {
    Query,          // Initial question
    Reasoning,      // Primary model reasoning
    BiasAnalysis,   // Bias check result (VISIBLE)
    Correction,     // Corrected reasoning (VISIBLE)
    Guidance,       // User input/guidance
    Synthesis,      // Final compilation
}

NextAction (enum) {
    BiasCheck,           // Next: bias analysis
    ContinueReasoning,   // Next: more reasoning
    RequestGuidance,     // Next: ask for guidance
    Synthesize,          // Next: create synthesis
    Complete,            // Analysis complete
}

SessionStatus {
    total_steps: u32,                    // Total steps so far
    reasoning_rounds: u32,               // Reasoning rounds completed
    biases_detected: u32,                // Number of biases found
    corrections_made: u32,               // Number of corrections
    is_complete: bool,                   // Session complete
}

BiasInfo {
    bias_type: String,                   // Type of bias detected
    severity: String,                    // "low", "medium", "high"
    description: String,                 // Explanation
    suggested_correction: String,        // How to correct
}

SynthesisSnapshot {
    current_understanding: String,       // Current synthesis
    top_insights: Vec<String>,          // Top 5 insights
    next_actions: Vec<String>,          // High-priority actions
    confidence_level: String,           // "low", "medium", "high"
    ready_for_decision: bool,           // Ready to decide
}
```

## 4. Planner Tool (`planner`)

### Request Structure
```rust
PlannerRequest {
    // Core fields
    step: String,                        // Task for step 1, planning content for 2+
    step_number: u32,                    // Current step number (starts at 1)
    total_steps: u32,                    // Estimated total steps
    next_step_required: bool,            // Whether another step is needed
    
    // Revision fields
    is_step_revision: bool,              // Revising a previous step
    revises_step_number: Option<u32>,    // Which step is being revised
    
    // Branching fields
    is_branch_point: bool,               // This is a branch
    branch_from_step: Option<u32>,       // Branching from which step
    branch_id: Option<String>,           // Branch identifier
    
    // Extension fields
    more_steps_needed: bool,             // Need more steps than estimated
    
    // Configuration
    session_id: Option<String>,          // Session tracking
    model: Option<String>,               // Model selection
    temperature: f32,                    // Temperature (default: 0.7)
}
```

### Response Structure
```rust
PlannerResponse {
    // Core fields
    status: String,                      // "planning", "complete", etc.
    step_number: u32,
    total_steps: u32,
    next_step_required: bool,
    step_content: String,                // The planning step content
    metadata: PlannerMetadata,           // Additional metadata
    
    // Optional fields
    continuation_id: Option<String>,     // For continuing sessions
    planning_complete: Option<bool>,     // Planning complete
    plan_summary: Option<String>,        // Summary of plan
    next_steps: Option<String>,          // What to do next
    thinking_required: Option<bool>,     // Needs deep thinking
    required_thinking: Option<Vec<String>>,  // What needs thinking
    planner_required: Option<bool>,      // Needs more planning
    model_used: Option<String>,          // Model that was used
    synthesis_snapshot: Option<SynthesisSnapshot>,  // Current synthesis
}

PlannerMetadata {
    branches: Vec<String>,               // List of branch IDs
    step_history_length: u32,            // Number of steps so far
    is_step_revision: bool,
    revises_step_number: Option<u32>,
    is_branch_point: bool,
    branch_from_step: Option<u32>,
    branch_id: Option<String>,
    more_steps_needed: bool,
}

SynthesisSnapshot {
    current_plan: String,                // Current plan understanding
    key_decisions: Vec<String>,          // Key decisions made
    next_actions: Vec<String>,           // Next actions to take
    confidence_level: String,            // "low", "medium", "high"
    ready_for_execution: bool,           // Ready to execute plan
}
```

## Common Patterns

### 1. Session Management
All tools support optional `session_id` for maintaining context across multiple calls.

### 2. Model Selection
All tools support optional `model` parameter for selecting specific LLM models.

### 3. Synthesis Integration
All reasoning tools integrate with the synthesis engine to build evolving understanding:
- `SynthesisSnapshot` provides current state
- Insights and actions are tracked
- Confidence and clarity scores guide decisions

### 4. Step-by-Step Processing
- **Traced Reasoning**: Thought-by-thought with monitoring
- **Biased Reasoning**: Alternating reasoning and bias checking
- **Planner**: Sequential planning steps with branching

### 5. Metacognitive Monitoring
- Semantic drift detection
- Circular reasoning detection
- Perplexity monitoring
- Consistency validation
- Attention entropy analysis

## Usage Examples

### Traced Reasoning
```json
{
  "thought": "How can we optimize database queries in a microservices architecture?",
  "thought_number": 1,
  "total_thoughts": 5,
  "next_thought_needed": true,
  "model": "o3-pro",
  "temperature": 0.7
}
```

### Biased Reasoning
```json
{
  "query": "Should we migrate from monolith to microservices?",
  "max_analysis_rounds": 3,
  "primary_model": "o3-pro",
  "verifier_model": "o4-mini"
}
```

### Planner
```json
{
  "step": "Design a real-time collaborative editing system",
  "step_number": 1,
  "total_steps": 7,
  "next_step_required": true,
  "model": "o3-pro"
}
```

### Chat (Simple)
```json
{
  "message": "Explain the CAP theorem",
  "model": "gpt-4o",
  "temperature": 0.5,
  "max_tokens": 1000
}
```