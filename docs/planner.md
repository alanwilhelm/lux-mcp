# Planner Tool Documentation

## Overview

The `planner` tool is an AI-powered sequential planning system that creates actionable, implementation-ready plans by directly examining project files and generating concrete steps. Unlike simple task lists, it produces mandatory actions grounded in your actual codebase, complete with specific file paths, function names, and implementation details.

## Key Features

- **Direct File Access**: Automatically discovers and reads project files
- **Mandatory Actions**: Returns MUST-DO actions marked with ⚠️
- **Implementation-Ready**: Includes specific code locations and commands
- **Auto-Discovery**: Finds relevant files based on planning context
- **Session-Based**: Maintains planning history across steps
- **Branching Support**: Explore alternative approaches
- **Deep Thinking**: Pauses for reflection on complex plans (≥5 steps)
- **File Caching**: Efficient reuse of file contents across steps

## How It Works

1. **Step 1**: You describe the task/problem to plan
2. **Auto-Discovery**: Tool finds and reads relevant project files
3. **Context Building**: Analyzes actual code structure and dependencies
4. **Step Generation**: AI creates concrete, actionable planning steps
5. **Mandatory Actions**: Returns specific actions you MUST take
6. **Iteration**: Continue with guided steps until plan is complete

## When to Use

### Ideal For:
- **Architecture Planning**: System design with real constraints
- **Feature Implementation**: Multi-step feature development
- **Refactoring Projects**: Large-scale code reorganization
- **Migration Planning**: Database, framework, or API migrations
- **Integration Tasks**: Third-party service integration
- **DevOps Setup**: CI/CD, deployment, infrastructure

### Not Recommended For:
- Simple single-file changes
- Pure theoretical planning (use `traced_reasoning`)
- Quick bug fixes
- Tasks without file context needs

## API Reference

### Request Parameters

```typescript
interface PlannerRequest {
  // Required
  step: string;                       // Task (step 1) or planning content (2+)
  step_number: number;                // Current step number (starts at 1)
  total_steps: number;                // Estimated total steps
  next_step_required: boolean;        // Whether another step is needed
  
  // Optional - File Reading
  auto_discover_files?: boolean;      // Auto-find relevant files (default: true)
  file_paths?: string[];              // Specific files to examine
  include_file_contents?: boolean;    // Read file contents (default: true)
  
  // Optional - Model Configuration
  model?: string;                     // Model to use (default: LUX_MODEL_REASONING)
  temperature?: number;               // Temperature 0.0-1.0 (default: 0.7)
  use_mini?: boolean;                // Use mini model for cost savings
  
  // Optional - Branching & Revision
  is_branch_point?: boolean;          // True if branching from previous step
  branch_from_step?: number;          // Which step is the branch point
  branch_id?: string;                // Branch identifier
  is_step_revision?: boolean;         // True if revising a step
  revises_step_number?: number;       // Which step is being revised
  more_steps_needed?: boolean;        // Extend beyond initial estimate
}
```

### Response Format

```typescript
interface PlannerResponse {
  step_number: number;
  content: string;                    // The planning step content
  next_step_required: boolean;
  total_steps: number;
  session_id: string;
  
  // Critical: Mandatory actions
  mandatory_actions: string[];        // Actions marked with ⚠️ MANDATORY
  
  // File context
  files_examined: string[];           // Files read during this step
  recommended_files: string[];        // Files to examine for implementation
  
  // Metadata
  metadata: {
    branch_id?: string;
    is_revision: boolean;
    planning_depth: number;
    model_used: string;
    files_cached: number;
  };
}
```

## Usage Examples

### Basic Planning Session

```json
// Step 1: Describe the task
{
  "tool": "planner",
  "arguments": {
    "step": "Design and implement a rate limiting system for our REST API",
    "step_number": 1,
    "total_steps": 6,
    "next_step_required": true
  }
}

// Response includes:
{
  "mandatory_actions": [
    "⚠️ MANDATORY: Examine /api/middleware/auth.js to understand current middleware structure",
    "⚠️ MANDATORY: Review /package.json for existing rate limiting dependencies"
  ],
  "files_examined": [
    "/api/middleware/auth.js",
    "/api/routes/index.js",
    "/package.json"
  ],
  "content": "## Step 1: Analyze Current API Structure\n\n..."
}

// Step 2: Continue planning
{
  "tool": "planner",
  "arguments": {
    "step": "Define rate limiting strategies per endpoint",
    "step_number": 2,
    "total_steps": 6,
    "next_step_required": true
  }
}
```

