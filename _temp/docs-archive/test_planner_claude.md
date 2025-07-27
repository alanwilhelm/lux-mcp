# Testing the Interactive Planner Tool in Claude Desktop

## How the Planner Works
The planner tool uses an LLM (default: o3-pro via `LUX_DEFAULT_REASONING_MODEL`) to generate planning steps:
- **Step 1**: You provide the initial task description
- **Steps 2+**: The LLM generates content based on previous steps and your guidance
- The `step` parameter serves as guidance/context for what the LLM should focus on

## Setup
1. Make sure lux-mcp is configured in Claude Desktop
2. Restart Claude Desktop after building with `cargo build --release`

## Test Script - Copy and paste this into Claude:

### Test 1: Basic Planning Flow
```
Use the planner tool to create a plan for "Building a scalable e-commerce platform with microservices architecture". Start with step 1, use 7 total steps, and demonstrate revisions and branches.

Step 1: Initialize the planning process
Step 2: Analyze technical requirements  
Step 3: Design service boundaries
Step 4: Create a branch to explore event-driven architecture
Step 5: Continue main path with API gateway design
Step 6: Revise step 2 with new insights
Step 7: Complete with deployment strategy
```

### Test 2: Complex Planning with Deep Thinking
```
Use the planner tool for "Designing a distributed machine learning pipeline with real-time inference capabilities". This should trigger deep thinking pauses since it's complex (8+ steps).

Make sure to:
- Start with problem analysis (step 1)
- Continue through architecture design
- Branch to explore different ML frameworks
- Revise earlier decisions based on new insights
- Complete the plan with monitoring and scaling strategies
```

### Test 3: Simple Planning (No Deep Thinking)
```
Use the planner tool for a simple task: "Create a landing page for a startup". Use only 3 steps total to avoid deep thinking pauses.
```

## Expected Behaviors:

1. **State Persistence**: The planner should remember all previous steps within a session
2. **Deep Thinking Pauses**: For plans with 5+ steps, the first 3 steps should trigger mandatory reflection pauses
3. **Branching**: Branches should be tracked separately and shown in the final summary
4. **Revisions**: Revised steps should replace the original in the history
5. **Completion**: Final step should generate a comprehensive summary of the planning journey

## Direct Tool Call Example:

```
/lux:planner {
  "step": "Design a fault-tolerant distributed task queue system",
  "step_number": 1,
  "total_steps": 5,
  "next_step_required": true,
  "temperature": 0.7
}
```

Then continue with:
```
/lux:planner {
  "step": "Choose between Redis-based queue vs Kafka vs RabbitMQ for message broker",
  "step_number": 2,
  "total_steps": 5,
  "next_step_required": true
}
```

## Verification Points:

- [ ] Tool accepts planning requests
- [ ] State persists between calls
- [ ] Deep thinking pauses work for complex plans (5+ steps)
- [ ] Branching creates separate tracked paths
- [ ] Revisions update the original steps
- [ ] Final summary includes all steps, branches, and revisions
- [ ] Monitoring detects circular reasoning if present