# Lux MCP Terminology Guide

## Core Concepts - What We Call Things

### Instead of "Thoughts" → **Cognitive Frames**
- Each step in reasoning is a "frame" in the cognitive process
- Example: "Frame 1 of 5" instead of "Thought 1 of 5"

### Instead of "Thinking" → **Cognitive Processing**
- The act of reasoning through problems
- Example: "Processing frame 3..." instead of "Thinking step 3..."

### Instead of "Brain/Mind" → **Reasoning Engine**
- The underlying AI system doing the work
- Example: "Reasoning engine activated" instead of "Brain engaged"

## Reasoning Terminology

### **Cognitive Frames** (formerly "thoughts")
- **Initial Frame** - Problem statement and setup
- **Exploration Frame** - Investigating possibilities
- **Analysis Frame** - Deep examination of specifics
- **Synthesis Frame** - Combining insights
- **Conclusion Frame** - Final determination

### **Reasoning Chain** (formerly "thought chain")
- The connected sequence of cognitive frames
- Example: "Building reasoning chain..." instead of "Following thought process..."

### **Metacognitive Monitoring** (stays the same)
- The system watching the reasoning quality
- Detects loops, drift, and degradation

### **Cognitive Coherence** (formerly "semantic similarity")
- How well frames connect to each other
- Measured as percentage alignment

### **Information Density** (stays the same)
- How much useful content per frame
- Quality metric for reasoning

## Visual Indicators

### Progress Indicators
- ⚡ **Power/Energy** - System activation
- 🔮 **Crystal Ball** - Synthesis/prediction
- 🎯 **Target** - Goal/conclusion
- 💫 **Sparkles** - Processing/computing
- 🔍 **Magnifier** - Analysis/exploration
- 🌿 **Branch** - Alternative paths
- ♻️ **Recycle** - Revision/correction
- 🚀 **Rocket** - Initialization

### Status Indicators
- 🟢 **Green** - Optimal (>80%)
- 🟡 **Yellow** - Acceptable (50-80%)
- 🔴 **Red** - Needs attention (<50%)
- ⚠️ **Warning** - Intervention needed
- ✅ **Check** - Complete/ready
- 🔄 **Cycle** - In progress

## Tool Names (Current → Better)

### Current Tool Names
- `traced_reasoning` → Could be: `cognitive_trace` or `frame_processor`
- `biased_reasoning` → Could be: `dual_validation` or `bias_guard`
- `planner` → Could be: `sequence_architect` or `task_designer`
- `confer` → Stays simple and clear
- `illumination_status` → Could be: `system_diagnostics` or `cognitive_status`

## Process Descriptions

### Instead of "Let me think about this..."
- "Initializing cognitive frame analysis..."
- "Processing query through reasoning engine..."
- "Constructing reasoning chain..."

### Instead of "I'm thinking step by step..."
- "Building sequential cognitive frames..."
- "Executing frame-by-frame analysis..."
- "Processing through reasoning chain..."

### Instead of "My thoughts are..."
- "Current frame analysis indicates..."
- "Cognitive processing reveals..."
- "Reasoning engine has determined..."

## Quality Metrics

### **Cognitive Load** (formerly "mental effort")
- How hard the system is working
- Measured 0.0 to 1.0

### **Frame Coherence** (formerly "thought consistency")
- How well frames connect
- Measured as correlation coefficient

### **Reasoning Depth** (formerly "thinking depth")
- How many layers of analysis
- Measured in frame count

### **Synthesis Readiness** (formerly "ready to conclude")
- Whether enough frames processed
- Boolean indicator

## Error States

### **Cognitive Loop** (formerly "circular reasoning")
- When frames repeat similar content
- Triggers intervention

### **Context Drift** (formerly "distractor fixation")
- When frames lose relevance to query
- Measured as distance from origin

### **Frame Degradation** (formerly "quality degradation")
- When later frames lose coherence
- Triggers quality intervention

## Example Usage

### Old Way:
```
"Thought 3 of 5: Let me think about the implications..."
🧠 Thinking... (85% confidence)
```

### New Way:
```
"Cognitive Frame 3/5: Analysis Phase"
⚡ Processing frame... [████████░░] 85%
🔮 Reasoning engine: Pattern recognition active
```

### Old Output:
```
Beginning thought process...
Thought 1: Understanding the problem
Thought 2: Exploring solutions
Thought 3: Analyzing trade-offs
```

### New Output:
```
═══════════════════════════════════════
⚡ REASONING CHAIN INITIALIZED
═══════════════════════════════════════
[🚀] Frame 1: Problem Decomposition
[🔍] Frame 2: Solution Space Exploration  
[⚡] Frame 3: Trade-off Analysis Matrix
═══════════════════════════════════════
```

## API Response Fields

### Current → Better
- `thought_number` → `frame_index`
- `total_thoughts` → `frame_count`
- `thought_content` → `frame_output`
- `thought_type` → `frame_phase`
- `next_thought_needed` → `continue_chain`
- `thinking` → `processing`
- `thought_history` → `reasoning_chain`

## Status Messages

### System States
- "Cognitive engine: ACTIVE"
- "Reasoning chain: IN_PROGRESS"
- "Metacognitive monitor: WATCHING"
- "Synthesis module: COLLECTING"
- "Bias detector: SCANNING"

### Completion States
- "Reasoning chain: COMPLETE"
- "Cognitive frames: PROCESSED"
- "Final synthesis: READY"
- "Confidence threshold: ACHIEVED"

## Benefits of This Terminology

1. **More Technical** - Sounds sophisticated and engineered
2. **More Accurate** - Better describes what's actually happening
3. **Less Anthropomorphic** - Avoids "thinking/brain" metaphors
4. **More Distinctive** - Unique to Lux MCP
5. **Clearer Structure** - Frames/chains are more concrete than thoughts

## Implementation Note

We can keep the internal code using current names for backward compatibility, but update:
1. User-facing output messages
2. Documentation
3. API response descriptions
4. Log messages
5. Comments in code

This gives us a unique identity while maintaining code stability!