### With Specific Files

```json
{
  "tool": "planner",
  "arguments": {
    "step": "Refactor the authentication system to use JWT tokens",
    "step_number": 1,
    "total_steps": 5,
    "next_step_required": true,
    "file_paths": [
      "/auth/login.js",
      "/auth/session.js",
      "/middleware/auth.js",
      "/config/auth.yaml"
    ],
    "auto_discover_files": false  // Only use specified files
  }
}
```

### Branching Example

```json
// Explore alternative approach
{
  "tool": "planner",
  "arguments": {
    "step": "Consider microservices architecture instead",
    "step_number": 4,
    "total_steps": 7,
    "next_step_required": true,
    "is_branch_point": true,
    "branch_from_step": 3,
    "branch_id": "microservices-approach"
  }
}
```

### Cost-Optimized Planning

```json
{
  "tool": "planner",
  "arguments": {
    "step": "Plan database migration from MySQL to PostgreSQL",
    "step_number": 1,
    "total_steps": 4,
    "next_step_required": true,
    "use_mini": true,              // Use cheaper model
    "auto_discover_files": false,  // Manual file selection
    "file_paths": [
      "/db/schema.sql",
      "/config/database.js"
    ]
  }
}
```

## Auto-Discovery Patterns

The planner automatically searches for files based on context:

### Project Structure Files
- `README.md` - Project overview
- `package.json` - Dependencies and scripts
- `Cargo.toml` - Rust projects
- `requirements.txt` - Python dependencies
- `Makefile` - Build configuration
- `docker-compose.yml` - Container setup

### Context-Specific Patterns

| Planning Context | File Patterns Searched |
|-----------------|------------------------|
| API/Routes | `*api*`, `*route*`, `*endpoint*` |
| Database | `*model*`, `*schema*`, `migrations/*` |
| Testing | `*test*`, `*spec*`, `test/*` |
| Security | `*auth*`, `*security*`, `*permission*` |
| Configuration | `config/*`, `*.config.*`, `.env*` |
| Documentation | `docs/*`, `*.md`, `API.md` |

### Example Auto-Discovery

```json
// Input: "Plan API versioning implementation"
// Auto-discovers:
- /api/routes/v1/users.js
- /api/routes/v1/products.js  
- /api/middleware/version.js
- /docs/API.md
- /package.json
```

## Mandatory Actions

The planner generates three types of actions:

### 1. Examination Actions
```
⚠️ MANDATORY: Examine /api/rate-limit.js lines 45-67 for current implementation
⚠️ MANDATORY: Review database schema at /db/schema.sql for user table structure
```

### 2. Implementation Actions
```
⚠️ MANDATORY: Create new file /api/middleware/rate-limiter.js with Redis connection
⚠️ MANDATORY: Add rate-limiter middleware to /api/routes/index.js after line 23
```

### 3. Verification Actions
```
⚠️ MANDATORY: Run 'npm test api/rate-limit.test.js' to verify implementation
⚠️ MANDATORY: Check Redis connection with 'redis-cli ping' before deployment
```

## Planning Patterns

### Feature Implementation

```python
def plan_feature(description, estimated_steps=5):
    session_id = None
    plan = []
    
    for i in range(1, estimated_steps + 1):
        response = call_tool("planner", {
            "step": description if i == 1 else f"Continue planning step {i}",
            "step_number": i,
            "total_steps": estimated_steps,
            "next_step_required": i < estimated_steps,
            "auto_discover_files": i == 1  # Only auto-discover on first step
        })
        
        plan.append(response)
        
        # Execute mandatory actions
        for action in response.get("mandatory_actions", []):
            print(f"Must do: {action}")
    
    return plan
```

### Migration Planning

```python
def plan_migration(source_tech, target_tech, codebase_path):
    return call_tool("planner", {
        "step": f"Migrate from {source_tech} to {target_tech}",
        "step_number": 1,
        "total_steps": 8,
        "next_step_required": True,
        "file_paths": [
            f"{codebase_path}/package.json",
            f"{codebase_path}/src/index.js",
            f"{codebase_path}/config/database.js"
        ]
    })
```

