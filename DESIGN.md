# Lux MCP - Design Document

## Overview

Lux MCP is a Model Context Protocol server that implements metacognitive monitoring for AI reasoning while maintaining and refining context across conversations. It provides tools to detect and prevent overthinking spirals, circular reasoning, and distractor fixation through real-time analysis of thought patterns, while preserving and evolving understanding across tool interactions.

## Core Design Principles

1. **Tool Specialization**: Focused tools with clear purposes
2. **Context Continuity**: Conversations flow across tool boundaries
3. **Dual Refinement**: Both understanding and quality improve over time
4. **Transparent Monitoring**: Clear about what's implemented vs placeholder
5. **Performance First**: Built in Rust for minimal overhead
6. **Model Agnostic**: Works with any LLM through unified interface

## Architecture

### Current Architecture
```
┌─────────────────────────────────────────────────────────────┐
│                    MCP Protocol Layer                        │
│                  (stdio transport, JSON-RPC)                 │
├─────────────────────────────────────────────────────────────┤
│                      Session Manager                         │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐           │
│  │ Session A   │  │ Session B   │  │ Session C   │          │
│  │ Monitor A   │  │ Monitor B   │  │ Monitor C   │  ...     │
│  │ TTL: 30min  │  │ TTL: 30min  │  │ TTL: 30min  │          │
│  └────────────┘  └────────────┘  └────────────┘           │
├─────────────────────────────────────────────────────────────┤
│                         Tools                                │
│  ┌────────────┐  ┌──────────────┐  ┌──────────────┐       │
│  │   confer    │  │    traced     │  │    biased     │     │
│  │            │  │  reasoning    │  │  reasoning    │     │
│  └────────────┘  └──────────────┘  └──────────────┘       │
├─────────────────────────────────────────────────────────────┤
│                    LLM Client Layer                          │
│  ┌────────────────────────┐  ┌─────────────────────┐       │
│  │      OpenAI Client      │  │  OpenRouter Client   │      │
│  │  (GPT-4, O3, O4-mini)   │  │  (Claude, Gemini)    │      │
│  └────────────────────────┘  └─────────────────────┘       │
└─────────────────────────────────────────────────────────────┘
```

### Target Architecture with Context Refinement
```
┌─────────────────────────────────────────────────────────────┐
│                    MCP Protocol Layer                        │
│                  (stdio transport, JSON-RPC)                 │
├─────────────────────────────────────────────────────────────┤
│                 Context Refinement Layer                     │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  Thread Manager (Cross-Tool Conversation Memory)     │   │
│  │  • UUID-based thread identification                  │   │
│  │  • Context reconstruction and enhancement            │   │
│  │  • 3-hour TTL with automatic cleanup                 │   │
│  └─────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  Synthesis Evolution (Understanding Refinement)      │   │
│  │  • Progressive understanding development             │   │
│  │  • Insight accumulation and connection               │   │
│  │  • Confidence trajectory tracking                    │   │
│  └─────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────┤
│                 Quality Assurance Layer                      │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  Metacognitive Monitoring (Per-Session)              │   │
│  │  • Circular reasoning detection                      │   │
│  │  • Distractor fixation prevention                    │   │
│  │  • Quality degradation tracking                      │   │
│  └─────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────┤
│                    Context-Aware Tools                       │
│  ┌────────────┐  ┌──────────────┐  ┌──────────────┐       │
│  │   confer    │  │    traced     │  │    biased     │     │
│  │ +threading  │  │  reasoning    │  │  reasoning    │     │
│  │            │  │  +threading   │  │  +threading   │     │
│  └────────────┘  └──────────────┘  └──────────────┘       │
├─────────────────────────────────────────────────────────────┤
│                 Unified Persistence Layer                    │
│  ┌──────────────────┐  ┌────────────────────────────┐      │
│  │  In-Memory Store  │  │  Database (SeaORM/SQLite)  │      │
│  │  • Thread context │  │  • Synthesis states        │      │
│  │  • Hot cache      │  │  • Historical insights     │      │
│  └──────────────────┘  └────────────────────────────┘      │
└─────────────────────────────────────────────────────────────┘
```

## Tool Design

### 1. confer - Simple Chat Interface

**Purpose**: Basic conversational AI with model selection flexibility.

**Design Rationale**:
- Minimal overhead for simple queries
- No monitoring for basic interactions
- Model selection per request

**Request Structure**:
```rust
pub struct ChatRequest {
    pub message: String,
    pub model: Option<String>,      // Defaults to LUX_DEFAULT_CHAT_MODEL
    pub session_id: Option<String>, // For conversation continuity
}
```

### 2. traced_reasoning - Monitored Step-by-Step Reasoning

**Purpose**: Complex reasoning with real-time metacognitive monitoring.

**Design Features**:
- Step-by-step thought tracking
- Real-time intervention system
- Quality metrics per step
- Guardrail configuration

**Request Structure**:
```rust
pub struct TracedReasoningRequest {
    pub query: String,
    pub model: Option<String>,
    pub max_steps: u32,             // Default: 10
    pub temperature: f32,           // Default: 0.7
    pub guardrails: GuardrailConfig,
    pub session_id: Option<String>,
}

pub struct GuardrailConfig {
    pub semantic_drift_check: bool,        // Default: true
    pub circular_reasoning_detection: bool, // Default: true
    pub perplexity_monitoring: bool,       // Default: true
}
```

**Monitoring Integration**:
```rust
// Per-step monitoring
let signals = monitor.analyze_thought(&thought, step_number);
if signals.intervention.is_some() {
    // Inject guidance into reasoning flow
}
```

### 3. biased_reasoning - Dual-Model Verification

**Purpose**: Critical reasoning with bias detection and correction.

