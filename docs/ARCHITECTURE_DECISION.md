# Architecture Clarification: In-Memory Only

## Status

Lux MCP is and will remain a **stateless, in-memory MCP server**.

## Current Architecture

- ✅ In-memory session management (3-hour TTL)
- ✅ No database dependencies
- ✅ Fast response times
- ✅ Simple deployment
- ✅ Stateless operation

## Why No Database?

1. **Simplicity**: MCP servers should be lightweight and easy to deploy
2. **Performance**: In-memory operations are orders of magnitude faster
3. **Stateless Design**: Aligns with MCP protocol expectations
4. **No Persistence Needed**: Sessions are ephemeral by design
5. **Reduced Complexity**: No migrations, connection pools, or DB management

## Design Decision

Database persistence was considered early in the project but rejected to maintain:
- Simplicity of deployment
- Maximum performance
- Alignment with MCP protocol design
- Zero external dependencies

## Architecture Principles

1. **Simplicity First**: Keep the server lightweight and easy to deploy
2. **Performance**: In-memory operations for maximum speed
3. **Stateless**: Align with MCP protocol expectations
4. **Zero Dependencies**: No external services required

## Session Management

Lux MCP uses simple in-memory session management:

```rust
pub struct ThreadManager {
    threads: Arc<Mutex<HashMap<Uuid, ThreadContext>>>,
    ttl: Duration,  // 3 hours
}
```

- Sessions expire after 3 hours of inactivity
- Automatic cleanup runs periodically
- No persistence across server restarts
- This is intentional and by design

## Actual File Structure

```
lux-mcp/
├── src/
│   ├── server/          # MCP server implementation
│   ├── tools/           # Tool implementations
│   ├── llm/             # LLM client abstraction
│   ├── monitoring/      # Metacognitive monitoring
│   └── models/          # Data models
└── Cargo.toml          # Simple dependencies, no database
```

## FAQ

### Q: Why no database?
A: MCP servers are meant to be lightweight tools that integrate with AI assistants. Adding database dependencies would complicate deployment and reduce performance.

### Q: What if I need persistence?
A: The MCP protocol is designed for stateless interactions. If you need persistence, consider implementing it at the application level, not in the MCP server.

### Q: Was database support ever considered?
A: Yes, but it was rejected to keep the server lightweight and aligned with MCP design principles.

## Conclusion

Lux MCP is and will remain a stateless, in-memory MCP server. This design choice ensures:

- Maximum performance
- Simple deployment
- No external dependencies
- Clear separation of concerns
- Alignment with MCP protocol design

## Summary

The in-memory architecture is a deliberate design choice that prioritizes:
- Developer experience (easy setup)
- Performance (no I/O overhead)
- Reliability (no external dependencies)
- Maintainability (less complexity)

This approach has proven effective for MCP server implementations and aligns with the protocol's stateless design philosophy.