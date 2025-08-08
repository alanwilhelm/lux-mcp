# Lux MCP Database Architecture

## Overview

Transitioning Lux MCP to a database-heavy approach provides several key benefits:

1. **Persistent Storage**: All reasoning sessions, synthesis evolution, and monitoring events are permanently stored
2. **Rich Querying**: SQL enables complex analytics on reasoning patterns, bias detection rates, and model performance
3. **Real-time Visualization**: Web dashboards can show live reasoning progress and synthesis evolution
4. **Multi-session Support**: Handle concurrent sessions across multiple users/clients
5. **Audit Trail**: Complete history of all reasoning steps and decisions
6. **Performance Analytics**: Track model response times, token usage, and costs

## Architecture Components

### 1. Database Layer (PostgreSQL)
- Primary data store for all session data
- JSONB columns for flexible metadata storage
- Time-series data for monitoring and analytics
- Full-text search capabilities for reasoning content

### 2. Rust Database Client
```toml
# Cargo.toml additions
[dependencies]
sqlx = { version = "0.7", features = ["runtime-tokio-native-tls", "postgres", "uuid", "json", "chrono"] }
tokio-postgres = "0.7"
deadpool-postgres = "0.11"
```

### 3. Data Flow

```
MCP Client Request
    ↓
Lux MCP Server
    ↓
Create/Update Session in DB
    ↓
LLM API Call
    ↓
Parse Response & Extract Synthesis
    ↓
Store Step + Synthesis State in DB
    ↓
Return Formatted Response to Client
    ↓
(Optional) Web Dashboard Real-time Update
```

## Key Design Decisions

### 1. Session Management
- Each tool invocation creates or continues a session
- Sessions have unique external IDs for MCP continuity
- All state is persisted immediately (no in-memory only data)

### 2. Synthesis Tracking
- Every `update_synthesis()` call is logged with raw text
- Parsed data stored in JSONB for flexible querying
- Version history maintained for evolution tracking
- Separate tables for insights and action items

### 3. Real-time Updates
- PostgreSQL LISTEN/NOTIFY for live updates
- Web dashboard can show reasoning in progress
- GraphQL subscriptions for efficient client updates

### 4. Performance Optimization
- Connection pooling with deadpool-postgres
- Batch inserts for related data (insights, actions)
- Materialized views for common analytics queries
- Partitioning for large-scale deployments

## Implementation Phases

### Phase 1: Core Database Integration
1. Set up database connections and pooling
2. Implement session creation and management
3. Store reasoning steps with metadata
4. Parse and store synthesis updates

### Phase 2: Enhanced Tracking
1. Implement bias detection storage
2. Add metacognitive monitoring events
3. Track token usage and costs
4. Add performance metrics

### Phase 3: Analytics & Visualization
1. Create analytics views and functions
2. Build REST/GraphQL API for data access
3. Develop web dashboard for visualization
4. Add export capabilities

### Phase 4: Advanced Features
1. Session comparison and diff tools
2. Pattern detection across sessions
3. Model performance analytics
4. Cost optimization recommendations

## Database Schema Highlights

### Sessions Table
- Tracks all tool invocations
- Links to specific tool types
- Maintains status and timing

### Reasoning Steps
- Stores full LLM responses
- Tracks confidence and clarity
- Links to synthesis states

### Synthesis States
- Version-controlled understanding
- Parsed structured data
- Evolution tracking

### Analytics Views
- Session summaries
- Bias detection rates
- Model performance metrics
- Cost analysis

## Migration Strategy

1. **Dual Mode Operation**
   - Keep in-memory for backward compatibility
   - Add database logging in parallel
   - Gradual transition to DB-first

2. **Data Migration**
   - Export existing sessions to DB
   - Validate data integrity
   - Switch to DB as source of truth

3. **Client Updates**
   - MCP responses remain unchanged
   - Add session URLs for web viewing
   - Optional real-time subscriptions

## Benefits for Users

1. **Better Debugging**: See exactly what models are doing
2. **Learning from History**: Analyze successful reasoning patterns
3. **Cost Tracking**: Monitor token usage and optimize
4. **Collaboration**: Share reasoning sessions with team
5. **Compliance**: Audit trail for decision-making

## Next Steps

1. Set up PostgreSQL with migrations
2. Implement database connection layer
3. Add logging to biased_reasoning first
4. Create simple web viewer
5. Gradually extend to other tools