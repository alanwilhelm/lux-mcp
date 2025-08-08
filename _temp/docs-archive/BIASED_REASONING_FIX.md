# Biased Reasoning Model Fix

## Summary
Fixed biased_reasoning tool to ALWAYS use configured default models regardless of request parameters.

## Changes Made

1. **Modified `src/tools/biased_reasoning.rs`**:
   - Removed logic that respected `primary_model` and `verifier_model` from request
   - Now always uses `LUX_DEFAULT_REASONING_MODEL` (default: o3-pro) for primary reasoning
   - Now always uses `LUX_DEFAULT_BIAS_CHECKER_MODEL` (default: o4-mini) for bias checking
   - Added logging when user tries to override models

2. **Updated Documentation**:
   - **README.md**: Added note that biased_reasoning always uses configured defaults
   - **CLAUDE.md**: Added note about forced model usage
   - **QUICKSTART.md**: Updated examples to remove model parameters and explain behavior

## Rationale
This ensures consistent behavior and prevents users from accidentally using inappropriate models for bias checking. The o3-pro/o4-mini combination provides:
- Deep reasoning capability from o3-pro
- Fast, efficient bias checking from o4-mini
- Consistent results regardless of user input

## Testing
Created `test_biased_reasoning_forced_models.sh` to verify the behavior.

## Configuration
Set these environment variables to control the models:
```bash
LUX_DEFAULT_REASONING_MODEL=o3-pro      # Primary reasoner
LUX_DEFAULT_BIAS_CHECKER_MODEL=o4-mini  # Bias checker
```