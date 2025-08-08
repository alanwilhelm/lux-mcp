# Context Refinement Implementation Plan

## Objective
Implement a dual context refinement system in Lux MCP that combines conversation threading (from Zen) with synthesis evolution and quality monitoring (existing Lux features).

## Phase 1: Thread Management Foundation (Week 1)

### 1.1 Create Thread Manager Module
**File**: `src/threading/mod.rs`
```rust
pub mod manager;
pub mod context;
pub mod reconstruction;
```

### 1.2 Implement ThreadManager
**File**: `src/threading/manager.rs`
```rust
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::Mutex;
use uuid::Uuid;
use std::time::{Duration, Instant};

pub struct ThreadManager {
    threads: Arc<Mutex<HashMap<Uuid, ThreadContext>>>,
    ttl: Duration, // 3 hours like Zen
}

pub struct ThreadContext {
    pub thread_id: Uuid,
    pub tool_name: String,
    pub turns: Vec<ConversationTurn>,
    pub initial_files: Vec<String>,
    pub created_at: Instant,
    pub last_accessed: Instant,
}

pub struct ConversationTurn {
    pub role: Role,
    pub content: String,
    pub tool_used: Option<String>,
    pub synthesis_snapshot: Option<SynthesisState>,
    pub quality_metrics: Option<QualityMetrics>,
    pub timestamp: Instant,
}

impl ThreadManager {
    pub fn new() -> Self {
        Self {
            threads: Arc::new(Mutex::new(HashMap::new())),
            ttl: Duration::from_secs(3 * 60 * 60), // 3 hours
        }
    }
    
    pub fn create_thread(&self, tool_name: &str) -> Uuid {
        let thread_id = Uuid::new_v4();
        let context = ThreadContext {
            thread_id,
            tool_name: tool_name.to_string(),
            turns: Vec::new(),
            initial_files: Vec::new(),
            created_at: Instant::now(),
            last_accessed: Instant::now(),
        };
        
        let mut threads = self.threads.lock();
        threads.insert(thread_id, context);
        thread_id
    }
    
    pub fn get_thread(&self, id: &Uuid) -> Option<ThreadContext> {
        let mut threads = self.threads.lock();
        if let Some(context) = threads.get_mut(id) {
            context.last_accessed = Instant::now();
            Some(context.clone())
        } else {
            None
        }
    }
    
    pub fn add_turn(&self, id: &Uuid, turn: ConversationTurn) -> bool {
        let mut threads = self.threads.lock();
        if let Some(context) = threads.get_mut(id) {
            context.turns.push(turn);
            context.last_accessed = Instant::now();
            true
        } else {
            false
        }
    }
    
    pub fn cleanup_expired(&self) -> usize {
        let mut threads = self.threads.lock();
        let now = Instant::now();
        let before = threads.len();
        
        threads.retain(|_, context| {
            now.duration_since(context.last_accessed) < self.ttl
        });
        
        before - threads.len()
    }
}
```

