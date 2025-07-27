#!/bin/bash
# Clean runner for Lux MCP Server
# This ensures no environment conflicts

# Unset any problematic environment variables that might interfere
unset RUST_LOG
unset RUST_BACKTRACE

# Run the server with clean environment
exec ./target/release/lux-mcp "$@"