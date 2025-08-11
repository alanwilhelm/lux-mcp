# Sequential Thinking Examples

## Table of Contents
1. [Basic Examples](#basic-examples)
2. [Advanced Workflows](#advanced-workflows)
3. [Real-World Scenarios](#real-world-scenarios)
4. [Integration Patterns](#integration-patterns)
5. [Testing Examples](#testing-examples)

## Basic Examples

### Example 1: Simple Linear Thinking

**Scenario**: Breaking down a feature implementation into steps.

```json
// Step 1: Define the problem
{
  "tool": "sequential_thinking",
  "arguments": {
    "thought": "Implement user authentication with JWT tokens",
    "thought_number": 1,
    "total_thoughts": 4,
    "next_thought_needed": true
  }
}

// Step 2: Design phase
{
  "tool": "sequential_thinking",
  "arguments": {
    "thought": "Design the JWT token structure and claims",
    "thought_number": 2,
    "total_thoughts": 4,
    "next_thought_needed": true,
    "session_id": "auth-impl-001"
  }
}

// Step 3: Implementation
{
  "tool": "sequential_thinking",
  "arguments": {
    "thought": "Create token generation and validation functions",
    "thought_number": 3,
    "total_thoughts": 4,
    "next_thought_needed": true,
    "session_id": "auth-impl-001"
  }
}

// Step 4: Testing
{
  "tool": "sequential_thinking",
  "arguments": {
    "thought": "Write unit tests for token lifecycle",
    "thought_number": 4,
    "total_thoughts": 4,
    "next_thought_needed": false,
    "session_id": "auth-impl-001"
  }
}
```

### Example 2: AI-Assisted Problem Solving

**Scenario**: Using AI to explore a technical problem.

```json
// Step 1: Present the problem to AI
{
  "tool": "sequential_thinking_external",
  "arguments": {
    "thought": "How do I optimize database queries in a high-traffic application?",
    "thought_number": 1,
    "total_thoughts": 5,
    "next_thought_needed": true,
    "model": "gpt-4o",
    "temperature": 0.7
  }
}
// AI Response: "First, let's analyze common query patterns..."

// Step 2: Focus on specific area
{
  "tool": "sequential_thinking_external",
  "arguments": {
    "thought": "Focus on indexing strategies for PostgreSQL",
    "thought_number": 2,
    "total_thoughts": 5,
    "next_thought_needed": true,
    "session_id": "db-optimization"
  }
}
// AI Response: "For PostgreSQL, consider these indexing approaches..."

// Step 3: Request concrete examples
{
  "tool": "sequential_thinking_external",
  "arguments": {
    "thought": "Show me specific index examples for a user table with email lookups",
    "thought_number": 3,
    "total_thoughts": 5,
    "next_thought_needed": true,
    "session_id": "db-optimization"
  }
}
// AI Response: "CREATE INDEX idx_users_email ON users(email)..."
```

## Advanced Workflows

### Example 3: Branching and Exploration

**Scenario**: Exploring multiple solution paths for a caching strategy.

```json
// Main path: Initial analysis
{
  "tool": "sequential_thinking",
  "arguments": {
    "thought": "Evaluate caching needs for our API",
    "thought_number": 1,
    "total_thoughts": 5,
    "next_thought_needed": true,
    "session_id": "cache-design"
  }
}

// Main path: Identify patterns
{
  "tool": "sequential_thinking",
  "arguments": {
    "thought": "Identify data access patterns and hot paths",
    "thought_number": 2,
    "total_thoughts": 5,
    "next_thought_needed": true,
    "session_id": "cache-design"
  }
}

// Branch 1: Explore Redis
{
  "tool": "sequential_thinking",
  "arguments": {
    "thought": "Evaluate Redis as distributed cache",
    "thought_number": 3,
    "total_thoughts": 7,  // Adjusted estimate
    "next_thought_needed": true,
    "branch_from_thought": 2,
    "branch_id": "redis-branch",
    "session_id": "cache-design"
  }
}

// Branch 1 continued
{
  "tool": "sequential_thinking",
  "arguments": {
    "thought": "Redis Cluster setup and failover strategy",
    "thought_number": 4,
    "total_thoughts": 7,
    "next_thought_needed": true,
    "branch_id": "redis-branch",
    "session_id": "cache-design"
  }
}

// Branch 2: Explore CDN
{
  "tool": "sequential_thinking",
  "arguments": {
    "thought": "Consider CDN for static content caching",
    "thought_number": 5,
    "total_thoughts": 8,  // Further adjusted
    "next_thought_needed": true,
    "branch_from_thought": 2,
    "branch_id": "cdn-branch",
    "session_id": "cache-design"
  }
}

// Main path: Synthesize findings
{
  "tool": "sequential_thinking",
  "arguments": {
    "thought": "Combine Redis for dynamic data and CDN for static assets",
    "thought_number": 6,
    "total_thoughts": 8,
    "next_thought_needed": true,
    "session_id": "cache-design"
  }
}
```

### Example 4: Revision Workflow

**Scenario**: Correcting course based on new information.

```json
// Initial thought
{
  "tool": "sequential_thinking_external",
  "arguments": {
    "thought": "Design a REST API for user management",
    "thought_number": 1,
    "total_thoughts": 5,
    "next_thought_needed": true,
    "model": "gpt-4o"
  }
}

// Continue with REST design
{
  "tool": "sequential_thinking_external",
  "arguments": {
    "thought": "Define REST endpoints: GET /users, POST /users, etc.",
    "thought_number": 2,
    "total_thoughts": 5,
    "next_thought_needed": true,
    "session_id": "api-design"
  }
}

// Revision based on new requirements
{
  "tool": "sequential_thinking_external",
  "arguments": {
    "thought": "Actually, let's use GraphQL instead for better flexibility",
    "thought_number": 3,
    "total_thoughts": 6,  // Adjusted for new direction
    "next_thought_needed": true,
    "is_revision": true,
    "revises_thought": 2,
    "session_id": "api-design"
  }
}

// Continue with revised approach
{
  "tool": "sequential_thinking_external",
  "arguments": {
    "thought": "Define GraphQL schema for User type and queries",
    "thought_number": 4,
    "total_thoughts": 6,
    "next_thought_needed": true,
    "session_id": "api-design"
  }
}
```

## Real-World Scenarios

### Example 5: Code Review Process

**Scenario**: Systematic code review with mixed manual and AI assistance.

```python
# Python script to conduct code review
import json
import requests

def conduct_code_review(file_content, session_id):
    thoughts = []
    
    # Step 1: Manual security check (sensitive data)
    response = call_tool("sequential_thinking", {
        "thought": "Security review: No hardcoded credentials or API keys found",
        "thought_number": 1,
        "total_thoughts": 5,
        "next_thought_needed": True,
        "session_id": session_id
    })
    thoughts.append(response)
    
    # Step 2: AI code quality analysis
    response = call_tool("sequential_thinking_external", {
        "thought": f"Analyze code quality:\n{file_content}",
        "thought_number": 2,
        "total_thoughts": 5,
        "next_thought_needed": True,
        "session_id": session_id,
        "model": "gpt-4o",
        "temperature": 0.3  # Lower temperature for consistency
    })
    thoughts.append(response)
    
    # Step 3: Manual performance review
    response = call_tool("sequential_thinking", {
        "thought": "Performance: O(n²) algorithm in sorting function needs optimization",
        "thought_number": 3,
        "total_thoughts": 5,
        "next_thought_needed": True,
        "session_id": session_id
    })
    thoughts.append(response)
    
    # Step 4: AI suggestions for improvement
    response = call_tool("sequential_thinking_external", {
        "thought": "Suggest optimizations for the O(n²) sorting algorithm",
        "thought_number": 4,
        "total_thoughts": 5,
        "next_thought_needed": True,
        "session_id": session_id
    })
    thoughts.append(response)
    
    # Step 5: Summary
    response = call_tool("sequential_thinking", {
        "thought": "Review complete: 1 security pass, 2 quality issues, 1 performance issue",
        "thought_number": 5,
        "total_thoughts": 5,
        "next_thought_needed": False,
        "session_id": session_id
    })
    thoughts.append(response)
    
    return thoughts
```

### Example 6: Architecture Decision Record (ADR)

**Scenario**: Creating an ADR with structured reasoning.

```javascript
// JavaScript function to create ADR
async function createADR(topic) {
    const session_id = `adr-${Date.now()}`;
    const steps = [];
    
    // Context
    const context = await callTool("sequential_thinking_external", {
        thought: `What is the context for: ${topic}?`,
        thought_number: 1,
        total_thoughts: 6,
        next_thought_needed: true,
        model: "gpt-4o"
    });
    steps.push(context);
    
    // Problem statement
    const problem = await callTool("sequential_thinking_external", {
        thought: "Define the specific problem we're solving",
        thought_number: 2,
        total_thoughts: 6,
        next_thought_needed: true,
        session_id: session_id
    });
    steps.push(problem);
    
    // Option 1
    const option1 = await callTool("sequential_thinking", {
        thought: "Option 1: Microservices architecture with Kubernetes",
        thought_number: 3,
        total_thoughts: 6,
        next_thought_needed: true,
        branch_from_thought: 2,
        branch_id: "microservices",
        session_id: session_id
    });
    steps.push(option1);
    
    // Option 2
    const option2 = await callTool("sequential_thinking", {
        thought: "Option 2: Monolithic architecture with modular design",
        thought_number: 4,
        total_thoughts: 6,
        next_thought_needed: true,
        branch_from_thought: 2,
        branch_id: "monolith",
        session_id: session_id
    });
    steps.push(option2);
    
    // Decision
    const decision = await callTool("sequential_thinking_external", {
        thought: "Compare options and make recommendation based on team size and complexity",
        thought_number: 5,
        total_thoughts: 6,
        next_thought_needed: true,
        session_id: session_id,
        model: "o3-mini"  // Use reasoning model for decision
    });
    steps.push(decision);
    
    // Consequences
    const consequences = await callTool("sequential_thinking_external", {
        thought: "What are the long-term consequences of this decision?",
        thought_number: 6,
        total_thoughts: 6,
        next_thought_needed: false,
        session_id: session_id
    });
    steps.push(consequences);
    
    return formatADR(steps);
}
```

## Integration Patterns

### Example 7: Combining with Other Lux Tools

**Scenario**: Use planner for high-level, then sequential thinking for details.

```json
// First: Use planner for overall structure
{
  "tool": "planner",
  "arguments": {
    "step": "Build a real-time chat application",
    "step_number": 1,
    "total_steps": 5,
    "next_step_required": true
  }
}
// Planner output: "1. Set up WebSocket infrastructure..."

// Then: Use sequential thinking for detailed implementation
{
  "tool": "sequential_thinking_external",
  "arguments": {
    "thought": "Implement WebSocket connection handling with reconnection logic",
    "thought_number": 1,
    "total_thoughts": 4,
    "next_thought_needed": true,
    "model": "gpt-4o"
  }
}

// Continue with detailed steps...
{
  "tool": "sequential_thinking_external",
  "arguments": {
    "thought": "Add message queue for offline message delivery",
    "thought_number": 2,
    "total_thoughts": 4,
    "next_thought_needed": true,
    "session_id": "websocket-impl"
  }
}
```

### Example 8: Hybrid Human-AI Collaboration

**Scenario**: Human provides structure, AI fills in details.

```python
def hybrid_problem_solving(problem_statement):
    session_id = f"hybrid-{uuid.uuid4()}"
    
    # Human defines structure
    structure = [
        "Understand the problem",
        "Identify constraints",
        "Generate solutions",
        "Evaluate trade-offs",
        "Make recommendation"
    ]
    
    results = []
    
    for i, step_guide in enumerate(structure, 1):
        if i == 1:
            # First step: AI analyzes the problem
            thought = problem_statement
        else:
            # Subsequent steps: Use human guidance
            thought = f"Now {step_guide} based on previous analysis"
        
        response = call_tool("sequential_thinking_external", {
            "thought": thought,
            "thought_number": i,
            "total_thoughts": len(structure),
            "next_thought_needed": i < len(structure),
            "session_id": session_id,
            "model": "gpt-4o" if i % 2 == 0 else "gpt-4o-mini",  # Alternate models
            "temperature": 0.5 if "evaluate" in step_guide.lower() else 0.7
        })
        
        results.append({
            "step": step_guide,
            "analysis": response["thought_content"],
            "confidence": response.get("confidence", 0)
        })
    
    return results
```

## Testing Examples

### Example 9: Unit Test for Sequential Thinking

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sequential_thinking_basic() {
        let tool = SequentialThinkingTool::new();
        
        let request = SequentialThinkingRequest {
            thought: "Test thought".to_string(),
            thought_number: 1,
            total_thoughts: 3,
            next_thought_needed: true,
            is_revision: false,
            revises_thought: None,
            branch_from_thought: None,
            branch_id: None,
            needs_more_thoughts: false,
            session_id: Some("test-session".to_string()),
        };
        
        let response = tool.process_thought(request).unwrap();
        
        assert_eq!(response.thought_number, 1);
        assert_eq!(response.total_thoughts, 3);
        assert_eq!(response.next_thought_needed, true);
        assert_eq!(response.status, "recorded");
        assert_eq!(response.thought_history_length, 1);
    }
    
    #[test]
    fn test_revision_tracking() {
        let tool = SequentialThinkingTool::new();
        let session_id = "revision-test".to_string();
        
        // First thought
        let request1 = SequentialThinkingRequest {
            thought: "Original thought".to_string(),
            thought_number: 1,
            total_thoughts: 2,
            next_thought_needed: true,
            session_id: Some(session_id.clone()),
            ..Default::default()
        };
        tool.process_thought(request1).unwrap();
        
        // Revision
        let request2 = SequentialThinkingRequest {
            thought: "Revised thought".to_string(),
            thought_number: 2,
            total_thoughts: 2,
            next_thought_needed: false,
            is_revision: true,
            revises_thought: Some(1),
            session_id: Some(session_id.clone()),
            ..Default::default()
        };
        let response = tool.process_thought(request2).unwrap();
        
        assert_eq!(response.status, "revision");
        assert_eq!(response.thought_history_length, 2);
    }
}
```

### Example 10: Integration Test Script

```bash
#!/bin/bash
# test_sequential_thinking.sh

echo "Testing Sequential Thinking Tools"
echo "================================="

SERVER_URL="http://localhost:8080"
SESSION_ID="test-$(date +%s)"

# Test 1: Simple sequential thinking
echo "Test 1: Basic sequential thinking"
curl -X POST $SERVER_URL/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/call",
    "params": {
      "name": "sequential_thinking",
      "arguments": {
        "thought": "Test thought 1",
        "thought_number": 1,
        "total_thoughts": 3,
        "next_thought_needed": true,
        "session_id": "'$SESSION_ID'"
      }
    }
  }' | jq '.result.content[0].text'

# Test 2: AI-powered thinking
echo -e "\nTest 2: AI-powered sequential thinking"
curl -X POST $SERVER_URL/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 2,
    "method": "tools/call",
    "params": {
      "name": "sequential_thinking_external",
      "arguments": {
        "thought": "Explain Docker containers",
        "thought_number": 1,
        "total_thoughts": 3,
        "next_thought_needed": true,
        "model": "gpt-4o-mini",
        "temperature": 0.5
      }
    }
  }' | jq '.result.content[0].text'

# Test 3: Branching
echo -e "\nTest 3: Branch creation"
curl -X POST $SERVER_URL/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 3,
    "method": "tools/call",
    "params": {
      "name": "sequential_thinking",
      "arguments": {
        "thought": "Alternative approach",
        "thought_number": 2,
        "total_thoughts": 4,
        "next_thought_needed": true,
        "branch_from_thought": 1,
        "branch_id": "alternative",
        "session_id": "'$SESSION_ID'"
      }
    }
  }' | jq '.result'

echo -e "\nTests completed!"
```

### Example 11: Performance Testing

```python
import time
import concurrent.futures
import statistics

def performance_test():
    """Test performance characteristics of sequential thinking tools"""
    
    def single_request(i):
        start = time.time()
        response = call_tool("sequential_thinking", {
            "thought": f"Performance test thought {i}",
            "thought_number": i,
            "total_thoughts": 100,
            "next_thought_needed": True,
            "session_id": f"perf-test-{i % 10}"  # Use 10 different sessions
        })
        return time.time() - start
    
    # Test simple tool (no LLM)
    print("Testing simple sequential thinking...")
    with concurrent.futures.ThreadPoolExecutor(max_workers=10) as executor:
        simple_times = list(executor.map(single_request, range(100)))
    
    print(f"Simple tool stats:")
    print(f"  Mean: {statistics.mean(simple_times):.3f}s")
    print(f"  Median: {statistics.median(simple_times):.3f}s")
    print(f"  Stdev: {statistics.stdev(simple_times):.3f}s")
    
    # Test AI-powered tool
    def ai_request(i):
        start = time.time()
        response = call_tool("sequential_thinking_external", {
            "thought": "Generate a thought about testing",
            "thought_number": 1,
            "total_thoughts": 3,
            "next_thought_needed": True,
            "model": "gpt-4o-mini",
            "temperature": 0.5,
            "session_id": f"perf-ai-{i}"
        })
        return time.time() - start
    
    print("\nTesting AI-powered sequential thinking...")
    ai_times = [ai_request(i) for i in range(10)]  # Fewer due to rate limits
    
    print(f"AI tool stats:")
    print(f"  Mean: {statistics.mean(ai_times):.3f}s")
    print(f"  Median: {statistics.median(ai_times):.3f}s")
    print(f"  Stdev: {statistics.stdev(ai_times):.3f}s")

if __name__ == "__main__":
    performance_test()
```

## Best Practices from Examples

### DO:
1. **Use consistent session IDs** - Maintains context across thoughts
2. **Adjust total_thoughts dynamically** - Estimates can change
3. **Branch for exploration** - Don't be afraid to explore alternatives
4. **Mix manual and AI** - Use the right tool for each step
5. **Set appropriate temperatures** - Lower for analysis, higher for creativity

### DON'T:
1. **Forget session IDs** - You'll lose context
2. **Skip important thoughts** - Maintain logical flow
3. **Overuse AI for simple tasks** - Sometimes manual is better
4. **Ignore confidence scores** - They indicate AI uncertainty
5. **Create too many branches** - Can become hard to manage

## Next Steps

- Explore the [API Reference](./sequential-thinking-api.md) for detailed specifications
- Review the [Comparison Guide](./sequential-thinking-comparison.md) for tool selection
- Check the main [Sequential Thinking Documentation](./sequential-thinking.md) for architecture details