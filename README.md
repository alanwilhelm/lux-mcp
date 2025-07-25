# Lux MCP

*Illuminate your thinking* - A focused Model Context Protocol (MCP) server built in Rust that shines light on cognitive patterns, preventing overthinking spirals and mental darkness.

## Overview

Lux brings clarity to AI reasoning through real-time metacognitive monitoring. Built in Rust for performance and precision, it detects when thoughts drift into shadow and guides them back to light.

### Why Rust?

- **Fast**: Native performance without garbage collection overhead
- **Correct**: Memory safety and thread safety guaranteed at compile time  
- **No Runtime**: Direct system integration without interpreter or VM dependencies
- **Predictable**: Consistent performance for real-time illumination

## Single Purpose: Illuminate Your Thinking

Lux provides **one tool** that does **one thing exceptionally well**: shining light on your reasoning process to prevent mental darkness.

### The Tool: `lux_think`

```json
{
  "thought": "Current thinking step",
  "thought_number": 1,
  "total_thoughts": 10,
  "model": "claude"
}
```

### What Lux Illuminates

Drawing from cutting-edge metacognitive research, Lux detects:

- **Circular Reasoning** (>85% similarity): "You're walking in circles in the dark"
- **Distractor Fixation** (<30% relevance): "You're following a false light"  
- **Quality Degradation** (declining clarity): "Your thinking is dimming"

### How Lux Guides You

When darkness threatens, Lux provides illuminating guidance:

- **Refocus Beacon**: Shines light back to the core problem
- **Consolidation Glow**: Gather your insights before they fade
- **Breakthrough Flash**: Break free from circular shadows

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      lux_think Tool                          │
├─────────────────────────────────────────────────────────────┤
│  Input → Illuminate → Guide → Enhance → Response            │
│                                                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │   Circular    │  │  Distractor  │  │     Quality      │  │
│  │   Shadow      │  │    Drift     │  │     Dimming      │  │
│  │   Detector    │  │   Detector   │  │    Detector      │  │
│  └──────┬───────┘  └──────┬───────┘  └────────┬─────────┘  │
│         └──────────────────┴───────────────────┘            │
│                            ▼                                 │
│                  ┌──────────────────┐                       │
│                  │   Illumination    │                       │
│                  │     Engine        │                       │
│                  └──────────────────┘                       │
└─────────────────────────────────────────────────────────────┘
```

## Key Features

### Light-Guided Reasoning

Our illumination model:
- Enforces clear, step-by-step thinking
- Maintains luminous context across steps  
- Lights alternative paths when needed
- Brightens previous thoughts for revision

### Real-Time Illumination

Continuous monitoring of cognitive shadows:
- **Pattern Recognition**: Detects when thoughts circle in darkness
- **Relevance Beacon**: Monitors drift from guiding light
- **Clarity Analysis**: Tracks when insights begin to dim
- **Confidence Radiance**: Observes the brightness of certainty

### Adaptive Guidance

Strategic illumination to maintain clarity:
- **Refocus Flare**: When detecting drift into shadows
- **Consolidation Lamp**: When clarity metrics show dimming
- **Breakthrough Spotlight**: When stuck in dark loops
- **Synthesis Beacon**: When overthinking clouds judgment

## The Science of Illumination

Based on research showing that distractor fixation is a real cognitive phenomenon affecting both human and AI reasoning, Lux implements:

1. **Multi-Level Shadow Detection**: Different types of mental darkness require different lights
2. **Temporal Pattern Tracking**: Some shadows are predictable, others random
3. **Graduated Illumination**: From gentle glow to bright spotlight as needed
4. **Cognitive Load Management**: More vigilance when thinking is already strained

## Getting Started

### Prerequisites

- Rust 1.70+ (for stable async support)
- API keys for your preferred model provider

### Building

```bash
cargo build --release
```

### Running

```bash
# Direct execution
cargo run

# With MCP client
./target/release/lux-mcp
```

### Configuration

```toml
# lux.toml
[illumination]
circular_threshold = 0.85
distractor_threshold = 0.3
clarity_window = 3

[guidance]
intervention_steps = 2
max_thoughts = 15
brightness = "balanced"
```

## Example Session

```json
// First thought - clear light
{
  "tool": "lux_think",
  "arguments": {
    "thought": "I need to design a distributed cache system",
    "thought_number": 1,
    "total_thoughts": 8,
    "model": "claude"
  }
}

// Response - healthy illumination
{
  "content": "Let me identify the key requirements for this distributed cache...",
  "illumination": {
    "brightness": 0.95,
    "shadows_detected": "none",
    "path_clarity": "excellent"
  }
}

// Later - shadows creeping in
{
  "tool": "lux_think",
  "arguments": {
    "thought": "But first, let me dive deep into Redis internals...",
    "thought_number": 5,
    "total_thoughts": 8,
    "model": "claude"
  }
}

// Lux intervenes with guiding light
{
  "content": "Redis uses...",
  "illumination": {
    "brightness": 0.6,
    "shadows_detected": "distractor_fixation",
    "guidance": "🔦 REFOCUS BEACON: You're following a false light into implementation details. Return to the illuminated path of high-level cache design."
  }
}
```

## Research Foundation

Lux builds upon:
- Visual attention and distractor fixation research
- "Inverse Scaling in Test-Time Compute" findings
- Metacognitive monitoring from cognitive science
- The power of light as a metaphor for clarity

## Roadmap

### Phase 1: Core Illumination (Current)
- [x] Rust MCP server foundation
- [ ] Basic light-guided reasoning
- [ ] Shadow detection system

### Phase 2: Advanced Illumination
- [ ] Multi-spectrum shadow analysis
- [ ] Adaptive brightness control
- [ ] Pattern-based light guidance
- [ ] Temporal shadow tracking

### Phase 3: Brilliant Clarity
- [ ] Learning from illumination patterns
- [ ] Model-specific light tuning
- [ ] Performance optimization
- [ ] Benchmark validation

## Contributing

Help us bring light to AI reasoning! We welcome contributions exploring:
- High-performance illumination algorithms
- Metacognitive light patterns  
- Shadow detection improvements

## License

MIT License - see LICENSE file for details

## Acknowledgments

- NIRVANA project for metacognitive insights
- Cognitive science research on attention and fixation
- The eternal human metaphor of light as understanding

---

*"In the right light, at the right time, everything is extraordinary."* - Aaron Rose