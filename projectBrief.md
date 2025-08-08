# Lux MCP - Metacognitive AI Reasoning Server

## Project Purpose
Lux MCP is a Model Context Protocol (MCP) server built in Rust that implements metacognitive monitoring for AI reasoning systems. It detects and prevents common reasoning failures like overthinking spirals, circular reasoning, and distractor fixation by "illuminating" thought processes with real-time quality monitoring.

## Key Features
- **Traced Reasoning**: Step-by-step reasoning with metacognitive monitoring and quality degradation detection
- **Biased Reasoning**: Dual-model reasoning with visible bias detection and correction
- **Interactive Planner**: LLM-powered sequential planning with branching and revision capabilities
- **Conversational AI**: Simple chat interface with model selection and aliasing
- **Metacognitive Monitoring**: Real-time detection of circular reasoning, distractor fixation, and quality issues
- **Multi-Provider Support**: Unified interface for OpenAI and OpenRouter APIs with automatic routing

## Target Audience
- AI researchers studying metacognitive reasoning
- Developers building AI-powered applications
- Claude Code users seeking enhanced reasoning capabilities
- MCP ecosystem contributors and integrators

## Core Technologies
- **Language**: Rust (async/await, tokio runtime)
- **Protocol**: Model Context Protocol (MCP) 1.0 over stdio transport
- **APIs**: OpenAI (including o3/o4 reasoning models), OpenRouter
- **Architecture**: Modular tool-based system with pluggable LLM backends
- **Monitoring**: Semantic similarity analysis, relevance tracking, quality metrics

## Key Components
- **Tools**: `confer`, `traced_reasoning`, `biased_reasoning`, `planner`, `illumination_status`
- **LLM Integration**: Unified client with model aliasing (gpt4.1 → gpt-4-turbo-preview)
- **Monitoring System**: Circular reasoning detection, distractor fixation prevention
- **Database**: Optional synthesis state persistence with SeaORM and SQLite

## Architecture Highlights
- **Modular Design**: Clean separation between tools, LLM clients, and monitoring
- **Error Resilience**: Comprehensive error handling and graceful degradation
- **Performance**: Async processing with configurable timeouts and token limits
- **Extensibility**: Plugin architecture for adding new tools and monitoring algorithms

## Current Status
Production-ready MCP server with comprehensive test suite, documentation, and Claude Code integration. Actively maintained with regular updates for new OpenAI reasoning models and MCP protocol enhancements.