### 1.3 Add Threading to Tool Responses
**Modify**: All tool response structures
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResponse {
    pub content: String,
    pub continuation_id: Option<Uuid>, // NEW FIELD
    pub metadata: ResponseMetadata,
}
```

### 1.4 Implement Context Reconstruction
**File**: `src/threading/reconstruction.rs`
```rust
impl ThreadManager {
    pub fn reconstruct_context(&self, thread_id: &Uuid) -> Option<String> {
        let threads = self.threads.lock();
        
        if let Some(context) = threads.get(thread_id) {
            let mut history = String::new();
            
            // Add conversation history
            for turn in &context.turns {
                history.push_str(&format!(
                    "\n[{}] {}: {}\n",
                    turn.timestamp.elapsed().as_secs(),
                    turn.role,
                    turn.content
                ));
                
                // Include synthesis snapshot if available
                if let Some(synthesis) = &turn.synthesis_snapshot {
                    history.push_str(&format!(
                        "Understanding: {}\n",
                        synthesis.current_understanding
                    ));
                }
                
                // Include quality metrics if available
                if let Some(metrics) = &turn.quality_metrics {
                    history.push_str(&format!(
                        "Quality: circular={:.2}, coherence={:.2}\n",
                        metrics.circular_reasoning_score,
                        metrics.coherence_score
                    ));
                }
            }
            
            Some(history)
        } else {
            None
        }
    }
}
```

## Phase 2: Tool Integration (Week 2)

### 2.1 Update Tool Handler
**File**: `src/server/handler.rs`
```rust
impl Handler {
    async fn handle_tool_call(&self, name: &str, args: Value) -> Result<Value> {
        // Extract continuation_id if present
        let continuation_id = args.get("continuation_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok());
        
        // Reconstruct context if continuation exists
        let context = if let Some(id) = continuation_id {
            self.thread_manager.reconstruct_context(&id)
        } else {
            None
        };
        
        // Create or get thread
        let thread_id = continuation_id.unwrap_or_else(|| {
            self.thread_manager.create_thread(name)
        });
        
        // Execute tool with context
        let response = match name {
            "confer" => self.handle_chat(args, context).await?,
            "traced_reasoning" => self.handle_traced(args, context).await?,
            "biased_reasoning" => self.handle_biased(args, context).await?,
            "planner" => self.handle_planner(args, context).await?,
            _ => return Err(anyhow!("Unknown tool: {}", name)),
        };
        
        // Add continuation_id to response
        let mut response_json = serde_json::to_value(response)?;
        response_json["continuation_id"] = json!(thread_id.to_string());
        
        // Store turn in thread
        let turn = ConversationTurn {
            role: Role::Assistant,
            content: response_json["content"].as_str().unwrap_or("").to_string(),
            tool_used: Some(name.to_string()),
            synthesis_snapshot: None, // TODO: Extract from response
            quality_metrics: None, // TODO: Extract from monitoring
            timestamp: Instant::now(),
        };
        self.thread_manager.add_turn(&thread_id, turn);
        
        Ok(response_json)
    }
}
```

### 2.2 Enhance Each Tool
**Example for traced_reasoning**:
```rust
impl TracedReasoningTool {
    pub async fn execute_with_context(
        &self,
        request: TracedReasoningRequest,
        context: Option<String>,
    ) -> Result<TracedReasoningResponse> {
        // Prepend context to query if available
        let enhanced_query = if let Some(ctx) = context {
            format!(
                "Previous conversation context:\n{}\n\nCurrent query: {}",
                ctx, request.query
            )
        } else {
            request.query.clone()
        };
        
        // Execute with enhanced query
        let mut request = request;
        request.query = enhanced_query;
        
        self.execute(request).await
    }
}
```

## Phase 3: Synthesis Integration (Week 3)

### 3.1 Link Synthesis to Threads
**File**: `src/threading/synthesis_bridge.rs`
```rust
impl ThreadManager {
    pub fn attach_synthesis(&self, thread_id: &Uuid, synthesis: &SynthesisState) {
        let mut threads = self.threads.lock();
        if let Some(context) = threads.get_mut(thread_id) {
            if let Some(last_turn) = context.turns.last_mut() {
                last_turn.synthesis_snapshot = Some(synthesis.clone());
            }
        }
    }
    