### Architecture Planning

```python
def plan_architecture(requirements, constraints):
    # Step 1: Analyze current architecture
    current = call_tool("planner", {
        "step": f"Analyze current architecture for: {requirements}",
        "step_number": 1,
        "total_steps": 6,
        "next_step_required": True,
        "auto_discover_files": True
    })
    
    # Step 2: Design new architecture
    design = call_tool("planner", {
        "step": f"Design architecture considering: {constraints}",
        "step_number": 2,
        "total_steps": 6,
        "next_step_required": True
    })
    
    return current, design
```

## Best Practices

### 1. Step Descriptions

```json
// Good: Specific and action-oriented
{
  "step": "Implement Redis caching for user sessions with 15-minute TTL"
}

// Poor: Vague or theoretical
{
  "step": "Think about caching"
}
```

### 2. File Path Management

```python
# Always use absolute paths
file_paths = [
    "/src/api/users.js",      # ✅ Absolute
    "./api/users.js",         # ❌ Relative
    "api/users.js"            # ❌ No prefix
]

# Verify files exist before planning
import os
file_paths = [f for f in file_paths if os.path.exists(f)]
```

### 3. Handling Mandatory Actions

```python
def execute_plan(plan_response):
    mandatory = plan_response.get("mandatory_actions", [])
    
    for action in mandatory:
        if "Examine" in action:
            # Extract file path and review
            file_path = extract_path(action)
            review_file(file_path)
        
        elif "Create" in action:
            # Extract file path and create
            file_path = extract_path(action)
            create_file(file_path)
        
        elif "Run" in action:
            # Extract command and execute
            command = extract_command(action)
            run_command(command)
```

### 4. Session Management

```python
class PlanningSession:
    def __init__(self, task):
        self.task = task
        self.steps = []
        self.files_cache = set()
    
    def add_step(self, description, step_num, total):
        response = call_tool("planner", {
            "step": description,
            "step_number": step_num,
            "total_steps": total,
            "next_step_required": step_num < total
        })
        
        self.steps.append(response)
        self.files_cache.update(response.get("files_examined", []))
        
        return response
```

### 5. Cost Optimization

- Use `use_mini: true` for initial exploration
- Limit `file_paths` to essential files
- Set `auto_discover_files: false` after step 1
- Cache planning sessions for similar tasks

## Performance Considerations

### Response Times

| Model | Step 1 (Discovery) | Steps 2+ | Total (6 steps) |
|-------|-------------------|----------|-----------------|
| GPT-5 | 10-20s | 5-10s | 35-70s |
| O3-Pro | 30-60s | 20-40s | 130-260s |
| GPT-4o | 5-10s | 3-7s | 20-45s |
| Mini | 3-7s | 2-5s | 13-32s |

### Token Usage

- **With Auto-Discovery**: 15K-25K tokens/step
- **With Manual Files**: 10K-20K tokens/step
- **Without Files**: 5K-10K tokens/step
- **File Reading Limit**: 15KB per file

### File Caching

The planner maintains a session-level file cache:
- Files read once per session
- Shared across planning steps
- Reduces redundant reads
- Cache size: unlimited within session

## Advanced Features

### Deep Thinking Mode

For complex plans (≥5 steps), the planner automatically:
1. Pauses for reflection between steps
2. Reviews previous steps for consistency
3. Adjusts total_steps if needed
4. Generates more detailed actions

### Branch Merging

```python
def explore_alternatives(base_task, alternatives):
    # Initial planning
    base = plan_feature(base_task, 3)
    
    # Branch for each alternative
    branches = {}
    for alt_name, alt_desc in alternatives.items():
        branch = call_tool("planner", {
            "step": f"Alternative approach: {alt_desc}",
            "step_number": 4,
            "total_steps": 5,
            "next_step_required": True,
            "is_branch_point": True,
            "branch_from_step": 3,
            "branch_id": alt_name
        })
        branches[alt_name] = branch
    
    # Synthesize best approach
    synthesis = call_tool("planner", {
        "step": "Synthesize best approach from alternatives",
        "step_number": 6,
        "total_steps": 6,
        "next_step_required": False
    })
    
    return base, branches, synthesis
```