**Design Features**:
- Primary model for reasoning
- Verifier model for bias checking
- Per-step bias analysis
- Corrected thoughts when needed

**Request Structure**:
```rust
pub struct BiasedReasoningRequest {
    pub query: String,
    pub primary_model: Option<String>,
    pub verifier_model: Option<String>,
    pub max_steps: u32,
    pub bias_config: BiasCheckConfig,
    pub session_id: Option<String>,
}
```

## Session Management Design

### Rationale

1. **Isolation**: Each conversation needs independent monitoring state
2. **Concurrency**: Multiple clients can use the server simultaneously
3. **Cleanup**: Automatic resource management for long-running servers

### Implementation

```rust
pub struct SessionManager {
    sessions: Arc<Mutex<HashMap<String, SessionData>>>,
    ttl: Duration, // 30 minutes
}

pub struct SessionData {
    pub monitor: Arc<Mutex<MetacognitiveMonitor>>,
    pub last_accessed: Instant,
    pub created_at: Instant,
}

// Automatic cleanup task
tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(300));
    loop {
        interval.tick().await;
        session_manager.cleanup_expired_sessions();
    }
});
```

## Monitoring Algorithms

### Current Implementations

#### 1. Circular Reasoning Detection
```rust
// Basic implementation using thought history
fn check_circular_reasoning(&self, current: &str) -> f64 {
    let mut max_similarity = 0.0;
    for previous in self.thought_history.iter().rev().take(5) {
        let similarity = calculate_word_overlap(current, previous);
        max_similarity = max_similarity.max(similarity);
    }
    max_similarity
}
```

#### 2. Quality Trend Analysis
```rust
fn analyze_quality_trend(&self) -> String {
    // Simple heuristic based on thought count
    match self.thought_history.len() {
        0..=3 => "improving",
        4..=7 => "stable",
        _ => "degrading",
    }
}
```

### Placeholder Implementations

These features currently return mock values:

1. **Perplexity Monitoring**
   - Current: Returns `20.0 + (thought.len() / 100.0)`
   - Future: Actual language model perplexity calculation

2. **Attention Entropy**
   - Current: Returns constant `0.7`
   - Future: Attention pattern analysis

3. **Semantic Similarity**
   - Current: Basic word overlap
   - Future: Embedding-based similarity

## LLM Client Abstraction

### Design Goals

1. **Unified Interface**: Same API for all LLM providers
2. **Model Aliasing**: User-friendly shortcuts
3. **Error Handling**: Consistent error types
4. **Async First**: All operations are async

### Implementation

```rust
#[async_trait]
pub trait LLMClient: Send + Sync {
    async fn complete(
        &self,
        messages: Vec<ChatMessage>,
        temperature: Option<f32>,
        max_tokens: Option<usize>,
    ) -> Result<LLMResponse>;
}

// Model resolution
pub struct ModelResolver {
    aliases: HashMap<String, String>,
}

// Examples:
// "claude" → "anthropic/claude-4-sonnet"
// "o3" → "o3"
// "gemini" → "google/gemini-2.5-pro"
```

## Design Decisions

### Why Three Tools?

1. **Separation of Concerns**: Each tool has distinct use cases
2. **Performance**: Simple queries don't need monitoring overhead
3. **Flexibility**: Users choose the right tool for their task
4. **Clarity**: Clear when monitoring is active

### Why Session-Based?

1. **Isolation**: Conversations don't interfere
2. **Scalability**: Supports multiple concurrent users
3. **State Management**: Clean lifecycle for monitors
4. **Resource Control**: Automatic cleanup prevents leaks

### Why Rust?

1. **Performance**: Near-zero overhead for monitoring
2. **Safety**: Memory and thread safety guaranteed
3. **Async**: Natural fit for concurrent operations
4. **No GC**: Predictable latency

## Current Limitations

1. **Monitoring Algorithms**: Many are placeholders
2. **Context Length**: No handling of very long conversations
3. **Persistence**: Sessions are memory-only
4. **Analytics**: No aggregated insights across sessions

## Future Roadmap

### Phase 1: Context Threading (High Priority)
- [ ] UUID-based thread management
- [ ] Cross-tool conversation memory
- [ ] Context reconstruction from threads
- [ ] Thread persistence (3-hour TTL)

### Phase 2: Synthesis Evolution
- [ ] Link synthesis to conversation threads
- [ ] Insight graph implementation
- [ ] Assumption tracking
- [ ] Evidence chain building

### Phase 3: Algorithm Implementation
- [ ] Real semantic similarity using embeddings
- [ ] Actual perplexity calculation
- [ ] Attention entropy analysis
- [ ] Improved circular reasoning detection

### Phase 4: Unified Context System
- [ ] Merge conversation, synthesis, and quality contexts
- [ ] Cross-pollination between systems
- [ ] Hybrid persistence (memory + database)
- [ ] Context-aware prompting

### Phase 5: Production Features
- [ ] Analytics and metrics
- [ ] Rate limiting
- [ ] Health checks
- [ ] Cross-session learning

## Testing Strategy

1. **Unit Tests**: Each module has tests
2. **Integration Tests**: Tool end-to-end testing
3. **Protocol Tests**: MCP compliance verification
4. **Performance Tests**: Overhead measurement

## Security Considerations

1. **API Key Management**: Never logged or exposed
2. **Session Isolation**: No cross-session data leakage
3. **Input Validation**: All requests validated
4. **Error Handling**: No sensitive data in errors

## Performance Targets

1. **Monitoring Overhead**: <100ms per thought
2. **Session Operations**: O(1) lookups
3. **Memory Usage**: <10MB per session
4. **Concurrent Sessions**: 1000+ supported