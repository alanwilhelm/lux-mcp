# Model Display in Lux-MCP Tools

Both `traced_reasoning` and `biased_reasoning` tools display the models they are using.

## traced_reasoning Model Display

When you call traced_reasoning, each thought shows the model used:

```
🧠 **REASONING THOUGHT** 🧠

Thought 2 of 5: [Type: Analysis]
Model: gpt-4                    <-- MODEL DISPLAYED HERE
Confidence: 0.85

---

[Thought content here...]

---
```

## biased_reasoning Model Display

When you call biased_reasoning, models are shown in multiple places:

### 1. At the top of the output:
```
⚖️ **BIAS-CHECKED REASONING COMPLETE** ⚖️

🤖 **Models Used:**
• Primary: gpt-4               <-- PRIMARY MODEL
• Verifier: o4-mini           <-- VERIFIER MODEL
```

### 2. In the detailed process log:
```
📋 **DETAILED PROCESS LOG** 📋

🧠 **Step 1 - PrimaryReasoning**
⏰ Time: 2024-01-27T10:30:45Z
🤖 Model: gpt-4               <-- MODEL FOR THIS STEP
⚡ Duration: 1250ms

Generated reasoning step:
[Content...]

---

🔍 **Step 1 - BiasChecking**
⏰ Time: 2024-01-27T10:30:46Z
🤖 Model: o4-mini            <-- MODEL FOR BIAS CHECK
⚡ Duration: 450ms

Bias check results:
[Content...]
```

## How Models Are Determined

1. **traced_reasoning**: 
   - Uses the `model` parameter if provided
   - Falls back to `LUX_DEFAULT_REASONING_MODEL` environment variable
   - Default: o3-pro

2. **biased_reasoning**:
   - Primary model: Uses `primary_model` parameter or `LUX_DEFAULT_REASONING_MODEL`
   - Verifier model: Uses `verifier_model` parameter or `LUX_DEFAULT_BIAS_CHECKER_MODEL`
   - Defaults: o3-pro (primary), o4-mini (verifier)

## Example Calls

### traced_reasoning with specific model:
```json
{
  "thought": "Analyze this code",
  "thought_number": 1,
  "total_thoughts": 3,
  "next_thought_needed": true,
  "model": "gpt-4"  // Will display "Model: gpt-4"
}
```

### biased_reasoning with specific models:
```json
{
  "query": "Should we implement this feature?",
  "primary_model": "gpt-4",      // Will show in "Primary: gpt-4"
  "verifier_model": "o4-mini"    // Will show in "Verifier: o4-mini"
}
```