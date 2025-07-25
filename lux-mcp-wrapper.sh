#!/bin/bash
# MCP wrapper to ensure proper buffering behavior

# Set unbuffered I/O
export RUST_LOG_STYLE=never

# Run with explicit unbuffered stdio
exec stdbuf -i0 -o0 -e0 /Users/alan/Projects/_MCP/nirvana/lux-mcp/target/release/lux-mcp "$@"