    pub fn get_synthesis_history(&self, thread_id: &Uuid) -> Vec<SynthesisState> {
        let threads = self.threads.lock();
        if let Some(context) = threads.get(thread_id) {
            context.turns
                .iter()
                .filter_map(|turn| turn.synthesis_snapshot.clone())
                .collect()
        } else {
            Vec::new()
        }
    }
}
```

### 3.2 Create Insight Graph
**File**: `src/synthesis/insight_graph.rs`
```rust
pub struct InsightGraph {
    nodes: Vec<Insight>,
    edges: Vec<InsightRelation>,
}

pub struct Insight {
    pub id: Uuid,
    pub content: String,
    pub confidence: f32,
    pub source_turn: usize,
    pub timestamp: Instant,
}

pub enum InsightRelation {
    Supports(Uuid, Uuid),
    Contradicts(Uuid, Uuid),
    Extends(Uuid, Uuid),
    Clarifies(Uuid, Uuid),
}

impl InsightGraph {
    pub fn add_insight(&mut self, insight: Insight) {
        // Check for related insights
        for existing in &self.nodes {
            let similarity = calculate_similarity(&insight.content, &existing.content);
            if similarity > 0.8 {
                self.edges.push(InsightRelation::Extends(existing.id, insight.id));
            } else if similarity < -0.8 {
                self.edges.push(InsightRelation::Contradicts(existing.id, insight.id));
            }
        }
        self.nodes.push(insight);
    }
}
```

## Phase 4: Quality Integration (Week 4)

### 4.1 Attach Quality Metrics to Threads
```rust
impl ThreadManager {
    pub fn attach_quality_metrics(&self, thread_id: &Uuid, metrics: QualityMetrics) {
        let mut threads = self.threads.lock();
        if let Some(context) = threads.get_mut(thread_id) {
            if let Some(last_turn) = context.turns.last_mut() {
                last_turn.quality_metrics = Some(metrics);
            }
        }
    }
    
    pub fn get_quality_trajectory(&self, thread_id: &Uuid) -> Vec<QualityMetrics> {
        let threads = self.threads.lock();
        if let Some(context) = threads.get(thread_id) {
            context.turns
                .iter()
                .filter_map(|turn| turn.quality_metrics.clone())
                .collect()
        } else {
            Vec::new()
        }
    }
}
```

### 4.2 Quality-Guided Refinement
```rust
impl QualityMonitor {
    pub fn guide_context_refinement(&self, thread_id: &Uuid) -> RefinementGuidance {
        let trajectory = self.thread_manager.get_quality_trajectory(thread_id);
        
        // Analyze trajectory
        let recent_quality = trajectory.iter().rev().take(3);
        let avg_circular = recent_quality.map(|m| m.circular_reasoning_score).sum::<f32>() / 3.0;
        
        if avg_circular > 0.7 {
            RefinementGuidance::BreakCircularPattern
        } else if self.is_degrading(&trajectory) {
            RefinementGuidance::RefocusOnCore
        } else {
            RefinementGuidance::ContinueDeepening
        }
    }
}
```

## Phase 5: Persistence Layer (Week 5)

### 5.1 Hybrid Storage
```rust
pub struct UnifiedPersistence {
    memory: Arc<ThreadManager>,           // Fast, temporary
    database: DatabaseConnection,         // Permanent
    cache: Arc<DashMap<Uuid, CachedContext>>, // Hot data
}

impl UnifiedPersistence {
    pub async fn save_checkpoint(&self, thread_id: &Uuid) -> Result<()> {
        let context = self.memory.get_thread(thread_id)
            .ok_or_else(|| anyhow!("Thread not found"))?;
        
        // Save to database
        let session = session::ActiveModel {
            id: Set(thread_id.clone()),
            thread_context: Set(serde_json::to_value(&context)?),
            created_at: Set(context.created_at),
            updated_at: Set(Instant::now()),
            ..Default::default()
        };
        
        session.insert(&self.database).await?;
        Ok(())
    }
    