### Integration with Other Tools

```python
def comprehensive_planning(task):
    # Step 1: Plan with planner
    plan = call_tool("planner", {
        "step": task,
        "step_number": 1,
        "total_steps": 5,
        "next_step_required": True
    })
    
    # Step 2: Analyze plan for biases
    bias_check = call_tool("biased_reasoning", {
        "query": f"Check this plan for biases: {plan['content']}"
    })
    
    # Step 3: Deep reasoning on critical steps
    critical_analysis = call_tool("traced_reasoning", {
        "thought": f"Analyze critical risks in: {plan['mandatory_actions']}",
        "thought_number": 1,
        "total_thoughts": 3,
        "next_thought_needed": True
    })
    
    return {
        "plan": plan,
        "bias_analysis": bias_check,
        "risk_assessment": critical_analysis
    }
```

## Comparison with Other Tools

| Feature | planner | traced_reasoning | sequential_thinking | biased_reasoning |
|---------|---------|------------------|--------------------|--------------------|
| File Reading | ✅ Auto-discovery | ✅ Manual | ❌ | ✅ Manual |
| Mandatory Actions | ✅ | ❌ | ❌ | ❌ |
| Implementation Focus | ✅ | ❌ | ❌ | ❌ |
| Monitoring | ❌ | ✅ Full | ❌ | Bias only |
| Branching | ✅ | ✅ | ✅ | ❌ |
| Speed | Medium | Medium | Fast | Slow |
| Best For | Implementation | Analysis | Control | Balance |

## Troubleshooting

### Common Issues

1. **"No files discovered"**
   ```json
   // Solution: Provide explicit file paths
   {
     "file_paths": ["/src/index.js", "/package.json"],
     "auto_discover_files": false
   }
   ```

2. **"Plan seems too theoretical"**
   - Ensure files are being read
   - Provide more specific step descriptions
   - Include actual code files, not just configs

3. **"Mandatory actions are vague"**
   - Check if relevant files were found
   - Be more specific in step descriptions
   - Ensure project structure is standard

4. **High latency with auto-discovery**
   - Normal for first step
   - Consider manual file selection
   - Use `use_mini: true` for speed

### Debug Mode

```bash
# Enable detailed logging
RUST_LOG=debug ./target/release/lux-mcp

# Check file discovery
RUST_LOG=lux_mcp::tools::planner=trace ./target/release/lux-mcp
```

## Real-World Examples

### API Rate Limiting Implementation

```json
{
  "step": "Implement Redis-based rate limiting for REST API with configurable limits per endpoint",
  "mandatory_actions": [
    "⚠️ MANDATORY: Install redis and express-rate-limit packages",
    "⚠️ MANDATORY: Create /api/middleware/rateLimiter.js with Redis store configuration",
    "⚠️ MANDATORY: Add rate limiter to /api/routes/users.js with 100 req/15min for GET, 10 req/15min for POST",
    "⚠️ MANDATORY: Update /config/redis.js with connection pool settings",
    "⚠️ MANDATORY: Add REDIS_URL to .env.example"
  ]
}
```

### Database Migration

```json
{
  "step": "Migrate user authentication from MongoDB to PostgreSQL",
  "mandatory_actions": [
    "⚠️ MANDATORY: Create migration script at /migrations/001_users_table.sql",
    "⚠️ MANDATORY: Update /models/User.js to use Sequelize instead of Mongoose",
    "⚠️ MANDATORY: Modify /api/auth/login.js lines 23-45 to use new User model",
    "⚠️ MANDATORY: Run 'npm install sequelize pg pg-hstore'",
    "⚠️ MANDATORY: Test with 'npm run test:auth' before deployment"
  ]
}
```

## Summary

The `planner` tool bridges the gap between high-level planning and concrete implementation by grounding plans in your actual codebase. Its ability to read files, generate mandatory actions, and maintain context makes it invaluable for complex multi-step implementations.

Key takeaways:
- Best for implementation planning with real code context
- Generates mandatory, actionable steps with specific details
- Auto-discovers relevant files or accepts manual selection
- Higher token usage due to file reading
- Produces implementation-ready plans, not theoretical frameworks

For pure analysis without implementation, use `traced_reasoning`. For simple task tracking, use `sequential_thinking`.