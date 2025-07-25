# Lux MCP - Focused Design Document

## Core Concept

A single-purpose MCP server that **illuminates your thinking** with real-time light guidance to prevent mental darkness and overthinking spirals. Nothing more, nothing less.

## Single Tool: `lux_think`

### Input Parameters
```json
{
  "thought": "Current thinking step content",
  "thought_number": 1,
  "total_thoughts": 10,
  "model": "claude"  // or "gpt4", "gemini", etc.
}
```

### What It Does

1. **Illuminates Your Thinking** in real-time, detecting:
   - **Circular Shadows**: When thoughts loop in darkness (>85% similarity)
   - **False Lights**: When following distractors away from the path
   - **Dimming Clarity**: When thought brightness and coherence fade

2. **Provides Illuminating Guidance**:
   - **Refocus Beacon**: "Following false light. Return to illuminated path: [original problem]"
   - **Consolidation Glow**: "Light fading. Gather your insights before darkness falls."
   - **Breakthrough Flash**: "Walking in circles. Shine light on a new path."

3. **Tracks Illumination State**:
   - Current phase: Dawn → Daylight → Twilight → Dusk
   - Brightness level: 0.0 to 1.0
   - Light trend: Brightening / Steady / Dimming

### Output Structure
```json
{
  "content": "Model's thinking response",
  "illumination": {
    "phase": "daylight",
    "brightness": 0.85,
    "shadow_score": 0.3,
    "false_light_score": 0.2,
    "light_trend": "steady",
    "luminosity": 0.4
  },
  "intervention": {
    "needed": false,
    "type": null,
    "guidance": null
  },
  "continue": true
}
```

## Key Algorithms (from nirvana-mcp)

### 1. Circular Reasoning Detection
```rust
// Track semantic similarity between recent thoughts
// Alert when similarity > 0.85
// Use sliding window of last 5 thoughts
```

### 2. Distractor Fixation Detection
```rust
// Compare current thought to initial problem statement
// Measure concept drift using keyword overlap
// Alert when relevance < 0.3
```

### 3. Quality Trend Analysis
```rust
// Track information density (new concepts per thought)
// Monitor coherence (logical flow)
// Detect declining patterns over 3-thought windows
```

### 4. Adaptive Intervention
```rust
// Base pause probability on problem complexity
// Adjust for metacognitive signals:
//   +0.3 for high distractor score
//   +0.2 for declining quality
//   +0.4 for circular reasoning
// Intervene when probability > 0.5
```

## Implementation Plan

### Phase 1: Core Monitoring
1. Implement the three detection algorithms
2. Create cognitive state tracker
3. Build intervention decision system

### Phase 2: Model Integration
1. Connect to OpenAI/Anthropic/Google APIs
2. Add model-specific prompting
3. Handle API responses

### Phase 3: Session Management
1. Track thought history
2. Maintain metacognitive state across calls
3. Calculate trends over time

## Why This Design?

- **Focused**: One tool, one purpose - metacognitive thinking
- **Sophisticated**: Real monitoring algorithms, not placeholders
- **Practical**: Addresses the actual "overthinking" problem from research
- **Simple Interface**: Just 4 parameters, clear outputs

## Example Usage

```json
// First thought
{
  "tool": "lux_think",
  "arguments": {
    "thought": "I need to design a distributed cache system",
    "thought_number": 1,
    "total_thoughts": 8,
    "model": "claude"
  }
}

// Response shows bright illumination
{
  "content": "Let me illuminate the key requirements...",
  "illumination": {
    "phase": "dawn",
    "brightness": 0.95,
    "shadow_score": 0.0,
    "false_light_score": 0.0,
    "light_trend": "steady",
    "luminosity": 0.2
  },
  "intervention": {
    "needed": false
  },
  "continue": true
}

// Later thought following false light
{
  "tool": "lux_think",
  "arguments": {
    "thought": "Actually, let me dive into Redis internals...",
    "thought_number": 5,
    "total_thoughts": 8,
    "model": "claude"
  }
}

// Response with illumination guidance
{
  "content": "Redis uses...",
  "illumination": {
    "phase": "daylight",
    "brightness": 0.6,
    "shadow_score": 0.2,
    "false_light_score": 0.75,
    "light_trend": "dimming",
    "luminosity": 0.7
  },
  "intervention": {
    "needed": true,
    "type": "refocus_beacon",
    "guidance": "🔦 REFOCUS BEACON: You're following a false light into implementation darkness. Return to the illuminated path of high-level cache design."
  },
  "continue": false
}
```

## Technical Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      lux_think Tool                          │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │   Thought     │  │   Shadow     │  │  Illumination    │  │
│  │   History     │→ │  Detection   │→ │    Guidance      │  │
│  └──────────────┘  └──────────────┘  └──────────────────┘  │
│                            ↓                                 │
│                    ┌──────────────┐                         │
│                    │ Model Caller │                         │
│                    │ (LLM APIs)   │                         │
│                    └──────────────┘                         │
│                            ↓                                 │
│                    ┌──────────────┐                         │
│                    │   Response    │                         │
│                    │  Formatting   │                         │
│                    └──────────────┘                         │
└─────────────────────────────────────────────────────────────┘
```

## Success Metrics

1. **Prevents Overthinking**: Detects and intervenes before quality degrades
2. **Maintains Focus**: Catches distraction within 2-3 thoughts
3. **Breaks Loops**: Identifies circular reasoning patterns
4. **Guides Effectively**: Clear, actionable intervention messages

## Non-Goals

- Multiple tools or modes
- Memory/storage systems
- Complex session branching
- General-purpose chat

This is a precision instrument for one thing: **illuminating your thoughts to prevent mental darkness**.