    pub async fn restore_thread(&self, thread_id: &Uuid) -> Result<ThreadContext> {
        // Check memory first
        if let Some(context) = self.memory.get_thread(thread_id) {
            return Ok(context);
        }
        
        // Check cache
        if let Some(cached) = self.cache.get(thread_id) {
            return Ok(cached.context.clone());
        }
        
        // Load from database
        let session = session::Entity::find_by_id(thread_id)
            .one(&self.database)
            .await?
            .ok_or_else(|| anyhow!("Thread not found in database"))?;
        
        let context: ThreadContext = serde_json::from_value(session.thread_context)?;
        
        // Restore to memory
        self.memory.threads.lock().insert(*thread_id, context.clone());
        
        Ok(context)
    }
}
```

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_thread_creation() {
        let manager = ThreadManager::new();
        let id = manager.create_thread("test_tool");
        assert!(manager.get_thread(&id).is_some());
    }
    
    #[test]
    fn test_context_reconstruction() {
        let manager = ThreadManager::new();
        let id = manager.create_thread("test");
        
        let turn = ConversationTurn {
            role: Role::User,
            content: "Test message".to_string(),
            tool_used: None,
            synthesis_snapshot: None,
            quality_metrics: None,
            timestamp: Instant::now(),
        };
        
        manager.add_turn(&id, turn);
        let context = manager.reconstruct_context(&id);
        assert!(context.is_some());
        assert!(context.unwrap().contains("Test message"));
    }
    
    #[test]
    fn test_thread_expiration() {
        let mut manager = ThreadManager::new();
        manager.ttl = Duration::from_millis(100); // Short TTL for testing
        
        let id = manager.create_thread("test");
        std::thread::sleep(Duration::from_millis(200));
        
        let expired = manager.cleanup_expired();
        assert_eq!(expired, 1);
        assert!(manager.get_thread(&id).is_none());
    }
}
```

### Integration Tests
```bash
#!/bin/bash
# test_threading.sh

# Test 1: Create thread with first tool
RESPONSE1=$(echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"traced_reasoning","arguments":{"query":"What is 2+2?"}},"id":1}' | nc localhost 3333)
THREAD_ID=$(echo $RESPONSE1 | jq -r '.result.continuation_id')

# Test 2: Continue with different tool
RESPONSE2=$(echo "{\"jsonrpc\":\"2.0\",\"method\":\"tools/call\",\"params\":{\"name\":\"confer\",\"arguments\":{\"message\":\"Explain your reasoning\",\"continuation_id\":\"$THREAD_ID\"}},\"id\":2}" | nc localhost 3333)

# Verify context was preserved
echo $RESPONSE2 | grep -q "2+2" && echo "✓ Context preserved" || echo "✗ Context lost"
```

## Rollout Plan

### Week 1: Foundation
- [ ] Implement ThreadManager
- [ ] Add continuation_id to responses
- [ ] Basic context reconstruction

### Week 2: Tool Integration
- [ ] Update all tools to accept context
- [ ] Implement context enhancement
- [ ] Add turn storage

### Week 3: Synthesis Bridge
- [ ] Link synthesis states to threads
- [ ] Implement insight graph
- [ ] Add synthesis history tracking

### Week 4: Quality Integration
- [ ] Attach quality metrics to threads
- [ ] Implement quality-guided refinement
- [ ] Add trajectory analysis

### Week 5: Persistence & Testing
- [ ] Implement hybrid storage
- [ ] Add database schema migrations
- [ ] Complete test suite
- [ ] Performance benchmarking

## Success Metrics

### Functional
- [ ] Context preserved across tool calls
- [ ] Synthesis evolution tracked
- [ ] Quality metrics guide refinement

### Performance
- [ ] Thread operations < 10ms
- [ ] Context reconstruction < 50ms
- [ ] Memory usage < 100MB for 100 threads

### Quality
- [ ] 100% test coverage for threading
- [ ] No context loss in 1000 operations
- [ ] Graceful handling of expired threads

## Risk Mitigation

### Risk 1: Memory Growth
**Mitigation**: Aggressive TTL enforcement, periodic cleanup task

### Risk 2: Thread Collision
**Mitigation**: UUID v4 guarantees uniqueness, mutex protection

### Risk 3: Performance Impact
**Mitigation**: Lazy loading, caching, async operations

### Risk 4: Breaking Changes
**Mitigation**: Feature flag for threading, gradual rollout